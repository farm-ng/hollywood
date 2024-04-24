use crate::core::actor::ForwardRequestTable;
use crate::core::runner::Runner;
use crate::prelude::*;
use crate::ForwardTable;
use crate::GenericActor;

/// Creates actor from its components.
///
/// Used in  [`IsInboundHub::from_builder`] public interface.
pub struct ActorBuilder<
    'a,
    Prop,
    State,
    IsOutboundHub,
    OutRequest: IsOutRequestHub<M>,
    M: IsInboundMessage,
    R: IsInRequestMessage,
> {
    /// unique identifier of the actor
    pub actor_name: String,
    prop: Prop,
    state: State,
    /// execution context
    pub context: &'a mut Hollywood,
    /// a channel for sending messages to the actor
    pub sender: tokio::sync::mpsc::UnboundedSender<M>,
    pub(crate) receiver: tokio::sync::mpsc::UnboundedReceiver<M>,
    /// a channel for sending requests to the actor
    pub request_sender: tokio::sync::mpsc::UnboundedSender<R>,
    pub(crate) request_receiver: tokio::sync::mpsc::UnboundedReceiver<R>,
    /// a collection of inbound channels
    pub forward: ForwardTable<Prop, State, IsOutboundHub, OutRequest, M>,
    /// a collection of inbound channels
    pub forward_request: ForwardRequestTable<Prop, State, IsOutboundHub, OutRequest, R>,
}

impl<
        'a,
        Prop,
        State,
        Outbound: IsOutboundHub,
        OutRequest: IsOutRequestHub<M>,
        R: IsInRequestMessage,
        M: IsInboundMessage,
    > ActorBuilder<'a, Prop, State, Outbound, OutRequest, M, R>
{
    pub(crate) fn new(
        context: &'a mut Hollywood,
        actor_name: &str,
        prop: Prop,
        initial_state: State,
    ) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let (request_sender, request_receiver) = tokio::sync::mpsc::unbounded_channel();

        Self {
            actor_name: actor_name.to_owned(),
            prop,
            state: initial_state,
            context,
            sender: sender.clone(),
            receiver,
            request_sender,
            request_receiver,
            forward: ForwardTable::new(),
            forward_request: ForwardRequestTable::new(),
        }
    }

    pub(crate) fn build<
        Inbound: IsInboundHub<Prop, State, Outbound, OutRequest, M, R>,
        InRequest: IsInRequestHub<Prop, State, Outbound, OutRequest, M, R>,
        Run: Runner<Prop, Inbound, InRequest, State, Outbound, OutRequest, M, R>,
    >(
        self,
        inbound: Inbound,
        in_requests: InRequest,
        outbound: Outbound,
        out_requests: OutRequest,
    ) -> GenericActor<Prop, Inbound, InRequest, State, Outbound, OutRequest, Run> {
        let mut actor = GenericActor {
            actor_name: self.actor_name.clone(),
            inbound,
            in_requests,
            outbound,
            out_requests,
            phantom: std::marker::PhantomData {},
        };
        self.context.actors.push(Run::new_actor_node(
            self.actor_name,
            self.prop,
            self.state,
            (self.forward, self.receiver, actor.outbound.extract()),
            (
                self.forward_request,
                self.request_receiver,
                actor.out_requests.extract(),
            ),
        ));
        actor
    }
}
