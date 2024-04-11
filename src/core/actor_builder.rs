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
    Request: IsRequestHub<M>,
    M: IsInboundMessage,
> {
    /// unique identifier of the actor
    pub actor_name: String,
    prop: Prop,
    state: State,
    /// execution context
    pub context: &'a mut Hollywood,
    /// a channel for sending messages to the actor
    pub sender: tokio::sync::mpsc::Sender<M>,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<M>,
    /// a collection of inbound channels
    pub forward: ForwardTable<Prop, State, IsOutboundHub, Request, M>,
}

impl<'a, Prop, State, Outbound: IsOutboundHub, Request: IsRequestHub<M>, M: IsInboundMessage>
    ActorBuilder<'a, Prop, State, Outbound, Request, M>
{
    pub(crate) fn new(
        context: &'a mut Hollywood,
        actor_name: &str,
        prop: Prop,
        initial_state: State,
    ) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);

        Self {
            actor_name: actor_name.to_owned(),
            prop,
            state: initial_state,
            context,
            sender: sender.clone(),
            receiver,
            forward: ForwardTable::new(),
        }
    }

    pub(crate) fn build<
        Inbound: IsInboundHub<Prop, State, Outbound, Request, M>,
        Run: Runner<Prop, Inbound, State, Outbound, Request, M>,
    >(
        self,
        inbound: Inbound,
        outbound: Outbound,
        request: Request,
    ) -> GenericActor<Prop, Inbound, State, Outbound, Request, Run> {
        let mut actor = GenericActor {
            actor_name: self.actor_name.clone(),
            inbound,
            outbound,
            request,
            phantom: std::marker::PhantomData {},
        };
        self.context.actors.push(Run::new_actor_node(
            self.actor_name,
            self.prop,
            self.state,
            self.receiver,
            self.forward,
            actor.outbound.extract(),
            actor.request.extract(),
        ));
        actor
    }
}
