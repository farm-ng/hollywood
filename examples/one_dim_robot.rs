use hollywood::actors::PeriodicActor;
use hollywood::compute::Context;
use hollywood::core::{
    Actor, ActorBuilder, DefaultRunner, DynActor, InboundChannel, InboundMessage,
    InboundMessageNew, InboundReceptionTrait, NullOutbounds, NullState, OnMessage,
};
use hollywood::examples::one_dim_filter::{FilterActor, FilterState};
use hollywood::examples::one_dim_robot::{Robot, RobotHeading};
use hollywood::examples::one_dim_sim::{SimActor, SimState, Stamped};

use hollywood::macros::prelude::*;

use std::fmt::Debug;

#[derive(Clone, Debug)]
#[actor_inputs(PrintInbounds,{NullState, NullOutbounds})]
pub enum PrintInboundMessage {
    RobotState(Stamped<Robot>),
    RobotBelieve((String, FilterState)),
}

impl OnMessage for PrintInboundMessage {
    fn on_message(&self, _state: &mut Self::State, _outputs: &Self::OutboundDistribution) {
        match self {
            PrintInboundMessage::RobotState(robot) => {
                println!("true robot {:?} at {}", robot.value, robot.time);
            }
            PrintInboundMessage::RobotBelieve(robot) => {
                println!("{} robot out {:?}", robot.0, robot.1);
            }
        }
    }
}

impl InboundMessageNew<Stamped<Robot>> for PrintInboundMessage {
    fn new(_inbound_name: String, msg: Stamped<Robot>) -> Self {
        PrintInboundMessage::RobotState(msg)
    }
}

impl InboundMessageNew<(String, FilterState)> for PrintInboundMessage {
    fn new(inbound_name: String, msg: (String, FilterState)) -> Self {
        match inbound_name.as_str() {
            "RobotBelieve" => PrintInboundMessage::RobotBelieve(msg),
            _ => panic!("unexpected inbound name"),
        }
    }
}

#[actor(PrintInboundMessage)]
type PrintActor = Actor<PrintInbounds, NullState, NullOutbounds>;

async fn run_robot_example() {
    let pipeline = Context::configure(&mut |context| {
        let mut timer = PeriodicActor::new_with_period(context, 0.25);
        let mut sim = SimActor::new_with_state(
            context,
            SimState {
                time: 0.0,
                true_robot: Robot {
                    heading: RobotHeading::East,
                    position: 0.0,
                    velocity: 0.4,
                    acceleration: 0.0,
                },
            },
        );
        let mut filter = FilterActor::new_default_init_state(context);
        let mut printer = PrintActor::new_default_init_state(context);

        timer
            .outbound
            .heart_beat
            .connect(context, &mut sim.inbound.heart_beat);

        sim.outbound
            .true_velocity
            .connect(context, &mut filter.inbound.noisy_velocity);
        sim.outbound
            .noisy_range
            .connect(context, &mut filter.inbound.noisy_range);

        context.register_cancel_requester(&mut sim.outbound.cancel_request);

        filter
            .outbound
            .predicted_state
            .connect(context, &mut printer.inbound.robot_believe);
        filter
            .outbound
            .updated_state
            .connect(context, &mut printer.inbound.robot_believe);
        sim.outbound
            .true_robot
            .connect(context, &mut printer.inbound.robot_state);
    });

    let pipeline = pipeline.run().await;
    //println!("done");
    pipeline.run().await;
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_robot_example().await;
        })
}
