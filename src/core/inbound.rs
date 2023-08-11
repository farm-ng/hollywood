use crate::compute::context::Context;
use crate::core::{actor_builder::ActorBuilder, outbound::OutboundHub, value::Value};

/// The inbound hub is a collection of inbound channels.
pub trait InboundHub<Prop, State: Value, OutboundHub, M: InboundMessage>: Send + Sync {
    /// Create a new inbound hub for an actor.
    fn from_builder(
        builder: &mut ActorBuilder<Prop, State, OutboundHub, M>,
        actor_name: &str,
    ) -> Self;
}

/// An empty inbound hub - for actors with no inbound channels.
#[derive(Debug, Clone)]
pub struct NullInbound {}

impl<Prop, State: Value, OutboundHub, NullMessage: InboundMessage>
    InboundHub<Prop, State, OutboundHub, NullMessage> for NullInbound
{
    fn from_builder(
        _builder: &mut ActorBuilder<Prop, State, OutboundHub, NullMessage>,
        _actor_name: &str,
    ) -> Self {
        Self {}
    }
}

/// Inbound channel to receive messages of a specific type `T`.
/// 
/// Inbound channels can be connected to one or more outbound channels of upstream actors. 
#[derive(Debug, Clone)]
pub struct InboundChannel<T, M: InboundMessage> {
    /// Unique identifier of the inbound channel.
    pub name: String,
    /// Name of the actor that the inbound messages are for.
    pub actor_name: String,
    pub(crate) sender: tokio::sync::mpsc::Sender<M>,
    pub(crate) phantom: std::marker::PhantomData<T>,
}

impl<T: Default + Clone + Send + Sync + std::fmt::Debug + 'static, M: InboundMessage>
    InboundChannel<T, M>
{
    /// Creates a new inbound channel.
    pub fn new(
        context: &mut Context,
        actor_name: &str,
        sender: &tokio::sync::mpsc::Sender<M>,
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
pub trait InboundMessage: Send + Sync + Clone + 'static {
    /// Prop type of the receiving actor.
    type Prop: Value;

    /// State type of the receiving actor.
    type State: Value;

    /// OutboundHub type of the receiving actor, to produce outbound messages downstream.
    type OutboundHub: Send + Sync + 'static;

    /// Name of the inbound channel that this message is for.
    fn inbound_channel(&self) -> String;
}

/// Customization point for processing inbound messages.
pub trait OnMessage: InboundMessage {
    /// Process the inbound message - user code with main business logic goes here.
    fn on_message(&self, prop: &Self::Prop, state: &mut Self::State, outbound: &Self::OutboundHub);
}

/// Trait for creating inbound messages of compatible types `T`.
pub trait InboundMessageNew<T>:
    std::fmt::Debug + Send + Sync + Clone + 'static + InboundMessage
{
    /// Create a new inbound message from the inbound channel name and the message value of type `T`.
    fn new(inbound_channel: String, value: T) -> Self;
}

/// Message forwarder.
pub trait ForwardMessage<
Prop: Value,
State: Value, OutboundHub, M: InboundMessage> {
    /// Forward the message to the OnMessage customization point.
    fn forward_message(&self, prop: &Prop, state: &mut State, outbound: &OutboundHub, msg: M);
}

impl<
        T: Default + Clone + Send + Sync + std::fmt::Debug + 'static,
        Prop: Value,
        State: Value,
        OutboundHub,
        M: OnMessage<Prop=Prop, State = State, OutboundHub = OutboundHub>,
    > ForwardMessage<Prop,State, OutboundHub, M> for InboundChannel<T, M>
{
    fn forward_message(&self, prop: &Prop, state: &mut State, outbound: &OutboundHub, msg: M) {
        msg.on_message(prop, state, outbound);
    }
}

/// Null message is a marker type for actors with no inbound channels.
#[derive(Debug)]
pub enum NullMessage<P: Value, S: Value, O: OutboundHub> {
    /// Null message.
    NullMessage(std::marker::PhantomData<(P, S, O)>),
}

impl<P: Value, S: Value, O: OutboundHub> NullMessage<P, S, O> {
    /// Creates a new null message.
    pub fn new() -> Self {
        NullMessage::NullMessage(std::marker::PhantomData {})
    }
}

impl<P: Value, S: Value, O: OutboundHub> Default for NullMessage<P, S, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Value, S: Value, O: OutboundHub> Clone for NullMessage<P, S, O> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<P: Value, S: Value, O: OutboundHub> InboundMessage for NullMessage<P, S, O> {
    type Prop = P;
    type State = S;
    type OutboundHub = O;

    fn inbound_channel(&self) -> String {
        "".to_owned()
    }
}

impl<P: Value, S: Value, O: OutboundHub> OnMessage for NullMessage<P, S, O> {
    fn on_message(&self, _prop: &P, _state: &mut Self::State, _outputs: &Self::OutboundHub) {}
}
