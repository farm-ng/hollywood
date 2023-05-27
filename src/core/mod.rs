#![deny(missing_docs)]
//! Core of hollywood actor framework.

/// Actor
pub mod actor;
pub use actor::Actor;
pub use actor::DynActor;
pub(crate) use actor::DynActiveActor;
pub (crate) use actor::DynDormantActor;

/// Actor builder
pub mod actor_builder;
pub use actor_builder::ActorBuilder;

/// Inbound
pub mod inbound;

pub use inbound::InboundReceptionTrait;
pub use inbound::NullInbounds;
pub use inbound::InboundChannel;
pub use inbound::InboundMessage;
pub use inbound::InboundMessageNew;
pub use inbound::NullMessage;
pub use inbound::OnMessage;

/// OutboundChannel
pub mod outbound;
pub use outbound::OutboundDistributionTrait;
pub use outbound::NullOutbounds;
pub use outbound::OutboundChannel;
pub use outbound::Morph;
pub(crate) use outbound::OutboundConnection;

/// Runner
pub mod runner;
pub use runner::DefaultRunner;

/// State
pub mod state;
pub use state::StateTrait;
pub use state::NullState;
