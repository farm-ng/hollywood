/// One dimensional world and sensor model
pub mod model;
pub use model::{RangeMeasurementModel, Robot, Stamped};

/// Simulation actor for the robot in the one dimensional world.
pub mod sim;
pub use sim::{Sim, SimState};

/// Kalman filter actor for the one dimensional robot.
pub mod filter;
pub use filter::{Filter, NamedFilterState};

/// Drawing actor for the one dimensional robot.
/// 
/// Draws "ascii art" of the robot and the filter state to the console.
/// 
/// ```text
/// time:2.25
///                         ⡏⢹⢀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⡀
///                   ⡆     ⡇⢸     ⢰
///                   ⠇     ⠃⠘     ⠸
/// ```
pub mod draw;
pub use draw::DrawActor;
