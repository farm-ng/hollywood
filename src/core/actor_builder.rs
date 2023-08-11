use std::collections::HashMap;

use crate::compute::context::Context;
use crate::core::{
    actor::GenericActor,
    inbound::{ForwardMessage, InboundHub, InboundMessage},
    outbound::OutboundHub,
    runner::Runner,
    value::Value,
};

/// Creates actor from its components.
/// 
/// Used in  [`InboundHub::from_builder`] public interface.
pub struct ActorBuilder<'a, Prop, State: Value, OutboundHub, M: InboundMessage> {
    /// unique identifier of the actor
    pub actor_name: String,
    prop: Prop,
    state: State,
    /// execution context
    pub  context: &'a mut Context,
    /// a channel for sending messages to the actor
    pub(crate) sender: tokio::sync::mpsc::Sender<M>,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<M>,
    /// a collection of inbound channels
    pub(crate) forward:
        HashMap<String, Box<dyn ForwardMessage<Prop, State, OutboundHub, M> + Send + Sync>>,
}

impl<'a, Prop, State: Value, Outbound: OutboundHub, M: InboundMessage>
    ActorBuilder<'a, Prop, State, Outbound, M>
{
    pub(crate) fn new(
        context: &'a mut Context,
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
            forward: HashMap::new(),
        }
    }

    pub(crate) fn build<
        Inbound: InboundHub<Prop, State, Outbound, M>,
        Run: Runner<Prop, Inbound, State, Outbound, M>,
    >(
        self,
        inbound: Inbound,
        outbound: Outbound,
    ) -> GenericActor<Prop, Inbound, State, Outbound, Run> {
        let mut actor = GenericActor {
            actor_name: self.actor_name.clone(),
            inbound,
            outbound,
            phantom: std::marker::PhantomData {},
        };
        let sleeping = Run::new_dormant_actor(
            self.actor_name,
            self.prop,
            self.state,
            self.receiver,
            self.forward,
            actor.outbound.extract(),
        );
        self.context.actors.push(sleeping);
        actor
    }
}
