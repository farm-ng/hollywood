use std::collections::HashMap;

use crate::core::{
    actor::{DormantActor, DynDormantActor},
    inbound::{InboundMessage, ForwardMessage, InboundReceptionTrait},
    outbound::OutboundDistributionTrait,
    state::StateTrait,
};

/// Runner of the hollywood compute graph.
pub trait RunnerTrait<
    InboundReception: InboundReceptionTrait<State, OutboundDistribution, M>,
    State: StateTrait,
    OutboundDistribution: OutboundDistributionTrait,
    M: InboundMessage,
>
{
    /// Produces a new dormant actor.
    fn new_dormant_actor(
        name: String,
        state: State,
        receiver: tokio::sync::mpsc::Receiver<M>,
        forward: HashMap<String, Box<dyn ForwardMessage<State, OutboundDistribution, M> + Send + Sync>>,
        outbound: OutboundDistribution,
    ) -> Box<dyn DynDormantActor + Send + Sync>;
}

/// The default runner for the hollywood compute graph.
pub struct DefaultRunner<
    InboundReception: Send + Sync,
    State: StateTrait,
    OutboundDistribution: Send + Sync + 'static,
> {
    phantom: std::marker::PhantomData<(InboundReception, State, OutboundDistribution)>,
}

impl<State: StateTrait, InboundReception: Send + Sync, OutboundDistribution: Send + Sync + 'static>
    DefaultRunner<InboundReception, State, OutboundDistribution>
{
    /// Create a new default runner.
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<State: StateTrait, InboundReception: Send + Sync, OutboundDistribution: Send + Sync + 'static> Default
    for DefaultRunner<InboundReception, State, OutboundDistribution>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        InboundReception: InboundReceptionTrait<State, OutboundDistribution, M>,
        State: StateTrait,
        OutboundDistribution: OutboundDistributionTrait,
        M: InboundMessage,
    > RunnerTrait<InboundReception, State, OutboundDistribution, M>
    for DefaultRunner<InboundReception, State, OutboundDistribution>
{
    fn new_dormant_actor(
        name: String,
        init_state: State,
        receiver: tokio::sync::mpsc::Receiver<M>,
        forward: HashMap<String, Box<dyn ForwardMessage<State, OutboundDistribution, M> + Send + Sync>>,
        outbound: OutboundDistribution,
    ) -> Box<dyn DynDormantActor + Send + Sync> {
        Box::new(DormantActor::<State, OutboundDistribution, M> {
            name: name.clone(),
            receiver,
            outbound,
            forward,
            init_state,
        })
    }
}
