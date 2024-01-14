use std::fmt::Debug;

use rand_distr::{Distribution, Normal};

use crate::compute::Context;
use crate::core::request::{ReplyMessage, RequestChannel, RequestHub};
use crate::core::{
    Actor, ActorBuilder, DefaultRunner, FromPropState, InboundChannel, InboundHub, InboundMessage,
    InboundMessageNew, Morph, NullProp, OnMessage, OutboundChannel, OutboundHub,
};
use crate::example_actors::one_dim_robot::{RangeMeasurementModel, Robot, Stamped};
use crate::macros::*;

#[derive(Clone, Debug, Default)]
pub struct PingPong {
    pub ping: f64,
    pub pong: f64,
}

/// Inbound channels for the simulation actor.
#[derive(Clone, Debug)]
#[actor_inputs(SimInbound, {NullProp, SimState, SimOutbound, SimRequest})]
pub enum SimInboundMessage {
    /// Time-stamp message to drive the simulation.
    TimeStamp(f64),
    /// Reply message from the compute pipeline.
    PingPongReply(ReplyMessage<PingPong>),
}

/// Simulation for the one-dimensional Robot.
#[actor(SimInboundMessage)]
pub type Sim = Actor<NullProp, SimInbound, SimState, SimOutbound, SimRequest>;

impl OnMessage for SimInboundMessage {
    /// Invokes [SimState::process_time_stamp()] on TimeStamp.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        request: &Self::RequestHub,
    ) {
        match self {
            SimInboundMessage::TimeStamp(time) => {
                state.process_time_stamp(time, outbound, request);
                if time >= state.shutdown_time {
                    outbound.cancel_request.send(());
                }
            }
            SimInboundMessage::PingPongReply(msg) => {
                println!("ping: {}, pong: {}", msg.reply.ping, msg.reply.pong);
            }
        }
    }
}

impl InboundMessageNew<f64> for SimInboundMessage {
    fn new(_inbound_name: String, msg: f64) -> Self {
        SimInboundMessage::TimeStamp(msg)
    }
}

impl InboundMessageNew<ReplyMessage<PingPong>> for SimInboundMessage {
    fn new(_inbound_name: String, msg: ReplyMessage<PingPong>) -> Self {
        SimInboundMessage::PingPongReply(msg)
    }
}

/// Simulation state
#[derive(Clone, Debug, Default)]
pub struct SimState {
    /// Time at which the simulation will be shut down.
    pub shutdown_time: f64,
    /// Current time.
    pub time: f64,
    /// True position and velocity of the robot.
    pub true_robot: Robot,
}

impl SimState {
    const RANGE_MODEL: RangeMeasurementModel = RangeMeasurementModel {};

    /// One step of the simulation.
    pub fn process_time_stamp(&mut self, time: f64, outbound: &SimOutbound, request: &SimRequest) {
        self.time = time;
        self.true_robot.position += self.true_robot.velocity * time;
        self.true_robot.velocity = 0.25 * (0.25 * time).cos();

        let true_range = Self::RANGE_MODEL.range(self.true_robot.position);
        const RANGE_STD_DEV: f64 = RangeMeasurementModel::RANGE_STD_DEV;
        let range_normal = Normal::new(0.0, RANGE_STD_DEV).unwrap();
        let noisy_range = true_range + range_normal.sample(&mut rand::thread_rng());

        const VELOCITY_STD_DEV: f64 = 0.01;
        let noisy_velocity = self.true_robot.velocity
            + Normal::new(0.0, VELOCITY_STD_DEV)
                .unwrap()
                .sample(&mut rand::thread_rng());

        outbound
            .true_robot
            .send(Stamped::from_stamp_and_value(time, &self.true_robot));
        outbound
            .true_range
            .send(Stamped::from_stamp_and_value(time, &true_range));
        outbound
            .noisy_range
            .send(Stamped::from_stamp_and_value(time, &noisy_range));
        outbound.true_velocity.send(Stamped::from_stamp_and_value(
            time,
            &self.true_robot.velocity,
        ));
        outbound
            .noisy_velocity
            .send(Stamped::from_stamp_and_value(time, &noisy_velocity));

        if time == 5.0 {
            request.ping_pong.send_request(time);
        }
    }
}

/// OutboundChannel channels for the simulation actor.
#[actor_outputs]
pub struct SimOutbound {
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
    /// Compute pipeline cancel request.
    pub cancel_request: OutboundChannel<()>,
}

/// Request of the simulation actor.
pub struct SimRequest {
    /// Check time-stamp of receiver
    pub ping_pong: RequestChannel<f64, PingPong, SimInboundMessage>,
}

impl RequestHub<SimInboundMessage> for SimRequest {
    fn from_context_and_parent(
        actor_name: &str,
        sender: &tokio::sync::mpsc::Sender<SimInboundMessage>,
    ) -> Self {
        Self {
            ping_pong: RequestChannel::new(actor_name.to_owned(), "ping_pong", sender),
        }
    }
}

impl Morph for SimRequest {
    fn extract(&mut self) -> Self {
        Self {
            ping_pong: self.ping_pong.extract(),
        }
    }

    fn activate(&mut self) {
        self.ping_pong.activate();
    }
}
