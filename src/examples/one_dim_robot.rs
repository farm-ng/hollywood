use std::fmt::Debug;

/// Heading of the one dimensional robot. Either East or West.
#[derive(Clone, Debug, Default, Copy)]
pub enum RobotHeading {
    /// Heading East.
    #[default]
    East,
    /// Heading West.
    West,
}

/// A robot in a one dimensional world.
#[derive(Clone, Debug, Default)]
pub struct Robot {
    /// Heading of the robot.
    pub heading: RobotHeading,
    /// Position of the robot.
    pub position: f64,
    /// Velocity of the robot.
    pub velocity: f64,
    /// Acceleration of the robot.
    pub acceleration: f64,
}

/// A one dimensional world model.
pub struct WorldModel {}

impl WorldModel {
    /// Boundary of the world in the East.
    pub const BOUNDARY_EAST: f64 = 100.0;
}

/// A range measurement model.
pub struct RangeMeasurementModel {}

impl RangeMeasurementModel {
    /// Standard deviation of the range measurement.
    pub const RANGE_STD_DEV: f64 = 0.1;

    /// Range measurement generative model.
    pub fn range(&self, robot_position: f64, heading: RobotHeading) -> f64 {
        match heading {
            RobotHeading::East => {
                if robot_position > WorldModel::BOUNDARY_EAST {
                    panic!("not implemented");
                } else {
                    WorldModel::BOUNDARY_EAST - robot_position
                }
            }
            RobotHeading::West => {
                panic!("not implemented");
            }
        }
    }
}
