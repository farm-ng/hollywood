use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::select;

use crate::compute::context::Context;
use crate::core::{
    actor_builder::ActorBuilder,
    inbound::{ForwardMessage, InboundHub, InboundMessage},
    outbound::OutboundHub,
    runner::{DefaultRunner, Runner},
    value::Value,
};

/// A generic actor in the hollywood compute graph framework.
///
/// An actor consists of its unique name, a set of inbound channels, a set of
/// outbound channels as well as its properties, state and runner types.
///
/// The generic actor struct is merely a user-facing facade to configure network connections. Actual
/// properties, state and inbound routing is stored in the [DormantActorNode] and [ActorNode]
/// structs.
pub struct GenericActor<Prop, Inbound, State, Outbound: OutboundHub, Run> {
    /// unique identifier of the actor
    pub actor_name: String,
    /// a collection of inbound channels
    pub inbound: Inbound,
    /// a collection of outbound channels
    pub outbound: Outbound,
    pub(crate) phantom: std::marker::PhantomData<(Prop, State, Run)>,
}

/// An actor of the default runner type, but otherwise generic over its, prop, state, inbound
/// and outbound channel types.
pub type Actor<Prop, Inbound, State, OutboundHub> = GenericActor<
    Prop,
    Inbound,
    State,
    OutboundHub,
    DefaultRunner<Prop, Inbound, State, OutboundHub>,
>;

/// New actor from properties and state.
pub trait FromPropState<
    Prop,
    Inbound: InboundHub<Prop, State, Outbound, M>,
    State: Value,
    Outbound: OutboundHub,
    M: InboundMessage,
    Run: Runner<Prop, Inbound, State, Outbound, M>,
>
{
    /// Produces a hint for the actor. The name_hint is used as a base to
    /// generate a unique name.
    fn name_hint(prop: &Prop) -> String;

    /// Produces a new actor with the given state.
    ///
    /// Also, a dormant actor node is created added to the context.
    fn from_prop_and_state(
        context: &mut Context,
        prop: Prop,
        initial_state: State,
    ) -> GenericActor<Prop, Inbound, State, Outbound, Run> {
        let actor_name = context.add_new_unique_name(Self::name_hint(&prop).to_string());
        let out = Outbound::from_context_and_parent(context, &actor_name);

        let mut builder = ActorBuilder::new(context, &actor_name, prop, initial_state);

        let inbound = Inbound::from_builder(&mut builder, &actor_name);
        builder.build::<Inbound, Run>(inbound, out)
    }
}

/// A dormant actor of the pipeline.
#[async_trait]
pub trait DormantActorNode {
    /// An active actor is returned leaving a shell behind.
    ///
    /// Repeated calls to this method may and will often lead to a panic. This method is not
    /// intended to be called directly. It is called by the Pipeline::from_context() construction 
    /// method.
    fn activate(self: Box<Self>) -> Box<dyn ActorNode + Send>;
}

/// Active actor node of the pipeline. It is created by the [DormantActorNode::activate()] method.
#[async_trait]
pub trait ActorNode {
    /// Return the actor's name.
    fn name(&self) -> &String;

    /// Resets the actor to its initial state.
    fn reset(&mut self);

    /// Run the actor as a node within the compute pipeline:
    ///
    ///   * For each inbound channel there are zero, one or more incoming connections. Messages on 
    ///     these incoming streams are merged into a single stream.
    ///   * Messages for all inbound channels are processed sequentially using the 
    ///     [OnMessage::on_message()](crate::core::OnMessage::on_message()) method. Sequential 
    ///     processing is crucial to ensure that the actor's state is updated in a consistent 
    ///     manner. Sequential mutable access to the state is enforced by the borrow checker at 
    ///     compile time.
    ///   * Outbound messages are produced by 
    ///     [OnMessage::on_message()](crate::core::OnMessage::on_message()) the method and sent to 
    ///     the through the corresponding outbound channel to downstream actors.
    ///
    /// Note: It is an async function which returns a future a completion handler. This method is
    /// not intended to be called directly but is called by the runtime of the pipeline.
    async fn run(&mut self, kill: tokio::sync::broadcast::Receiver<()>);
}

pub(crate) struct ActorNodeImpl<Prop, State, OutboundHub, M> {
    pub(crate) name: String,
    pub(crate) prop: Prop,
    pub(crate) init_state: State,
    pub(crate) state: Option<State>,
    pub(crate) receiver: Option<tokio::sync::mpsc::Receiver<M>>,
    pub(crate) outbound: Arc<OutboundHub>,
    pub(crate) forward:
        HashMap<String, Box<dyn ForwardMessage<Prop, State, OutboundHub, M> + Send + Sync>>,
}

impl<Prop, State: Value, Outbound: OutboundHub, M: InboundMessage>
    ActorNodeImpl<Prop, State, Outbound, M>
{
}

#[async_trait]
impl<Prop: Value, State: Value, Outbound: OutboundHub, M: InboundMessage> ActorNode
    for ActorNodeImpl<Prop, State, Outbound, M>
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
            &self.prop,
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

pub(crate) struct DormantActorImpl<Prop: Value, State, OutboundHub, M> {
    pub(crate) name: String,
    pub(crate) prop: Prop,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<M>,
    pub(crate) outbound: OutboundHub,
    pub(crate) forward:
        HashMap<String, Box<dyn ForwardMessage<Prop, State, OutboundHub, M> + Send + Sync>>,
    pub(crate) init_state: State,
}

impl<Prop: Value, State: Value, Outbound: OutboundHub, M: InboundMessage> DormantActorNode
    for DormantActorImpl<Prop, State, Outbound, M>
{
    fn activate(mut self: Box<Self>) -> Box<dyn ActorNode + Send> {
        self.outbound.activate();

        Box::new(ActorNodeImpl::<Prop, State, Outbound, M> {
            name: self.name.clone(),
            prop: self.prop.clone(),
            state: None,
            init_state: self.init_state.clone(),
            receiver: Some(self.receiver),
            outbound: Arc::new(self.outbound),
            forward: self.forward,
        })
    }
}

pub(crate) async fn on_message<
    Prop: Value,
    State: Value,
    Outbound: Sync + Send,
    M: InboundMessage,
>(
    _actor_name: String,
    mut receiver: tokio::sync::mpsc::Receiver<M>,
    prop: &Prop,
    mut state: State,
    forward: &HashMap<String, Box<dyn ForwardMessage<Prop, State, Outbound, M> + Send + Sync>>,
    outbound: Arc<Outbound>,
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
                let t = forward.get(&m.inbound_channel());
                if t.is_none() {
                    continue;
                }
                t.unwrap().forward_message(prop, &mut state, &outbound, m);
            }
        }
    }
}
