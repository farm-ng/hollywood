use std::collections::HashMap;

use crate::compute::context::Context;
use crate::core::{
    actor::GenericActor,
    inbound::{InboundMessage, ForwardMessage, InboundReceptionTrait},
    outbound::OutboundDistributionTrait,
    runner::RunnerTrait,
    state::StateTrait,
};

/// A build pattern for the actor.
pub struct ActorBuilder<'a, State: StateTrait, OutboundDistribution, M: InboundMessage> {
    /// unique name of the actor
    pub actor_name: String,
    state: State,
    /// execution context
    pub context: &'a mut Context,
    /// a channel for sending messages to the actor
    pub sender: tokio::sync::mpsc::Sender<M>,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<M>,
    /// a collection of inbound channels
    pub forward: HashMap<String, Box<dyn ForwardMessage<State, OutboundDistribution, M> + Send + Sync>>,
}

impl<'a, State: StateTrait, OutboundDistribution: OutboundDistributionTrait, M: InboundMessage>
    ActorBuilder<'a, State, OutboundDistribution, M>
{
    /// Create the actor builder  from given actor name and initial state.
    pub fn new(context: &'a mut Context, actor_name: &str, initial_state: State) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);

        Self {
            actor_name: actor_name.to_owned(),
            state: initial_state,
            context,
            sender: sender.clone(),
            receiver,
            forward: HashMap::new(),
        }
    }

    /// Create a new actor given its inbound reception and outbound distribution.
    pub fn build<
        InboundReception: InboundReceptionTrait<State, OutboundDistribution, M>,
        Runner: RunnerTrait<InboundReception, State, OutboundDistribution, M>,
    >(
        self,
        inbound: InboundReception,
        outbound: OutboundDistribution,
    ) -> GenericActor<InboundReception, State, OutboundDistribution, Runner> {
        let mut actor = GenericActor {
            actor_name: self.actor_name.clone(),
            inbound,
            outbound,
            phantom: std::marker::PhantomData {},
        };
        let sleeping = Runner::new_dormant_actor(
            self.actor_name,
            self.state,
            self.receiver,
            self.forward,
            actor.outbound.extract(),
        );
        self.context.actors.push(sleeping);
        actor
    }
}
