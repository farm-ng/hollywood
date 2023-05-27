use std::fmt::Debug;

use rand_distr::{Distribution, Normal};

use super::one_dim_robot::{RangeMeasurementModel, Robot};
use crate::compute::Context;
use crate::core::{
    Actor, ActorBuilder, DefaultRunner, DynActor, InboundChannel, InboundMessage,
    InboundMessageNew, InboundReceptionTrait, Morph, OnMessage, OutboundChannel,
    OutboundDistributionTrait, StateTrait,
};
use hollywood_macros::{actor, actor_inputs, actor_outputs};

/// A generic value with a timestamp.
#[derive(Clone, Debug, Default)]
pub struct Stamped<T: Clone + Debug> {
    /// Timestamp of the value.
    pub time: f64,
    /// The value.
    pub value: T,
}

impl<T: Clone + Debug> Stamped<T> {
    fn new(time: f64, value: &T) -> Self {
        Self {
            time,
            value: value.clone(),
        }
    }
}

/// Inbound channels for the simulation actor.
#[derive(Clone, Debug)]
#[actor_inputs(SimInbounds, {SimState, SimOutbounds})]
pub enum SimInboundMessage {
    /// Heartbeat message to drive the simulation.
    HeartBeat(f64),
}

/// Simulation for the one-dimensional Robot.
#[actor(SimInboundMessage)]
pub type SimActor = Actor<SimInbounds, SimState, SimOutbounds>;

impl OnMessage for SimInboundMessage {
    /// Invokes SimState::process_heart_beat on HeartBeat.
    fn on_message(&self, state: &mut Self::State, outbound: &Self::OutboundDistribution) {
        match self {
            SimInboundMessage::HeartBeat(time) => {
                state.process_heart_beat(*time, outbound);
                if time >= &10.0 {
                    outbound.cancel_request.send(());
                }
            }
        }
    }
}

impl InboundMessageNew<f64> for SimInboundMessage {
    fn new(_inbound_name: String, msg: f64) -> Self {
        SimInboundMessage::HeartBeat(msg)
    }
}

/// Simulation state
#[derive(Clone, Debug, Default)]
pub struct SimState {
    /// Current time.
    pub time: f64,
    /// True position of the robot.
    pub true_robot: Robot,
}

impl SimState {
    const RANGE_MODEL: RangeMeasurementModel = RangeMeasurementModel {};

    /// One step of the simulation.
    pub fn process_heart_beat(&mut self, time: f64, outbound: &SimOutbounds) {
        self.time = time;
        self.true_robot.position += self.true_robot.velocity * time;
        self.true_robot.velocity += self.true_robot.acceleration * time;
        self.true_robot.acceleration = 0.0; //0.1 * f64::sin(0.1 * time);

        let true_range = Self::RANGE_MODEL.range(self.true_robot.position, self.true_robot.heading);
        const RANGE_STD_DEV: f64 = RangeMeasurementModel::RANGE_STD_DEV;
        let range_normal = Normal::new(0.0, RANGE_STD_DEV).unwrap();
        let noisy_range = true_range + range_normal.sample(&mut rand::thread_rng());

        const VELOCITY_STD_DEV: f64 = 0.1;
        let noisy_velocity = self.true_robot.velocity
            + Normal::new(0.0, VELOCITY_STD_DEV)
                .unwrap()
                .sample(&mut rand::thread_rng());

        outbound
            .true_robot
            .send(Stamped::new(time, &self.true_robot));
        outbound.true_range.send(Stamped::new(time, &true_range));
        outbound.noisy_range.send(Stamped::new(time, &noisy_range));
        outbound
            .true_velocity
            .send(Stamped::new(time, &self.true_robot.velocity));
        outbound
            .noisy_velocity
            .send(Stamped::new(time, &noisy_velocity));

        println!(
            "time: {}, true range: {}, noisy range: {}, true velocity: {}, noisy velocity: {}",
            time, true_range, noisy_range, self.true_robot.velocity, noisy_velocity
        );

        println!(
            "time: {}, true: {} {}",
            time, self.true_robot.position, self.true_robot.velocity
        );
    }
}

impl StateTrait for SimState {}

/// OutboundChannel channels for the simulation actor.
#[actor_outputs]
pub struct SimOutbounds {
    /// True position of the robot.
    pub true_robot: OutboundChannel<Stamped<Robot>>,
    /// True range measurement - i.e. distance between the robot and the wall.
    pub true_range: OutboundChannel<Stamped<f64>>,
    /// True velocity of the robot.
    pub true_velocity: OutboundChannel<Stamped<f64>>,
    /// Noisy range measurement - i.e. measured distance to the wall.
    pub noisy_range: OutboundChannel<Stamped<f64>>,
    /// Noisy velocity measurement of the robot.
    pub noisy_velocity: OutboundChannel<Stamped<f64>>,
    /// Forward compute graph cancel request.
    pub cancel_request: OutboundChannel<()>,
}
