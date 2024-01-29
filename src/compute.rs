/// The compute context.
pub mod context;
pub use context::Context;

/// The compute graph of actors.
pub mod pipeline;
pub(crate) use pipeline::CancelRequest;
pub use pipeline::Pipeline;

/// The graph topology.
pub mod topology;
pub(crate) use topology::ActorNode;
pub(crate) use topology::Topology;
