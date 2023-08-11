use std::collections::HashMap;

use crate::core::{
    actor::{DormantActorImpl, DormantActorNode},
    inbound::{ForwardMessage, InboundHub, InboundMessage},
    outbound::OutboundHub,
    value::Value,
};

/// Runner executes the pipeline.
pub trait Runner<
    Prop,
    Inbound: InboundHub<Prop, State, Outbound, M>,
    State: Value,
    Outbound: OutboundHub,
    M: InboundMessage,
>
{
    /// Create a new dormant actor to be stored by the context.
    fn new_dormant_actor(
        name: String,
        prop: Prop,
        state: State,
        receiver: tokio::sync::mpsc::Receiver<M>,
        forward: HashMap<
            String,
            Box<dyn ForwardMessage<Prop, State, Outbound, M> + Send + Sync>,
        >,
        outbound: Outbound,
    ) -> Box<dyn DormantActorNode + Send + Sync>;
}

/// The default runner.
pub struct DefaultRunner<
    Prop,
    Inbound: Send + Sync,
    State: Value,
    Outbound: Send + Sync + 'static,
> {
    phantom: std::marker::PhantomData<(Prop, Inbound, State, Outbound)>,
}

impl<Prop, State: Value, Inbound: Send + Sync, Outbound: Send + Sync + 'static>
    DefaultRunner<Prop, Inbound, State, Outbound>
{
    /// Create a new default runner.
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<Prop, State: Value, Inbound: Send + Sync, Outbound: Send + Sync + 'static> Default
    for DefaultRunner<Prop, Inbound, State, Outbound>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Prop: Value,
        Inbound: InboundHub<Prop, State, Outbound, M>,
        State: Value,
        Outbound: OutboundHub,
        M: InboundMessage,
    > Runner<Prop, Inbound, State, Outbound, M>
    for DefaultRunner<Prop, Inbound, State, Outbound>
{
    fn new_dormant_actor(
        name: String,
        prop: Prop,
        init_state: State,
        receiver: tokio::sync::mpsc::Receiver<M>,
        forward: HashMap<
            String,
            Box<dyn ForwardMessage<Prop, State, Outbound, M> + Send + Sync>,
        >,
        outbound: Outbound,
    ) -> Box<dyn DormantActorNode + Send + Sync> {
        Box::new(DormantActorImpl::<Prop, State, Outbound, M> {
            name: name.clone(),
            prop,
            receiver,
            outbound,
            forward,
            init_state,
        })
    }
}
