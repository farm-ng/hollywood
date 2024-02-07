use crate::core::{
    inbound::{InboundHub, InboundMessage},
    outbound::OutboundHub,
};

use super::{
    actor::{ActorNodeImpl, ForwardTable},
    request::RequestHub,
    ActorNode,
};

/// Runner executes the pipeline.
pub trait Runner<
    Prop,
    Inbound: InboundHub<Prop, State, Outbound, Request, M>,
    State,
    Outbound: OutboundHub,
    Request: RequestHub<M>,
    M: InboundMessage,
>
{
    /// Create a new actor to be stored by the context.
    fn new_actor_node(
        name: String,
        prop: Prop,
        state: State,
        receiver: tokio::sync::mpsc::Receiver<M>,
        forward: ForwardTable<Prop, State, Outbound, Request, M>,
        outbound: Outbound,
        request: Request,
    ) -> Box<dyn ActorNode + Send + Sync>;
}

/// The default runner.
pub struct DefaultRunner<
    Prop,
    Inbound: Send + Sync,
    State,
    Outbound: Send + Sync + 'static,
    Request: Send + Sync + 'static,
> {
    phantom: std::marker::PhantomData<(Prop, Inbound, State, Outbound, Request)>,
}

impl<
        Prop,
        State,
        Inbound: Send + Sync,
        Outbound: Send + Sync + 'static,
        Request: Send + Sync + 'static,
    > Default for DefaultRunner<Prop, Inbound, State, Outbound, Request>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Prop,
        State,
        Inbound: Send + Sync,
        Outbound: Send + Sync + 'static,
        Request: Send + Sync + 'static,
    > DefaultRunner<Prop, Inbound, State, Outbound, Request>
{
    /// Create a new default runner.
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<
        Prop: std::marker::Send + std::marker::Sync + 'static,
        Inbound: InboundHub<Prop, State, Outbound, Request, M>,
        State: std::marker::Send + std::marker::Sync + 'static,
        Outbound: OutboundHub,
        M: InboundMessage,
        Request: RequestHub<M>,
    > Runner<Prop, Inbound, State, Outbound, Request, M>
    for DefaultRunner<Prop, Inbound, State, Outbound, Request>
{
    fn new_actor_node(
        name: String,
        prop: Prop,
        init_state: State,
        receiver: tokio::sync::mpsc::Receiver<M>,
        forward: ForwardTable<Prop, State, Outbound, Request, M>,
        outbound: Outbound,
        request: Request,
    ) -> Box<dyn ActorNode + Send + Sync> {
        Box::new(ActorNodeImpl::<Prop, State, Outbound, Request, M> {
            name,
            prop,
            state: Some(init_state),
            receiver: Some(receiver),
            outbound,
            forward,
            request,
        })
    }
}
