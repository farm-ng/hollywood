use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::select;

use crate::compute::context::Context;
use crate::core::{
    actor_builder::ActorBuilder,
    inbound::{ForwardMessage, InboundMessage, InboundReceptionTrait},
    outbound::OutboundDistributionTrait,
    runner::{DefaultRunner, RunnerTrait},
    state::StateTrait,
};

/// A generic actor in the hollywood compute graph framework.
///
/// An actor  consists of it's unique name, a set of inbound channels called `InboundReception`, a set of
/// outbound channels called `OutboundDistribution`, and  it's `State`.
pub struct GenericActor<
    InboundReception,
    State,
    OutboundDistribution: OutboundDistributionTrait,
    Runner,
> {
    /// unique name of the actor
    pub actor_name: String,
    /// a collection of inbound channels
    pub inbound: InboundReception,
    /// a collection of outbound channels
    pub outbound: OutboundDistribution,
    pub(crate) phantom: std::marker::PhantomData<(State, Runner)>,
}

/// An actor with the default runner, but otherwise generic over it's inbound reception, state and
/// outbound OutboundDistribution.
pub type Actor<InboundReception, State, OutboundDistribution> = GenericActor<
    InboundReception,
    State,
    OutboundDistribution,
    DefaultRunner<InboundReception, State, OutboundDistribution>,
>;

/// An actor.
pub trait DynActor<
    InboundReception: InboundReceptionTrait<State, OutboundDistribution, M>,
    State: StateTrait,
    OutboundDistribution: OutboundDistributionTrait,
    M: InboundMessage,
    Runner: RunnerTrait<InboundReception, State, OutboundDistribution, M>,
>
{
    /// Produces a hint for the name of the actor. Typically the hint is mangled with an id to
    /// produce a unique name, if multiple actors of the same type are created.
    fn name_hint() -> String;

    /// Produces a new actor with default state.
    fn new_default_init_state(
        context: &mut Context,
    ) -> GenericActor<InboundReception, State, OutboundDistribution, Runner> {
        Self::new_with_state(context, State::default())
    }

    /// Produces a new actor with the given state.
    fn new_with_state(
        context: &mut Context,
        initial_state: State,
    ) -> GenericActor<InboundReception, State, OutboundDistribution, Runner> {
        let actor_name = context.add_new_unique_name(Self::name_hint().to_string());
        let out = OutboundDistribution::from_context_and_parent(context, &actor_name);

        let mut builder = ActorBuilder::new(context, &actor_name, initial_state);

        let inbound = InboundReception::from_builder(&mut builder, &actor_name);
        builder.build::<InboundReception, Runner>(inbound, out)
    }
}

/// Active actors of a compute graph with runtime state type, inbound and outbound channels.
#[async_trait]
pub trait DynActiveActor {
    /// Get the name of the actor.
    fn name(&self) -> &String;
    /// Reset the actor to its initial state.
    fn reset(&mut self);
    /// Run the actor.
    async fn run(&mut self, kill: tokio::sync::broadcast::Receiver<()>);
}

pub(crate) struct ActiveActor<State, OutboundDistribution, M> {
    pub(crate) name: String,
    pub(crate) init_state: State,
    pub(crate) state: Option<State>,
    pub(crate) receiver: Option<tokio::sync::mpsc::Receiver<M>>,
    pub(crate) outbound: Arc<OutboundDistribution>,
    pub(crate) forward:
        HashMap<String, Box<dyn ForwardMessage<State, OutboundDistribution, M> + Send + Sync>>,
}

impl<State: StateTrait, OutboundDistribution: OutboundDistributionTrait, M: InboundMessage>
    ActiveActor<State, OutboundDistribution, M>
{
}

#[async_trait]
impl<State: StateTrait, OutboundDistribution: OutboundDistributionTrait, M: InboundMessage>
    DynActiveActor for ActiveActor<State, OutboundDistribution, M>
{
    fn name(&self) -> &String {
        &self.name
    }

    fn reset(&mut self) {
        self.state = Some(self.init_state.clone());
    }

    async fn run(&mut self, kill: tokio::sync::broadcast::Receiver<()>) {
        let new_state = self.init_state.clone();
        let (state, recv) = on_message(
            self.name.clone(),
            self.receiver.take().unwrap(),
            new_state,
            &self.forward,
            self.outbound.clone(),
            kill,
        )
        .await;

        self.state = Some(state);
        self.receiver = Some(recv);
    }
}

/// A dynamic dormant actor with runtime state type, inbound and outbound channels.
#[async_trait]
pub trait DynDormantActor {
    /// Activate the actor. An active actor is returned.
    fn activate(self: Box<Self>) -> Box<dyn DynActiveActor + Send>;
}

pub(crate) struct DormantActor<State, OutboundDistribution, M> {
    pub(crate) name: String,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<M>,
    pub(crate) outbound: OutboundDistribution,
    pub(crate) forward:
        HashMap<String, Box<dyn ForwardMessage<State, OutboundDistribution, M> + Send + Sync>>,
    pub(crate) init_state: State,
}

impl<State: StateTrait, OutboundDistribution: OutboundDistributionTrait, M: InboundMessage>
    DynDormantActor for DormantActor<State, OutboundDistribution, M>
{
    fn activate(mut self: Box<Self>) -> Box<dyn DynActiveActor + Send> {
        self.outbound.activate();

        Box::new(ActiveActor::<State, OutboundDistribution, M> {
            name: self.name.clone(),
            state: None,
            init_state: self.init_state.clone(),
            receiver: Some(self.receiver),
            outbound: Arc::new(self.outbound),
            forward: self.forward,
        })
    }
}


pub(crate) async fn on_message<
    State: StateTrait,
    OutboundDistribution: Sync + Send,
    M: InboundMessage,
>(
    _actor_name: String,
    mut receiver: tokio::sync::mpsc::Receiver<M>,
    mut state: State,
    forward: &HashMap<
        String,
        Box<dyn ForwardMessage<State, OutboundDistribution, M> + Send + Sync>,
    >,
    outbound: Arc<OutboundDistribution>,
    mut kill: tokio::sync::broadcast::Receiver<()>,
) -> (State, tokio::sync::mpsc::Receiver<M>) {
    let outbound = outbound.clone();
    loop {
        select! {
            _ = kill.recv() => {

                while receiver.try_recv().is_ok(){}

                return (state, receiver);
            },
            m = receiver.recv() => {
                if m.is_none() {
                    let _ = kill.try_recv();
                    return (state, receiver);
                }
                let m = m.unwrap();
                let t = forward.get(&m.inbound_name());
                if t.is_none() {
                    continue;
                }
                t.unwrap().forward_message(&mut state, &outbound, m);
            }
        }
    }
}