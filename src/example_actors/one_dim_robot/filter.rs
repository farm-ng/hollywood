use std::fmt::{Debug, Display};

use crate::compute::Context;
use crate::core::request::{NullRequest, RequestMessage};
use crate::core::{
    Activate, Actor, ActorBuilder, DefaultRunner, FromPropState, InboundChannel, InboundHub,
    InboundMessage, InboundMessageNew, NullProp, OnMessage, OutboundChannel, OutboundHub,
};
use crate::example_actors::one_dim_robot::{RangeMeasurementModel, Stamped};
use hollywood_macros::{actor, actor_inputs, actor_outputs};

use super::sim::PingPong;

/// Inbound channels for the filter actor.
#[derive(Clone, Debug)]
#[actor_inputs(FilterInbound,{NullProp, FilterState, FilterOutbound, NullRequest})]
pub enum FilterInboundMessage {
    /// noisy velocity measurements
    NoisyVelocity(Stamped<f64>),
    /// noisy range measurements
    NoisyRange(Stamped<f64>),
    /// Request
    PingPongRequest(RequestMessage<f64, PingPong>),
}

#[actor(FilterInboundMessage)]
type Filter = Actor<NullProp, FilterInbound, FilterState, FilterOutbound, NullRequest>;

impl OnMessage for FilterInboundMessage {
    /// Process the inbound message NoisyVelocity or NoisyRange.
    ///
    /// On NoisyVelocity, FilterState::prediction is called.
    /// On NoisyRange, FilterState::update is called.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        _request: &Self::RequestHub,
    ) {
        match self {
            FilterInboundMessage::NoisyVelocity(v) => {
                state.prediction(&v, outbound);
            }
            FilterInboundMessage::NoisyRange(r) => {
                state.update(&r, outbound);
            }
            FilterInboundMessage::PingPongRequest(request) => {
                request.reply(|ping| PingPong {
                    ping,
                    pong: state.time,
                });
            }
        }
    }
}

impl InboundMessageNew<Stamped<f64>> for FilterInboundMessage {
    fn new(inbound_channel: String, msg: Stamped<f64>) -> Self {
        if inbound_channel == "NoisyRange" {
            FilterInboundMessage::NoisyRange(msg)
        } else {
            FilterInboundMessage::NoisyVelocity(msg)
        }
    }
}

impl InboundMessageNew<RequestMessage<f64, PingPong>> for FilterInboundMessage {
    fn new(_inbound_channel: String, request: RequestMessage<f64, PingPong>) -> Self {
        FilterInboundMessage::PingPongRequest(request)
    }
}

/// Filter state
#[derive(Clone, Debug, Default)]
pub struct FilterState {
    /// time of the last prediction or update
    pub time: f64,
    /// Monotonically increasing sequence number
    pub seq: u64,
    /// belief about the robot's position
    pub pos_vel_acc: PositionBelieve,
}

impl Display for FilterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(time: {}, robot_position: {})",
            self.time, self.pos_vel_acc.mean
        )
    }
}

/// Named filter state
#[derive(Clone, Debug, Default)]
pub struct NamedFilterState {
    /// name
    pub name: String,
    /// filter state
    pub state: FilterState,
}

impl NamedFilterState {
    /// Create a new NamedFilterState
    pub fn new(name: String, state: FilterState) -> Self {
        Self { name, state }
    }
}

impl Display for NamedFilterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.state)
    }
}

/// Belief about the robot's position
#[derive(Clone, Debug)]
pub struct PositionBelieve {
    /// mean of the position believe
    pub mean: nalgebra::Vector3<f64>,
    /// covariance of the position believe
    pub covariance: nalgebra::Matrix3<f64>,
}

impl Default for PositionBelieve {
    fn default() -> Self {
        Self {
            mean: nalgebra::Vector3::new(0.0, 0.0, 0.0),
            covariance: nalgebra::Matrix3::new(
                100.0, 0.0, 0.0, //
                0.0, 100.0, 0.0, //
                0.0, 0.0, 100.0,
            ),
        }
    }
}

impl FilterState {
    const RANGE_MODEL: RangeMeasurementModel = RangeMeasurementModel {};

    /// Prediction step of the filter
    ///
    /// Predicts the new robot's position based the previous position and the velocity measurement.
    pub fn prediction(&mut self, noisy_velocity: &Stamped<f64>, outbound: &FilterOutbound) {
        let dt = noisy_velocity.time - self.time;
        self.time = noisy_velocity.time;

        // 1. Random-walk acceleration motion model
        self.pos_vel_acc.mean[0] +=
            self.pos_vel_acc.mean[1] * dt + 0.5 * self.pos_vel_acc.mean[2] * dt * dt;
        self.pos_vel_acc.mean[1] += self.pos_vel_acc.mean[2] * dt;
        let f = nalgebra::Matrix3::new(1.0, dt, 0.5 * dt * dt, 0.0, 1.0, dt, 0.0, 0.0, 1.0);
        let acceleration_noise_variance = 0.5;
        let q = nalgebra::Matrix3::new(
            0.25 * dt.powi(4),
            0.5 * dt.powi(3),
            0.5 * dt.powi(2) * acceleration_noise_variance,
            0.5 * dt.powi(3),
            dt.powi(2),
            dt * acceleration_noise_variance,
            0.5 * dt.powi(2) * acceleration_noise_variance,
            dt * acceleration_noise_variance,
            acceleration_noise_variance,
        );
        self.pos_vel_acc.covariance = f * self.pos_vel_acc.covariance * f.transpose() + q;

        // 2. Update velocity based on the velocity measurement
        // (strictly speaking this is an update, not a prediction)
        let h_velocity = nalgebra::Matrix1x3::new(0.0, 1.0, 0.0);
        let predicted_velocity = self.pos_vel_acc.mean[1];
        let innovation_velocity = noisy_velocity.value - predicted_velocity;
        const VELOCITY_MEASUREMENT_NOISE: f64 = 0.1;
        let r_velocity =
            nalgebra::Matrix1::new(VELOCITY_MEASUREMENT_NOISE * VELOCITY_MEASUREMENT_NOISE);
        let kalman_gain_velocity = self.pos_vel_acc.covariance
            * h_velocity.transpose()
            * (h_velocity * self.pos_vel_acc.covariance * h_velocity.transpose() + r_velocity)
                .try_inverse()
                .unwrap();
        self.pos_vel_acc.mean += kalman_gain_velocity * innovation_velocity;
        let identity = nalgebra::Matrix3::identity();
        self.pos_vel_acc.covariance =
            (identity - kalman_gain_velocity * h_velocity) * self.pos_vel_acc.covariance;

        outbound.predicted_state.send(NamedFilterState::new(
            "Predicted: ".to_owned(),
            self.clone(),
        ));
    }

    /// Update step of the filter
    ///
    /// Updates the robot's position based on the range measurement.
    pub fn update(&mut self, noisy_range: &Stamped<f64>, outbound: &FilterOutbound) {
        // Update position based on the range measurement
        let h = nalgebra::Matrix1x3::new(1.0, 0.0, 0.0);
        let predicted_range = Self::RANGE_MODEL.range(self.pos_vel_acc.mean[0]);
        let innovation = predicted_range - noisy_range.value;
        const RANGE_STD_DEV: f64 = RangeMeasurementModel::RANGE_STD_DEV;
        let r = nalgebra::Matrix1::new(RANGE_STD_DEV * RANGE_STD_DEV);
        let kalman_gain = self.pos_vel_acc.covariance
            * h.transpose()
            * (h * self.pos_vel_acc.covariance * h.transpose() + r)
                .try_inverse()
                .unwrap();
        self.pos_vel_acc.mean += kalman_gain * innovation;
        let identity = nalgebra::Matrix3::identity();
        self.pos_vel_acc.covariance = (identity - kalman_gain * h) * self.pos_vel_acc.covariance;
        self.seq += 1;
        outbound
            .updated_state
            .send(NamedFilterState::new("Updated: ".to_owned(), self.clone()));
    }
}

/// OutboundChannel channels for the filter actor.
#[actor_outputs]
pub struct FilterOutbound {
    /// Publishes the predicted state of the filter.
    pub predicted_state: OutboundChannel<NamedFilterState>,
    /// Publishes the updated state of the filter.
    pub updated_state: OutboundChannel<NamedFilterState>,
}
