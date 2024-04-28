use crate::prelude::*;
use std::sync::Arc;

/// The inbound hub is a collection of inbound channels.
pub trait IsInRequestHub<
    Prop,
    State,
    OutboundHub,
    Request: IsOutRequestHub<M>,
    M: IsInboundMessage,
    R: IsInRequestMessage,
>: Send + Sync
{
    /// Create a new inbound hub for an actor.
    fn from_builder(
        builder: &mut ActorBuilder<Prop, State, OutboundHub, Request, M, R>,
        actor_name: &str,
    ) -> Self;
}

/// An empty inbound hub - for actors with no inbound channels.
#[derive(Debug, Clone)]
pub struct NullInRequests {}

impl<
        Prop,
        State,
        OutboundHub,
        NullInRequestMessage: IsInRequestMessage,
        NullMessage: IsInboundMessage,
        Request: IsOutRequestHub<NullMessage>,
    > IsInRequestHub<Prop, State, OutboundHub, Request, NullMessage, NullInRequestMessage>
    for NullInRequests
{
    fn from_builder(
        _builder: &mut ActorBuilder<
            Prop,
            State,
            OutboundHub,
            Request,
            NullMessage,
            NullInRequestMessage,
        >,
        _actor_name: &str,
    ) -> Self {
        Self {}
    }
}

/// InRequest channel to receive messages of a specific type `T`.
///
/// InRequest channels can be connected to one or more outbound channels of upstream actors.
#[derive(Debug)]
pub struct InRequestChannel<T, M: IsInRequestMessage> {
    /// Unique identifier of the in-request channel.
    pub name: String,
    /// Name of the actor that the requests are sent to.
    pub actor_name: String,
    pub(crate) sender: Arc<tokio::sync::mpsc::UnboundedSender<M>>,
    pub(crate) phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + std::fmt::Debug + 'static, M: IsInRequestMessage> Clone
    for InRequestChannel<T, M>
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            actor_name: self.actor_name.clone(),
            sender: self.sender.clone(),
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<T: Send + Sync + std::fmt::Debug + 'static, M: IsInRequestMessage> InRequestChannel<T, M> {
    /// Creates a new inbound channel.
    pub fn new(
        context: &mut Hollywood,
        actor_name: &str,
        sender: &tokio::sync::mpsc::UnboundedSender<M>,
        name: String,
    ) -> Self {
        context.assert_unique_inbound_name(name.clone(), actor_name);
        Self {
            name,
            actor_name: actor_name.to_owned(),
            sender: Arc::new(sender.clone()),
            phantom: std::marker::PhantomData {},
        }
    }
}

/// InRequest messages to be received by the actor.
pub trait IsInRequestMessage: Send + Sync + 'static {
    /// Prop type of the receiving actor.
    type Prop;

    /// State type of the receiving actor.
    type State;

    /// OutboundHub type of the receiving actor, to produce outbound messages downstream.
    type OutboundHub: Send + Sync + 'static;

    /// OutRequestHub type of the receiving actor.
    type OutRequestHub: Send + Sync + 'static;

    /// Name of the in-request channel that this message is for.
    fn in_request_channel(&self) -> String;
}

/// Customization point for processing inbound request messages.
pub trait HasOnRequestMessage: IsInRequestMessage {
    /// Process the inbound request message.
    fn on_request(
        self,
        prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        request: &Self::OutRequestHub,
    );
}

/// Trait for creating in-request messages of compatible types `T`.
pub trait IsInRequestMessageNew<T>:
    std::fmt::Debug + Send + Sync + 'static + IsInRequestMessage
{
    /// Create a new inbound message from the inbound channel name and the message value of type `T`.
    fn new(inbound_channel: String, value: T) -> Self;
}

/// Message forwarder.
pub trait HasForwardRequestMessage<Prop, State, OutboundHub, OutRequestHub, M: IsInRequestMessage> {
    /// Forward the message to the HasOnMessage customization point.
    fn forward_message(
        &self,
        prop: &Prop,
        state: &mut State,
        outbound: &OutboundHub,
        request: &OutRequestHub,
        msg: M,
    );
}

impl<
        T: Send + Sync + std::fmt::Debug + 'static,
        Prop,
        State,
        OutboundHub,
        OutRequestHub,
        M: HasOnRequestMessage<
            Prop = Prop,
            State = State,
            OutboundHub = OutboundHub,
            OutRequestHub = OutRequestHub,
        >,
    > HasForwardRequestMessage<Prop, State, OutboundHub, OutRequestHub, M>
    for InRequestChannel<T, M>
{
    fn forward_message(
        &self,
        prop: &Prop,
        state: &mut State,
        outbound: &OutboundHub,
        request: &OutRequestHub,
        msg: M,
    ) {
        msg.on_request(prop, state, outbound, request);
    }
}

/// Null message is a marker type for actors with no inbound channels.
#[derive(Debug)]
pub enum NullInRequestMessage {
    /// Null message.
    Null,
}

impl IsInRequestMessage for NullInRequestMessage {
    type Prop = NullProp;
    type State = NullState;
    type OutboundHub = NullOutbound;
    type OutRequestHub = NullOutRequests;

    fn in_request_channel(&self) -> String {
        "".to_owned()
    }
}

impl HasOnRequestMessage for NullInRequestMessage {
    fn on_request(
        self,
        _prop: &Self::Prop,
        _state: &mut Self::State,
        _outputs: &Self::OutboundHub,
        _request: &Self::OutRequestHub,
    ) {
    }
}
