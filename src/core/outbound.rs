use crate::prelude::*;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tracing::warn;

/// IsOutboundHub is a collection of outbound channels for the actor.
pub trait IsOutboundHub: Send + Sync + 'static + HasActivate {
    /// Creates the IsOutboundHub from context and the actor name.
    fn from_context_and_parent(context: &mut Hollywood, actor_name: &str) -> Self;
}

/// An empty outbound hub - used for actors that do not have any outbound channels.
#[derive(Debug, Clone)]
pub struct NullOutbound {}

impl HasActivate for NullOutbound {
    fn extract(&mut self) -> Self {
        Self {}
    }

    fn activate(&mut self) {}
}

impl IsOutboundHub for NullOutbound {
    fn from_context_and_parent(_context: &mut Hollywood, _actor_name: &str) -> Self {
        Self {}
    }
}

/// OutboundChannel is a connections for messages which are sent to a downstream actor.
pub struct OutboundChannel<T> {
    /// Unique name of the outbound.
    pub name: String,
    /// Name of the actor that sends the outbound messages.
    pub actor_name: String,
    /// register
    pub connection_register: ConnectionEnum<T>,
}

impl<OutT: Clone + Send + Sync + std::fmt::Debug + 'static> OutboundChannel<OutT> {
    /// Create a new outbound for actor in provided context.    
    pub fn new(context: &mut Hollywood, name: String, actor_name: &str) -> Self {
        context.assert_unique_outbound_name(name.clone(), actor_name);

        Self {
            name: name.clone(),
            actor_name: actor_name.to_owned(),
            connection_register: ConnectionEnum::new(),
        }
    }

    /// Connect the outbound channel from this actor to the inbound channel of another actor.
    pub fn connect<M: IsInboundMessageNew<OutT>>(
        &mut self,
        ctx: &mut Hollywood,
        inbound: &mut InboundChannel<OutT, M>,
    ) {
        ctx.connect_impl(self, inbound);
        self.connection_register
            .push(Arc::new(OutboundConnection::<OutT, M> {
                sender: inbound.sender.clone(),
                inbound_channel: inbound.name.clone(),
                phantom: PhantomData,
            }));
    }

    /// Connect the outbound channel of type OutT to the inbound channel of another type InT.
    /// The user provided adapter function is used to convert from OutT to InT.
    pub fn connect_with_adapter<
        InT: Clone + Send + Sync + std::fmt::Debug + 'static,
        M: IsInboundMessageNew<InT>,
    >(
        &mut self,
        ctx: &mut Hollywood,
        adapter: fn(OutT) -> InT,
        inbound: &mut InboundChannel<InT, M>,
    ) {
        ctx.connect_impl(self, inbound);
        self.connection_register
            .push(Arc::new(OutboundConnectionWithAdapter::<OutT, InT, M> {
                sender: inbound.sender.clone(),
                inbound_channel: inbound.name.clone(),
                adapter,
            }));
    }

    /// Send a message to the connected inbound channels to other actors.
    pub fn send(&self, msg: OutT) {
        self.connection_register.send(msg);
    }
}

/// Outbound/request channel activation
pub trait HasActivate {
    /// Extract outbound/request channel and returns it.
    fn extract(&mut self) -> Self;

    /// Activates the outbound/request channel to be used.
    fn activate(&mut self);
}

impl<T> HasActivate for OutboundChannel<T> {
    fn activate(&mut self) {
        self.connection_register.activate();
    }

    fn extract(&mut self) -> Self {
        Self {
            name: self.name.clone(),
            actor_name: self.actor_name.clone(),
            connection_register: self.connection_register.extract(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OutboundConnection<Out, M: IsInboundMessage> {
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<M>,
    pub(crate) inbound_channel: String,
    pub(crate) phantom: std::marker::PhantomData<Out>,
}

#[derive(Clone)]
pub(crate) struct OutboundConnectionWithAdapter<Out, InT, M: IsInboundMessage> {
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<M>,
    pub(crate) inbound_channel: String,
    pub(crate) adapter: fn(Out) -> InT,
}

impl<Out, InT, M: IsInboundMessage> Debug for OutboundConnectionWithAdapter<Out, InT, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutboundConnection")
            .field("inbound_channel", &self.inbound_channel)
            .finish()
    }
}

/// Generic connection trait
pub trait IsGenericConnection<T>: Send + Sync {
    /// Send a message to the connected inbound channels to other actors.
    fn send_impl(&self, msg: T);
}

impl<Out: Send + Sync, M: IsInboundMessageNew<Out>> IsGenericConnection<Out>
    for OutboundConnection<Out, M>
{
    fn send_impl(&self, msg: Out) {
        let msg = M::new(self.inbound_channel.clone(), msg);
        let c = self.sender.clone();
        let handler = tokio::spawn(async move {
            match c.send(msg) {
                Ok(_) => {}
                Err(SendError(e)) => {
                    warn!("Send message error: {:?}", e);
                }
            }
        });
        std::mem::drop(handler);
    }
}

impl<Out: Send + Sync, InT, M: IsInboundMessageNew<InT>> IsGenericConnection<Out>
    for OutboundConnectionWithAdapter<Out, InT, M>
{
    fn send_impl(&self, msg: Out) {
        let msg = M::new(self.inbound_channel.clone(), (self.adapter)(msg));
        let c = self.sender.clone();
        let handler = tokio::spawn(async move {
            match c.send(msg) {
                Ok(_) => {}
                Err(SendError(e)) => {
                    warn!("Send message error: {:?}", e);
                }
            }
        });
        std::mem::drop(handler);
    }
}
