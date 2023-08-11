use crate::core::InboundMessageNew;
use crate::core::NullOutbound;
use crate::core::OnMessage;
use crate::core::Value;
use crate::core::*;
use crate::examples::one_dim_robot::{NamedFilterState, Robot, Stamped};
use crate::macros::*;
use drawille::Canvas;

/// Inbound channels for the draw actor
#[derive(Clone, Debug)]
#[actor_inputs(DrawInbound, {NullProp, DrawState, NullOutbound})]
pub enum DrawInboundMessage {
    /// True position of the robot.
    TruePos(Stamped<Robot>),
    /// True range measurement.
    TrueRange(Stamped<f64>),
    /// Filter estimate of the robot's position.
    FilterEst(NamedFilterState),
}

/// Draw actor for one-dim-robot example.
#[actor(DrawInboundMessage)]
pub type DrawActor = Actor<NullProp, DrawInbound, DrawState, NullOutbound>;

impl OnMessage for DrawInboundMessage {
    /// Forward the message to the correct handler method of [DrawState].
    fn on_message(&self, _prop: &NullProp, state: &mut Self::State, outbound: &Self::OutboundHub) {
        match self {
            DrawInboundMessage::TruePos(msg) => {
                state.true_pos(msg.clone(), outbound);
            }
            DrawInboundMessage::TrueRange(msg) => {
                state.true_range(msg.clone(), outbound);
            }
            DrawInboundMessage::FilterEst(msg) => {
                state.filter_est(msg.clone(), outbound);
            }
        }
    }
}

impl InboundMessageNew<Stamped<Robot>> for DrawInboundMessage {
    fn new(inbound_channel: String, msg: Stamped<Robot>) -> Self {
        match inbound_channel.as_str() {
            "TruePos" => DrawInboundMessage::TruePos(msg),
            _ => panic!("Unknown inbound name {}", inbound_channel),
        }
    }
}

impl InboundMessageNew<Stamped<f64>> for DrawInboundMessage {
    fn new(inbound_channel: String, msg: Stamped<f64>) -> Self {
        match inbound_channel.as_str() {
            "TrueRange" => DrawInboundMessage::TrueRange(msg),
            _ => panic!("Unknown inbound name {}", inbound_channel),
        }
    }
}

impl InboundMessageNew<NamedFilterState> for DrawInboundMessage {
    fn new(inbound_channel: String, msg: NamedFilterState) -> Self {
        match inbound_channel.as_str() {
            "FilterEst" => DrawInboundMessage::FilterEst(msg),
            _ => panic!("Unknown inbound name {}", inbound_channel),
        }
    }
}

/// State of the draw actor.
#[derive(Clone, Debug, Default)]
pub struct DrawState {
    /// The most recent true position.
    pub true_robot: Option<Stamped<Robot>>,
    /// The most recent true range measurement.
    pub true_range: Option<Stamped<f64>>,
    /// The most recent filter estimate.
    pub filter_est: Option<NamedFilterState>,
}

impl DrawState {
    /// Visualize the true robot state by saving it to the state and invoking [DrawState::draw].
    pub fn true_pos(&mut self, p: Stamped<Robot>, _: &NullOutbound) {
        self.true_robot = Some(p);

        self.draw();
    }

    /// Visualize the range measurement by saving it to the state and invoking [DrawState::draw].
    pub fn true_range(&mut self, p: Stamped<f64>, _: &NullOutbound) {
        self.true_range = Some(p);

        self.draw();
    }

    /// Visualize the filter estimate by saving it to the state and invoking [DrawState::draw].
    pub fn filter_est(&mut self, p: NamedFilterState, _: &NullOutbound) {
        self.filter_est = Some(p);
        self.draw();
    }

    /// Draw the current state to the console if all information of the most recent timestamp is 
    /// available.
    pub fn draw(&mut self) {
        let factor = 6.0;

        let width_pixels: u32 = 100;
        let height_pixels: u32 = 10;
        let u_offset = 0.5 * (width_pixels as f64);
        let v_offset = 0.9 * (height_pixels as f64);

        let pixels_from_meter = |x: f64, y: f64| -> (u32, u32) {
            (
                ((x * factor) + u_offset) as u32,
                ((y * -factor) + v_offset) as u32,
            )
        };

        let mut canvas = Canvas::new(width_pixels, height_pixels);

        if let Some(p) = &self.true_robot {
            let true_x = p.value.position;
            if let Some(r) = &self.true_range {
                if let Some(est) = &self.filter_est {
                    if r.time == p.time && r.time == est.state.time {
                        let x_left_ground = pixels_from_meter(true_x - 0.25, 0.0);
                        let x_right_ground = pixels_from_meter(true_x + 0.25, 0.0);
                        let x_left_up = pixels_from_meter(true_x - 0.25, 1.5);
                        let x_right_up = pixels_from_meter(true_x + 0.25, 1.5);
                        canvas.line_colored(
                            x_left_ground.0,
                            x_left_ground.1,
                            x_left_up.0,
                            x_left_up.1,
                            drawille::PixelColor::Blue,
                        );
                        canvas.line_colored(
                            x_left_up.0,
                            x_left_up.1,
                            x_right_up.0,
                            x_right_up.1,
                            drawille::PixelColor::Blue,
                        );
                        canvas.line_colored(
                            x_right_ground.0,
                            x_right_ground.1,
                            x_right_up.0,
                            x_right_up.1,
                            drawille::PixelColor::Blue,
                        );

                        let range_start = pixels_from_meter(p.value.position + 0.5, 1.0);
                        let range_endpoint = pixels_from_meter(p.value.position + r.value, 1.0);
                        canvas.line_colored(
                            range_start.0,
                            range_start.1,
                            range_endpoint.0,
                            range_endpoint.1,
                            drawille::PixelColor::Red,
                        );

                        let std = est.state.robot_position.covariance.sqrt();

                        let believe_min = pixels_from_meter(p.value.position - 3.0 * std, 1.0);
                        let believe_max = pixels_from_meter(p.value.position + 3.0 * std, 1.0);

                        canvas.line_colored(
                            believe_min.0,
                            5,
                            believe_min.0,
                            10,
                            drawille::PixelColor::Green,
                        );
                        canvas.line_colored(
                            believe_max.0,
                            5,
                            believe_max.0,
                            10,
                            drawille::PixelColor::Green,
                        );

                        println!("time:{}\n{}", r.time, canvas.frame());
                    }
                }
            }
        }
    }
}

impl Value for DrawState {}
