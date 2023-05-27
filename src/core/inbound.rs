use crate::compute::context::Context;
use crate::core::{
    actor_builder::ActorBuilder, outbound::OutboundDistributionTrait, state::StateTrait,
};

/// InboundReception is a collection of inbound channels for an actor.
pub trait InboundReceptionTrait<State: StateTrait, OutboundDistribution, M: InboundMessage>:
    Send + Sync
{
    /// All inbounds for this actor.
    type AllInbounds;

    /// Create a new reception for an actor.
    fn from_builder(
        builder: &mut ActorBuilder<State, OutboundDistribution, M>,
        actor_name: &str,
    ) -> Self;
}

/// A reception which has no inbound channels.
#[derive(Debug, Clone)]
pub struct NullInbounds {}

impl<State: StateTrait, OutboundDistribution, NullMessage: InboundMessage>
    InboundReceptionTrait<State, OutboundDistribution, NullMessage> for NullInbounds
{
    type AllInbounds = NullInbounds;
    fn from_builder(
        _builder: &mut ActorBuilder<State, OutboundDistribution, NullMessage>,
        _actor_name: &str,
    ) -> Self {
        Self {}
    }
}

/// Inbound channel for messages which are received by the actor.
#[derive(Debug, Clone)]
pub struct InboundChannel<T, M: InboundMessage> {
    /// Unique name of the inbound channel.
    pub name: String,
    /// Name of the actor that the inbound messages are for.
    pub actor_name: String,
    pub(crate) sender: tokio::sync::mpsc::Sender<M>,
    pub(crate) phantom: std::marker::PhantomData<T>,
}

impl<T: Default + Clone + Send + Sync + std::fmt::Debug + 'static, M: InboundMessage>
    InboundChannel<T, M>
{
    /// Create a new inbound for actor in provided context.
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

/// Inbound messages which are received by an actor.
pub trait InboundMessage: Send + Sync + Clone + 'static {
    /// State type of the receiving actor.
    type State: StateTrait;

    /// OutboundDistribution type of the receiving actor, to produce outbound messages downstream.
    type OutboundDistribution: Send + Sync + 'static;

    /// Name of the inbound that this message is for.
    fn inbound_name(&self) -> String;
}

/// Customization point for processing inbound messages.
pub trait OnMessage: InboundMessage {
    /// Process the inbound message - user code goes here.
    fn on_message(&self, state: &mut Self::State, outbound: &Self::OutboundDistribution);
}

/// Trait for creating inbound messages.
pub trait InboundMessageNew<T>:
    std::fmt::Debug + Send + Sync + Clone + 'static + InboundMessage
{
    /// Create a new inbound message.
    fn new(inbound_name: String, msg: T) -> Self;
}

/// Forward message, to be processed by OnMessage customization point.
pub trait ForwardMessage<State: StateTrait, OutboundDistribution, M: InboundMessage> {
    /// Forward the message to the OnMessage customization point.
    fn forward_message(&self, state: &mut State, outbound: &OutboundDistribution, msg: M);
}

impl<
        T: Default + Clone + Send + Sync + std::fmt::Debug + 'static,
        State: StateTrait,
        OutboundDistribution,
        M: OnMessage<State = State, OutboundDistribution = OutboundDistribution>,
    > ForwardMessage<State, OutboundDistribution, M> for InboundChannel<T, M>
{
    fn forward_message(&self, state: &mut State, outbound: &OutboundDistribution, msg: M) {
        msg.on_message(state, outbound);
    }
}

/// Null message is a marker message which does nothing.
#[derive(Debug)]
pub enum NullMessage<S: StateTrait, O: OutboundDistributionTrait> {
    /// Null message.
    NullMessage(std::marker::PhantomData<(S, O)>),
}

impl<S: StateTrait, O: OutboundDistributionTrait> NullMessage<S, O> {
    /// Create a new null message.
    pub fn new() -> Self {
        NullMessage::NullMessage(std::marker::PhantomData {})
    }
}

impl<S: StateTrait, O: OutboundDistributionTrait> Default for NullMessage<S, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: StateTrait, O: OutboundDistributionTrait> Clone for NullMessage<S, O> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<S: StateTrait, O: OutboundDistributionTrait> InboundMessage for NullMessage<S, O> {
    type State = S;
    type OutboundDistribution = O;

    fn inbound_name(&self) -> String {
        "".to_owned()
    }
}

impl<S: StateTrait, O: OutboundDistributionTrait> OnMessage for NullMessage<S, O> {
    fn on_message(&self, _state: &mut Self::State, _outputs: &Self::OutboundDistribution) {}
}