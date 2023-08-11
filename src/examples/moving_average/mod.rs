use crate::macros::*;

// needed for actor_outputs macro
pub use crate::compute::Context;
pub use crate::core::{Morph, OutboundChannel, OutboundHub};

// needed for actor_inputs macro
pub use crate::core::{
    ActorBuilder, InboundChannel, InboundHub, InboundMessage, InboundMessageNew, OnMessage,
    Value,
};

// needed for actor macro
pub use crate::core::{Actor, DefaultRunner, ActorFacade};

/// Outbound hub for the MovingAverage.
#[actor_outputs]
pub struct MovingAverageOutbound {
    /// Running average of computed by the actor.
    pub average: OutboundChannel<f64>,

    /// Cancel request, to request aborting compute pipeline execution.
    pub cancel_request: OutboundChannel<()>,
}

/// Properties of the MovingAverage actor.
#[derive(Clone, Debug)]
pub struct MovingAverageProp {
    /// Alpha value for the moving average with 0.0 < alpha < 1.0.
    pub alpha: f64,
    /// Time when cancel request is send out.
    pub timeout: f64,
}

impl Default for MovingAverageProp {
    fn default() -> Self {
        MovingAverageProp {
            alpha: 0.5,
            timeout: 10.0,
        }
    }
}

impl Value for MovingAverageProp {}



/// State of the MovingAverage actor.
#[derive(Clone, Debug, Default)]
pub struct MovingAverageState {
    /// current moving average
    pub moving_average: f64,
}

impl Value for MovingAverageState {}



/// Inbound message for the MovingAverage actor.
///
#[derive(Clone, Debug)]
#[actor_inputs(MovingAverageInbound, {MovingAverageProp, MovingAverageState, MovingAverageOutbound})]
pub enum MovingAverageMessage {
    /// a float value
    Value(f64),
}

impl OnMessage for MovingAverageMessage {
    /// Process the inbound time_stamp message.
    fn on_message(&self, prop: &Self::Prop, state: &mut Self::State, outbound: &Self::OutboundHub) {
        match &self {
            MovingAverageMessage::Value(new_value) => {
                state.moving_average =
                    (prop.alpha * new_value) + (1.0 - prop.alpha) * state.moving_average;

                outbound.average.send(state.moving_average);

                if new_value > &prop.timeout {
                    outbound.cancel_request.send(());
                }
            }
        }
    }
}

impl InboundMessageNew<f64> for MovingAverageMessage {
    fn new(_inbound_name: String, msg: f64) -> Self {
        MovingAverageMessage::Value(msg)
    }
}

/// The MovingAverage actor.
///
#[actor(MovingAverageMessage)]
type MovingAverage =
    Actor<MovingAverageProp, MovingAverageInbound, MovingAverageState, MovingAverageOutbound>;


/// Manual implementation of the moving average actor
pub mod manual;
