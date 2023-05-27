#![deny(missing_docs)]
//! Hollywood is an actor framework for Rust.

/// The core framework concepts such as actors, state, inbound, outbound and runners.
pub mod core;

/// The compute context and compute graph.
pub mod compute;

/// Library of actors.
pub mod actors;

/// Library of actors.
pub mod examples;

/// Convenience macros for hollywood from the hollywood_macros crate.
pub mod macros {
    
    /// All macros from the hollywood_macros crate.
    pub mod prelude {
        pub use hollywood_macros::*;
    }
}
