use crate::macros::*;

// needed for actor_outputs macro
pub use crate::compute::Context;
use crate::core::request::NullRequest;
pub use crate::core::Activate;
pub use crate::core::OutboundChannel;
pub use crate::core::OutboundHub;

// needed for actor_inputs macro
pub use crate::core::ActorBuilder;
pub use crate::core::InboundChannel;
pub use crate::core::InboundHub;
pub use crate::core::InboundMessage;
pub use crate::core::InboundMessageNew;
pub use crate::core::OnMessage;

// needed for actor macro
pub use crate::core::Actor;
pub use crate::core::DefaultRunner;
pub use crate::core::FromPropState;

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

/// State of the MovingAverage actor.
#[derive(Clone, Debug, Default)]
pub struct MovingAverageState {
    /// current moving average
    pub moving_average: f64,
}

/// Inbound message for the MovingAverage actor.
///
#[derive(Clone, Debug)]
#[actor_inputs(MovingAverageInbound, {MovingAverageProp, MovingAverageState, MovingAverageOutbound, NullRequest})]
pub enum MovingAverageMessage {
    /// a float value
    Value(f64),
}

impl OnMessage for MovingAverageMessage {
    /// Process the inbound time_stamp message.
    fn on_message(
        self,
        prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        _request: &Self::RequestHub,
    ) {
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
type MovingAverage = Actor<
    MovingAverageProp,
    MovingAverageInbound,
    MovingAverageState,
    MovingAverageOutbound,
    NullRequest,
>;

/// Manual implementation of the moving average actor
pub mod manual;
