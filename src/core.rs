//! Core of hollywood actor framework.

/// Actor
pub mod actor;
pub use actor::Actor;
pub(crate) use actor::ActorNode;
pub use actor::FromPropState;

/// Actor builder
pub mod actor_builder;
pub use actor_builder::ActorBuilder;

/// Inbound
pub mod inbound;

pub use inbound::InboundChannel;
pub use inbound::InboundHub;
pub use inbound::InboundMessage;
pub use inbound::InboundMessageNew;
pub use inbound::NullInbound;
pub use inbound::NullMessage;
pub use inbound::OnMessage;

/// Outbound
pub mod outbound;
pub use outbound::Activate;
pub use outbound::NullOutbound;
pub use outbound::OutboundChannel;
pub(crate) use outbound::OutboundConnection;
pub use outbound::OutboundHub;

/// Request
pub mod request;
pub use request::NullRequest;
pub use request::RequestHub;

/// Connection
pub mod connection;

/// Run
pub mod runner;
pub use runner::DefaultRunner;

/// State
pub mod value;
pub use value::NullProp;
pub use value::NullState;
