use crate::core::actor::ActorNodeImpl;
use crate::prelude::*;

/// Runner executes the pipeline.
pub trait IsRunner<
    Prop,
    Inbound: IsInboundHub<Prop, State, Outbound, OutRequest, M, R>,
    InRequest,
    State,
    Outbound: IsOutboundHub,
    OutRequest: IsOutRequestHub<M>,
    M: IsInboundMessage,
    R: IsInRequestMessage,
>
{
    /// Create a new actor to be stored by the context.
    fn new_actor_node(
        name: String,
        prop: Prop,
        init_state: State,
        forward_receiver_outbound: (
            ForwardTable<Prop, State, Outbound, OutRequest, M>,
            tokio::sync::mpsc::UnboundedReceiver<M>,
            Outbound,
        ),
        forward_receiver_request: (
            ForwardRequestTable<Prop, State, Outbound, OutRequest, R>,
            tokio::sync::mpsc::UnboundedReceiver<R>,
            OutRequest,
        ),
        on_exit_fn: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
    ) -> Box<dyn IsActorNode + Send + Sync>;
}

/// The default runner.
pub struct DefaultRunner<
    Prop,
    Inbound: Send + Sync,
    InRequest,
    State,
    Outbound: Send + Sync + 'static,
    Request: Send + Sync + 'static,
> {
    phantom: std::marker::PhantomData<(Prop, Inbound, InRequest, State, Outbound, Request)>,
}

impl<
        Prop,
        State,
        Inbound: Send + Sync,
        InRequest,
        Outbound: Send + Sync + 'static,
        Request: Send + Sync + 'static,
    > Default for DefaultRunner<Prop, Inbound, InRequest, State, Outbound, Request>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Prop,
        State,
        Inbound: Send + Sync,
        InRequest,
        Outbound: Send + Sync + 'static,
        Request: Send + Sync + 'static,
    > DefaultRunner<Prop, Inbound, InRequest, State, Outbound, Request>
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
        Inbound: IsInboundHub<Prop, State, Outbound, OutRequest, M, R>,
        InRequest,
        State: std::marker::Send + std::marker::Sync + 'static,
        Outbound: IsOutboundHub,
        R: IsInRequestMessage,
        M: IsInboundMessage,
        OutRequest: IsOutRequestHub<M>,
    > IsRunner<Prop, Inbound, InRequest, State, Outbound, OutRequest, M, R>
    for DefaultRunner<Prop, Inbound, InRequest, State, Outbound, OutRequest>
{
    fn new_actor_node(
        name: String,
        prop: Prop,
        init_state: State,
        forward_receiver_outbound: (
            ForwardTable<Prop, State, Outbound, OutRequest, M>,
            tokio::sync::mpsc::UnboundedReceiver<M>,
            Outbound,
        ),
        forward_receiver_request: (
            ForwardRequestTable<Prop, State, Outbound, OutRequest, R>,
            tokio::sync::mpsc::UnboundedReceiver<R>,
            OutRequest,
        ),
        on_exit_fn: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
    ) -> Box<dyn IsActorNode + Send + Sync> {
        Box::new(ActorNodeImpl::<Prop, State, Outbound, OutRequest, M, R> {
            name,
            prop,
            state: Some(init_state),
            forward: forward_receiver_outbound.0,
            receiver: Some(forward_receiver_outbound.1),
            outbound: forward_receiver_outbound.2,
            forward_request: forward_receiver_request.0,
            request_receiver: Some(forward_receiver_request.1),
            out_request: forward_receiver_request.2,
            on_exit_fn,
        })
    }
}
