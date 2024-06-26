use crate::prelude::*;

/// The inbound hub is a collection of inbound channels.
pub trait IsInboundHub<
    Prop,
    State,
    OutboundHub,
    OutRequest: IsOutRequestHub<M>,
    M: IsInboundMessage,
    R: IsInRequestMessage,
>: Send + Sync
{
    /// Create a new inbound hub for an actor.
    fn from_builder(
        builder: &mut ActorBuilder<Prop, State, OutboundHub, OutRequest, M, R>,
        actor_name: &str,
    ) -> Self;
}

/// An empty inbound hub - for actors with no inbound channels.
#[derive(Debug, Clone)]
pub struct NullInbound {}

impl<
        Prop,
        State,
        OutboundHub,
        InRequestMessage: IsInRequestMessage,
        Message: IsInboundMessage,
        OutRequest: IsOutRequestHub<Message>,
    > IsInboundHub<Prop, State, OutboundHub, OutRequest, Message, InRequestMessage>
    for NullInbound
{
    fn from_builder(
        _builder: &mut ActorBuilder<
            Prop,
            State,
            OutboundHub,
            OutRequest,
            Message,
            InRequestMessage,
        >,
        _actor_name: &str,
    ) -> Self {
        Self {}
    }
}

/// Inbound channel to receive messages of a specific type `T`.
///
/// Inbound channels can be connected to one or more outbound channels of upstream actors.
#[derive(Debug, Clone)]
pub struct InboundChannel<T, M: IsInboundMessage> {
    /// Unique identifier of the inbound channel.
    pub name: String,
    /// Name of the actor that the inbound messages are for.
    pub actor_name: String,
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<M>,
    pub(crate) phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Send + Sync + std::fmt::Debug + 'static, M: IsInboundMessage> InboundChannel<T, M> {
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
            sender: sender.clone(),
            phantom: std::marker::PhantomData {},
        }
    }
}

/// Inbound messages to be received by the actor.
pub trait IsInboundMessage: Send + Sync + Clone + 'static {
    /// Prop type of the receiving actor.
    type Prop;

    /// State type of the receiving actor.
    type State;

    /// IsOutboundHub type of the receiving actor, to produce outbound messages downstream.
    type OutboundHub: Send + Sync + 'static;

    /// IsRequestHub type of the receiving actor, to send requests upstream.
    type OutRequestHub: Send + Sync + 'static;

    /// Name of the inbound channel that this message is for.
    fn inbound_channel(&self) -> String;
}

/// Customization point for processing inbound messages.
pub trait HasOnMessage: IsInboundMessage {
    /// Process the inbound message - user code with main business logic goes here.
    fn on_message(
        self,
        prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        request: &Self::OutRequestHub,
    );
}

/// Trait for creating inbound messages of compatible types `T`.
pub trait IsInboundMessageNew<T>:
    std::fmt::Debug + Send + Sync + Clone + 'static + IsInboundMessage
{
    /// Create a new inbound message from the inbound channel name and the message value of type `T`.
    fn new(inbound_channel: String, value: T) -> Self;
}

/// Message forwarder.
pub trait HasForwardMessage<Prop, State, OutboundHub, OutRequestHub, M: IsInboundMessage> {
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
        T: Clone + Send + Sync + std::fmt::Debug + 'static,
        Prop,
        State,
        OutboundHub,
        OutRequestHub,
        M: HasOnMessage<
            Prop = Prop,
            State = State,
            OutboundHub = OutboundHub,
            OutRequestHub = OutRequestHub,
        >,
    > HasForwardMessage<Prop, State, OutboundHub, OutRequestHub, M> for InboundChannel<T, M>
{
    fn forward_message(
        &self,
        prop: &Prop,
        state: &mut State,
        outbound: &OutboundHub,
        request: &OutRequestHub,
        msg: M,
    ) {
        msg.on_message(prop, state, outbound, request);
    }
}

/// Null message is a marker type for actors with no inbound channels.
#[derive(Debug)]
pub enum NullMessage {
    /// Null message.
    Null,
}

impl Clone for NullMessage {
    fn clone(&self) -> Self {
        Self::Null
    }
}

impl IsInboundMessage for NullMessage {
    type Prop = NullProp;
    type State = NullState;
    type OutboundHub = NullOutbound;
    type OutRequestHub = NullOutRequests;

    fn inbound_channel(&self) -> String {
        "".to_owned()
    }
}

impl HasOnMessage for NullMessage {
    fn on_message(
        self,
        _prop: &Self::Prop,
        _state: &mut Self::State,
        _outputs: &Self::OutboundHub,
        _request: &Self::OutRequestHub,
    ) {
    }
}
