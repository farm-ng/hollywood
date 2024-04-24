use crate::core::runner::Runner;
use crate::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::select;

/// A generic actor in the hollywood compute graph framework.
///
/// An actor consists of its unique name, a set of inbound channels, a set of
/// outbound channels as well as its properties, state and runner types.
///
/// The generic actor struct is merely a user-facing facade to configure network connections. Actual
/// properties, state and inbound routing is stored in the [IsActorNode] structs.
pub struct GenericActor<Prop, Inbound, InRequest, State, Outbound: IsOutboundHub, OutRequest, Run> {
    /// unique identifier of the actor
    pub actor_name: String,
    /// a collection of inbound channels
    pub inbound: Inbound,
    /// a collection of inbound channels
    pub in_requests: InRequest,
    /// a collection of outbound channels
    pub outbound: Outbound,
    /// a collection of request channels    
    pub out_requests: OutRequest,
    pub(crate) phantom: std::marker::PhantomData<(Prop, State, Run)>,
}

/// An actor of the default runner type, but otherwise generic over its, prop, state, inbound
/// and outbound channel types.
pub type Actor<Prop, Inbound, InRequest, State, Outbound, OutRequest> = GenericActor<
    Prop,
    Inbound,
    InRequest,
    State,
    Outbound,
    OutRequest,
    DefaultRunner<Prop, Inbound, InRequest, State, Outbound, OutRequest>,
>;

/// New actor from properties and state.
pub trait HasFromPropState<
    Prop,
    Inbound: IsInboundHub<Prop, State, Outbound, OutRequest, M, R>,
    InRequest: IsInRequestHub<Prop, State, Outbound, OutRequest, M, R>,
    State,
    Outbound: IsOutboundHub,
    M: IsInboundMessage,
    R: IsInRequestMessage,
    OutRequest: IsOutRequestHub<M>,
    Run: Runner<Prop, Inbound, InRequest, State, Outbound, OutRequest, M, R>,
>
{
    /// Produces a hint for the actor. The name_hint is used as a base to
    /// generate a unique name.
    fn name_hint(prop: &Prop) -> String;

    /// Produces a new actor with the given state.
    ///
    /// Also, a dormant actor node is created added to the context.
    fn from_prop_and_state(
        context: &mut Hollywood,
        prop: Prop,
        initial_state: State,
    ) -> GenericActor<Prop, Inbound, InRequest, State, Outbound, OutRequest, Run> {
        let actor_name = context.add_new_unique_name(Self::name_hint(&prop).to_string());
        let out = Outbound::from_context_and_parent(context, &actor_name);

        let mut builder = ActorBuilder::<Prop, State, Outbound, OutRequest, M, R>::new(
            context,
            &actor_name,
            prop,
            initial_state,
        );

        let out_request = OutRequest::from_parent_and_sender(&actor_name, &builder.sender);

        let inbound = Inbound::from_builder(&mut builder, &actor_name);
        let in_request = InRequest::from_builder(&mut builder, &actor_name);
        builder.build::<Inbound, InRequest, Run>(inbound, in_request, out, out_request)
    }
}

/// Actor node of the pipeline. It is created by the [Runner::new_actor_node()] method.
#[async_trait]
pub trait IsActorNode {
    /// Return the actor's name.
    fn name(&self) -> &String;

    /// Run the actor as a node within the compute pipeline:
    ///
    ///   * For each inbound channel there are zero, one or more incoming connections. Messages on
    ///     these incoming streams are merged into a single stream.
    ///   * Messages for all inbound channels are processed sequentially using the
    ///     [HasOnMessage::on_message()] method. Sequential processing is crucial to ensure that
    ///     the actor's state is updated in a consistent manner. Sequential mutable access to the
    ///     state is enforced by the borrow checker at compile time.
    ///   * Outbound messages are produced by [HasOnMessage::on_message()] the method and sent to
    ///     the through the corresponding outbound channel to downstream actors.
    ///
    /// Note: It is an async function which returns a future a completion handler. This method is
    /// not intended to be called directly but is called by the runtime of the pipeline.
    async fn run(&mut self, kill: tokio::sync::broadcast::Receiver<()>);
}

/// A table to forward outbound messages to message handlers of downstream actors.
pub type ForwardTable<Prop, State, OutboundHub, Request, M> =
    HashMap<String, Box<dyn HasForwardMessage<Prop, State, OutboundHub, Request, M> + Send + Sync>>;

/// A table to forward outbound messages to message handlers of downstream actors.
pub type ForwardRequestTable<Prop, State, OutRequestHub, Request, R> = HashMap<
    String,
    Box<dyn HasForwardRequestMessage<Prop, State, OutRequestHub, Request, R> + Send + Sync>,
>;

pub(crate) struct ActorNodeImpl<Prop, State, OutboundHub, OutRequestHub, M, R> {
    pub(crate) name: String,
    pub(crate) prop: Prop,
    pub(crate) state: Option<State>,
    pub(crate) forward: ForwardTable<Prop, State, OutboundHub, OutRequestHub, M>,
    pub(crate) receiver: Option<tokio::sync::mpsc::UnboundedReceiver<M>>,
    pub(crate) outbound: OutboundHub,
    pub(crate) forward_request: ForwardRequestTable<Prop, State, OutboundHub, OutRequestHub, R>,
    pub(crate) request_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<R>>,
    pub(crate) out_request: OutRequestHub,
}

impl<Prop, State, Outbound: IsOutboundHub, Request, R: IsInRequestMessage, M: IsInboundMessage>
    ActorNodeImpl<Prop, State, Outbound, Request, M, R>
{
}

#[async_trait]
impl<
        Prop: std::marker::Send + std::marker::Sync + 'static,
        State: std::marker::Send + std::marker::Sync + 'static,
        Outbound: IsOutboundHub,
        Request: IsOutRequestHub<M>,
        R: IsInRequestMessage,
        M: IsInboundMessage,
    > IsActorNode for ActorNodeImpl<Prop, State, Outbound, Request, M, R>
{
    fn name(&self) -> &String {
        &self.name
    }

    async fn run(&mut self, kill: tokio::sync::broadcast::Receiver<()>) {
        self.outbound.activate();
        self.out_request.activate();

        let (state, recv) = on_message(
            self.name.clone(),
            &self.prop,
            OnMessageMutValues {
                state: self.state.take().unwrap(),
                receiver: self.receiver.take().unwrap(),
                request_receiver: self.request_receiver.take().unwrap(),
                kill,
            },
            &self.forward,
            &self.forward_request,
            &self.outbound,
            &self.out_request,
        )
        .await;
        self.state = Some(state);
        self.receiver = Some(recv);
    }
}

pub(crate) struct OnMessageMutValues<State, M: IsInboundMessage, R: IsInRequestMessage> {
    state: State,
    receiver: tokio::sync::mpsc::UnboundedReceiver<M>,
    request_receiver: tokio::sync::mpsc::UnboundedReceiver<R>,
    kill: tokio::sync::broadcast::Receiver<()>,
}

pub(crate) async fn on_message<
    Prop,
    State,
    Outbound: Sync + Send,
    Request: Sync + Send,
    M: IsInboundMessage,
    R: IsInRequestMessage,
>(
    _actor_name: String,
    prop: &Prop,
    mut values: OnMessageMutValues<State, M, R>,
    forward: &ForwardTable<Prop, State, Outbound, Request, M>,
    forward_request: &ForwardRequestTable<Prop, State, Outbound, Request, R>,
    outbound: &Outbound,
    request: &Request,
) -> (State, tokio::sync::mpsc::UnboundedReceiver<M>) {
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
            },
            m = values.request_receiver.recv() => {
                if m.is_some() {
                    let m = m.unwrap();
                    let t = forward_request.get(&m.in_request_channel());
                    if t.is_none() {
                        continue;
                    }
                    t.unwrap().forward_message(prop, &mut values.state, outbound, request, m);
                } else{
                    tokio::task::yield_now().await;
                }
            }
        }
    }
}
