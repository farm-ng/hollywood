use std::fmt::Debug;

use super::{one_dim_robot::RangeMeasurementModel, one_dim_sim::Stamped};
use crate::compute::Context;
use crate::{
    core::{
        Actor, ActorBuilder, DefaultRunner, DynActor, InboundChannel, InboundMessage,
        InboundMessageNew, InboundReceptionTrait, Morph, OnMessage, OutboundChannel,
        OutboundDistributionTrait, StateTrait,
    },
    examples::one_dim_robot::RobotHeading,
};
use hollywood_macros::{actor, actor_inputs, actor_outputs};

/// Inbound channels for the filter actor.
#[derive(Clone, Debug)]
#[actor_inputs(FilterInbounds,{FilterState,FilterOutbounds})]
pub enum FilterInboundMessage {
    /// noisy velocity measurements
    NoisyVelocity(Stamped<f64>),
    /// noisy range measurements
    NoisyRange(Stamped<f64>),
}

#[actor(FilterInboundMessage)]
type FilterActor = Actor<FilterInbounds, FilterState, FilterOutbounds>;

impl OnMessage for FilterInboundMessage {
    /// Process the inbound message NoisyVelocity or NoisyRange.
    ///
    /// On NoisyVelocity, FilterState::prediction is called.
    /// On NoisyRange, FilterState::update is called.
    fn on_message(&self, state: &mut Self::State, outbound: &Self::OutboundDistribution) {
        match &self {
            FilterInboundMessage::NoisyVelocity(v) => {
                state.prediction(v, outbound);
            }
            FilterInboundMessage::NoisyRange(r) => {
                state.update(r, outbound);
            }
        }
    }
}

impl InboundMessageNew<Stamped<f64>> for FilterInboundMessage {
    fn new(inbound_name: String, msg: Stamped<f64>) -> Self {
        if inbound_name == "NoisyRange" {
            FilterInboundMessage::NoisyRange(msg)
        } else {
            FilterInboundMessage::NoisyVelocity(msg)
        }
    }
}

/// Filter state
#[derive(Clone, Debug, Default)]
pub struct FilterState {
    /// time of the last prediction or update
    pub time: f64,
    /// belief about the robot's position
    pub robot_position: PositionBelieve,
}

/// Belief about the robot's position
#[derive(Clone, Debug)]
pub struct PositionBelieve {
    /// mean of the position believe
    pub mean: f64,
    /// covariance of the position believe
    pub covariance: f64,
}

impl Default for PositionBelieve {
    fn default() -> Self {
        Self {
            mean: 0.0,
            covariance: 100.0,
        }
    }
}

impl FilterState {
    const RANGE_MODEL: RangeMeasurementModel = RangeMeasurementModel {};

    /// Prediction step of the filter
    ///
    /// Predicts the new robot's position based the previous position and the velocity measurement.
    pub fn prediction(&mut self, noisy_velocity: &Stamped<f64>, outbound: &FilterOutbounds) {
        let dt = noisy_velocity.time - self.time;
        self.time = noisy_velocity.time;

        self.robot_position.mean += noisy_velocity.value * dt;
        const VELOCITY_STD_DEV: f64 = 0.3;
        self.robot_position.covariance += VELOCITY_STD_DEV * VELOCITY_STD_DEV * dt;
        outbound
            .predicted_state
            .send(("Predicted".to_owned(), self.clone()));
    }

    /// Update step of the filter
    ///
    /// Updates the robot's position based on the range measurement.
    pub fn update(&mut self, noisy_range: &Stamped<f64>, outbound: &FilterOutbounds) {
        let predicted_range = Self::RANGE_MODEL.range(self.robot_position.mean, RobotHeading::East);
        const RANGE_STD_DEV: f64 = RangeMeasurementModel::RANGE_STD_DEV;
        let mat_h = -1.0; // Jacobian of the measurement function

        // Adjust innovation calculation using the Jacobian
        let innovation = noisy_range.value - predicted_range;
        let innovation_covariance = self.robot_position.covariance + RANGE_STD_DEV * RANGE_STD_DEV;
        let kalman_gain = mat_h * self.robot_position.covariance / innovation_covariance;
        self.robot_position.mean += kalman_gain * innovation;
        self.robot_position.covariance *= 1.0 - kalman_gain * mat_h;

        outbound
            .updated_state
            .send(("Updated".to_owned(), self.clone()));
    }
}

impl StateTrait for FilterState {}

/// OutboundChannel channels for the filter actor.
#[actor_outputs]
pub struct FilterOutbounds {
    /// Publishes the predicted state of the filter.
    pub predicted_state: OutboundChannel<(String, FilterState)>,
    /// Publishes the updated state of the filter.
    pub updated_state: OutboundChannel<(String, FilterState)>,
}
