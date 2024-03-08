use std::fmt::Debug;
use std::fmt::Display;

/// A generic value with a timestamp.
#[derive(Clone, Debug, Default)]
pub struct Stamped<T: Clone + Debug> {
    /// Timestamp of the value.
    pub time: f64,
    /// Monotonic sequence counter
    pub seq: u64,
    /// The value.
    pub value: T,
}

impl<T: Clone + Debug> Stamped<T> {
    /// Creates a new value with a timestamp.
    pub fn from_stamp_counter_and_value(time: f64, seq: u64, value: &T) -> Self {
        Self {
            time,
            seq,
            value: value.clone(),
        }
    }
}

impl<T: Display + Clone + Debug> Display for Stamped<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{} {}", self.time, self.value)
    }
}

/// A robot in a one dimensional world.
#[derive(Clone, Debug, Default)]
pub struct Robot {
    /// Position of the robot.
    pub position: f64,
    /// Velocity of the robot.
    pub velocity: f64,
}

impl Display for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "position: {}, velocity: {})",
            self.position, self.velocity
        )
    }
}

/// A one dimensional world model.
pub struct WorldModel {}

impl WorldModel {
    /// Boundary of the world in the East.
    pub const BOUNDARY_EAST: f64 = 45.0;
}

/// A range measurement model.
pub struct RangeMeasurementModel {}

impl RangeMeasurementModel {
    /// Standard deviation of the range measurement.
    pub const RANGE_STD_DEV: f64 = 0.1;

    /// Range measurement generative model.
    pub fn range(&self, robot_position: f64) -> f64 {
        if robot_position > WorldModel::BOUNDARY_EAST {
            panic!("not implemented");
        } else {
            WorldModel::BOUNDARY_EAST - robot_position
        }
    }

    /// Derivative of the range measurement w.r.t. the robot position.
    pub fn dx_range(&self) -> f64 {
        -1.0
    }
}
