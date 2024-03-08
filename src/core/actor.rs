use async_trait::async_trait;
use std::collections::HashMap;
use tokio::select;

use crate::compute::context::Context;
use crate::core::actor_builder::ActorBuilder;
use crate::core::inbound::ForwardMessage;
use crate::core::inbound::InboundHub;
use crate::core::inbound::InboundMessage;
use crate::core::outbound::OutboundHub;
use crate::core::runner::DefaultRunner;
use crate::core::runner::Runner;

use super::request::RequestHub;

/// A generic actor in the hollywood compute graph framework.
///
/// An actor consists of its unique name, a set of inbound channels, a set of
/// outbound channels as well as its properties, state and runner types.
///
/// The generic actor struct is merely a user-facing facade to configure network connections. Actual
/// properties, state and inbound routing is stored in the [ActorNode] structs.
pub struct GenericActor<Prop, Inbound, State, Outbound: OutboundHub, Request, Run> {
    /// unique identifier of the actor
    pub actor_name: String,
    /// a collection of inbound channels
    pub inbound: Inbound,
    /// a collection of outbound channels
    pub outbound: Outbound,
    /// a collection of request channels    
    pub request: Request,
    pub(crate) phantom: std::marker::PhantomData<(Prop, State, Run)>,
}

/// An actor of the default runner type, but otherwise generic over its, prop, state, inbound
/// and outbound channel types.
pub type Actor<Prop, Inbound, State, OutboundHub, Request> = GenericActor<
    Prop,
    Inbound,
    State,
    OutboundHub,
    Request,
    DefaultRunner<Prop, Inbound, State, OutboundHub, Request>,
>;

/// New actor from properties and state.
pub trait FromPropState<
    Prop,
    Inbound: InboundHub<Prop, State, Outbound, Request, M>,
    State,
    Outbound: OutboundHub,
    M: InboundMessage,
    Request: RequestHub<M>,
    Run: Runner<Prop, Inbound, State, Outbound, Request, M>,
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
    ) -> GenericActor<Prop, Inbound, State, Outbound, Request, Run> {
        let actor_name = context.add_new_unique_name(Self::name_hint(&prop).to_string());
        let out = Outbound::from_context_and_parent(context, &actor_name);

        let mut builder = ActorBuilder::new(context, &actor_name, prop, initial_state);

        let request = Request::from_parent_and_sender(&actor_name, &builder.sender);

        let inbound = Inbound::from_builder(&mut builder, &actor_name);
        builder.build::<Inbound, Run>(inbound, out, request)
    }
}

/// Actor node of the pipeline. It is created by the [Runner::new_actor_node()] method.
#[async_trait]
pub trait ActorNode {
    /// Return the actor's name.
    fn name(&self) -> &String;

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

/// A table to forward outbound messages to message handlers of downstream actors.
pub type ForwardTable<Prop, State, OutboundHub, Request, M> =
    HashMap<String, Box<dyn ForwardMessage<Prop, State, OutboundHub, Request, M> + Send + Sync>>;

pub(crate) struct ActorNodeImpl<Prop, State, OutboundHub, Request, M> {
    pub(crate) name: String,
    pub(crate) prop: Prop,
    pub(crate) state: Option<State>,
    pub(crate) receiver: Option<tokio::sync::mpsc::Receiver<M>>,
    pub(crate) outbound: OutboundHub,
    pub(crate) request: Request,
    pub(crate) forward: ForwardTable<Prop, State, OutboundHub, Request, M>,
}

impl<Prop, State, Outbound: OutboundHub, Request, M: InboundMessage>
    ActorNodeImpl<Prop, State, Outbound, Request, M>
{
}

#[async_trait]
impl<
        Prop: std::marker::Send + std::marker::Sync + 'static,
        State: std::marker::Send + std::marker::Sync + 'static,
        Outbound: OutboundHub,
        Request: RequestHub<M>,
        M: InboundMessage,
    > ActorNode for ActorNodeImpl<Prop, State, Outbound, Request, M>
{
    fn name(&self) -> &String {
        &self.name
    }

    async fn run(&mut self, kill: tokio::sync::broadcast::Receiver<()>) {
        self.outbound.activate();
        self.request.activate();

        let (state, recv) = on_message(
            self.name.clone(),
            &self.prop,
            OnMessageMutValues {
                state: self.state.take().unwrap(),
                receiver: self.receiver.take().unwrap(),
                kill,
            },
            &self.forward,
            &self.outbound,
            &self.request,
        )
        .await;
        self.state = Some(state);
        self.receiver = Some(recv);
    }
}

pub(crate) struct OnMessageMutValues<State, M: InboundMessage> {
    state: State,
    receiver: tokio::sync::mpsc::Receiver<M>,
    kill: tokio::sync::broadcast::Receiver<()>,
}

pub(crate) async fn on_message<
    Prop,
    State,
    Outbound: Sync + Send,
    Request: Sync + Send,
    M: InboundMessage,
>(
    _actor_name: String,
    prop: &Prop,
    mut values: OnMessageMutValues<State, M>,
    forward: &ForwardTable<Prop, State, Outbound, Request, M>,
    outbound: &Outbound,
    request: &Request,
) -> (State, tokio::sync::mpsc::Receiver<M>) {
    loop {
        select! {
            _ = values.kill.recv() => {

                while values.receiver.try_recv().is_ok(){}

                return (values.state, values.receiver);
            },
            m = values.receiver.recv() => {
                if m.is_none() {
                    let _ = values.kill.try_recv();
                    return (values.state, values.receiver);
                }
                let m = m.unwrap();
                let t = forward.get(&m.inbound_channel());
                if t.is_none() {
                    continue;
                }
                t.unwrap().forward_message(prop, &mut values.state, outbound, request, m);
            }
        }
    }
}
