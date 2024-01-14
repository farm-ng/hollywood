//! Core of hollywood actor framework.

/// Actor
pub mod actor;
pub use actor::{Actor, FromPropState};
pub(crate) use actor::ActorNode;

/// Actor builder
pub mod actor_builder;
pub use actor_builder::ActorBuilder;

/// Inbound
pub mod inbound;

pub use inbound::{
    InboundChannel, InboundHub, InboundMessage, InboundMessageNew, NullInbound, NullMessage,
    OnMessage,
};

/// Outbound 
pub mod outbound;
pub(crate) use outbound::OutboundConnection;
pub use outbound::{Morph, NullOutbound, OutboundChannel, OutboundHub};

/// Request
pub mod request;

/// Connection
pub mod connection;

/// Run
pub mod runner;
pub use runner::DefaultRunner;

/// State
pub mod value;
pub use value::{NullProp, NullState};
