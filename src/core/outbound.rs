use std::{marker::PhantomData, sync::Arc, vec};
use tokio::sync::mpsc::error::SendError;

use crate::compute::context::Context;
use crate::core::inbound::{InboundChannel, InboundMessage, InboundMessageNew};

/// OutboundHub is a collection of outbound channels for the actor.
pub trait OutboundHub: Send + Sync + 'static + Morph {
    /// Creates the OutboundHub from context and the actor name.
    fn from_context_and_parent(context: &mut Context, actor_name: &str) -> Self;
}

/// An empty outbound hub - used for actors that do not have any outbound channels.
#[derive(Debug, Clone)]
pub struct NullOutbound {
}

impl Morph for NullOutbound {
    fn extract(&mut self) -> Self {
        Self {}
    }

    fn activate(&mut self) {}
}

impl OutboundHub for NullOutbound {
    fn from_context_and_parent(_context: &mut Context, _actor_name: &str) -> Self {
        Self {}
    }
}

/// OutboundChannel is a connections for messages which are sent to a downstream actor.
pub struct OutboundChannel<T> {
    /// Unique name of the outbound.
    pub name: String,
    /// Name of the actor that sends the outbound messages.
    pub actor_name: String,
    pub(crate) connection_register: ConnectionEnum<T>,
}

impl<T: Default + Clone + Send + Sync + std::fmt::Debug + 'static> OutboundChannel<T> {
    /// Create a new outbound for actor in provided context.    
    pub fn new(context: &mut Context, name: String, actor_name: &str) -> Self {
        context.assert_unique_outbound_name(name.clone(), actor_name);

        Self {
            name: name.clone(),
            actor_name: actor_name.to_owned(),
            connection_register: ConnectionEnum::new(),
        }
    }

    /// Connect the outbound channel from this actor to the inbound channel of another actor.
    pub fn connect<M: InboundMessageNew<T>>(
        &mut self,
        ctx: &mut Context,
        inbound: &mut InboundChannel<T, M>,
    ) {
        ctx.connect_impl(self, inbound);
        self.connection_register.push(Arc::new(OutboundConnection {
            sender: inbound.sender.clone(),
            inbound_channel: inbound.name.clone(),
            phantom: PhantomData {},
        }));
    }

    /// Send a message to the connected inbound channels to other actors.
    pub fn send(&self, msg: T) {
        self.connection_register.send(msg);
    }
}

/// Trait for morphing state of an outbound channel.
pub trait Morph {
    /// Extract outbound channel and returns it.
    fn extract(&mut self) -> Self;

    /// Activates the outbound channel to be used.
    fn activate(&mut self);
}

impl<T> Morph for OutboundChannel<T> {
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

#[derive(Debug, Clone)]
pub(crate) struct OutboundConnection<T, M: InboundMessage> {
    pub(crate) sender: tokio::sync::mpsc::Sender<M>,
    pub(crate) inbound_channel: String,
    pub(crate) phantom: PhantomData<T>,
}

pub(crate) trait GenericConnection<T>: Send + Sync {
    fn send_impl(&self, msg: T);
}

impl<T: Send + Sync, M: InboundMessageNew<T>> GenericConnection<T> for OutboundConnection<T, M> {
    fn send_impl(&self, msg: T) {
        let msg = M::new(self.inbound_channel.clone(), msg);
        let c = self.sender.clone();
        let handler = tokio::spawn(async move {
            match c.send(msg).await {
                Ok(_) => {}
                Err(SendError(_)) => {
                    println!("SendError");
                }
            }
        });
        std::mem::drop(handler);
    }
}

type ConnectionRegister<T> = Vec<Arc<dyn GenericConnection<T> + Send + Sync>>;

pub(crate) struct ConnectionConfig<T> {
    pub connection_register: ConnectionRegister<T>,
    pub maybe_register_launch_pad: Option<tokio::sync::oneshot::Sender<ConnectionRegister<T>>>,
    pub maybe_register_landing_pad: Option<tokio::sync::oneshot::Receiver<ConnectionRegister<T>>>,
}

impl<T> Drop for ConnectionConfig<T> {
    fn drop(&mut self) {
        if let Some(connection_launch_pad) = self.maybe_register_launch_pad.take() {
            let connection_register = std::mem::take(&mut self.connection_register);
            let _ = connection_launch_pad.send(connection_register);
        } else {
            panic!("ConnectionConfig dropped when launch pad is is empty");
        }
    }
}

impl<T> ConnectionConfig<T> {
    pub fn new() -> Self {
        let (connection_launch_pad, connection_landing_pad) = tokio::sync::oneshot::channel();
        Self {
            connection_register: vec![],
            maybe_register_launch_pad: Some(connection_launch_pad),
            maybe_register_landing_pad: Some(connection_landing_pad),
        }
    }
}

pub(crate) struct ActiveConnection<T> {
    pub maybe_registers: Option<ConnectionRegister<T>>,
    pub maybe_register_landing_pad: Option<tokio::sync::oneshot::Receiver<ConnectionRegister<T>>>,
}

pub(crate) enum ConnectionEnum<T> {
    Config(ConnectionConfig<T>),
    Active(ActiveConnection<T>),
}

impl<T: Default + Clone + Send + Sync + std::fmt::Debug + 'static> ConnectionEnum<T> {
    pub fn new() -> Self {
        Self::Config(ConnectionConfig::new())
    }

    pub fn push(&mut self, connection: Arc<dyn GenericConnection<T> + Send + Sync>) {
        match self {
            Self::Config(config) => {
                config.connection_register.push(connection);
            }
            Self::Active(_) => {
                panic!("Cannot push to active connection");
            }
        }
    }

    fn send(&self, msg: T) {
        match self {
            Self::Config(_) => {
                panic!("Cannot send to config connection");
            }
            Self::Active(active) => {
                for i in active.maybe_registers.as_ref().unwrap().iter() {
                    i.send_impl(msg.clone());
                }
            }
        }
    }
}

impl<T> Morph for ConnectionEnum<T> {
    fn extract(&mut self) -> Self {

        println!("ConnectionEnum::extract");
        match self {
            Self::Config(config) => Self::Active(ActiveConnection {
                maybe_registers: None,
                maybe_register_landing_pad: Some(config.maybe_register_landing_pad.take().unwrap()),
            }),
            Self::Active(_) => {
                panic!("Cannot extract active connection");
            }
        }
    }

    fn activate(&mut self) {
        println!("ConnectionEnum::activate");
        match self {
            Self::Config(_) => {
                panic!("Cannot activate config connection");
            }
            Self::Active(active) => {
                let connection_register = active
                    .maybe_register_landing_pad
                    .take()
                    .unwrap()
                    .try_recv()
                    .unwrap();
                active.maybe_registers = Some(connection_register);
            }
        }
    }
}
