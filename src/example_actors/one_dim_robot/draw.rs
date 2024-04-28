use crate::actors::zip::Tuple3;
use crate::example_actors::one_dim_robot::NamedFilterState;
use crate::example_actors::one_dim_robot::Robot;
use crate::example_actors::one_dim_robot::Stamped;
use crate::prelude::*;
use drawille::Canvas;

/// Inbound channels for the draw actor
#[derive(Clone, Debug)]
#[actor_inputs(DrawInbound,
    {
        NullProp,
        DrawState,
        NullOutbound,
        NullOutRequests,
        NullInRequestMessage
    })]
pub enum DrawInboundMessage {
    /// Tuple of true pos, true range and filter state
    Zipped(Tuple3<u64, Stamped<Robot>, Stamped<f64>, NamedFilterState>),
}

/// Draw actor for one-dim-robot example.
#[actor(DrawInboundMessage, NullInRequestMessage)]
pub type DrawActor =
    Actor<NullProp, DrawInbound, NullInRequests, DrawState, NullOutbound, NullOutRequests>;

impl HasOnMessage for DrawInboundMessage {
    /// Forward the message to the correct handler method of [DrawState].
    fn on_message(
        self,
        _prop: &NullProp,
        state: &mut Self::State,
        _outbound: &Self::OutboundHub,
        _request: &Self::OutRequestHub,
    ) {
        match self {
            DrawInboundMessage::Zipped(msg) => {
                state.draw(msg.item0.clone(), msg.item1.clone(), msg.item2.clone());
            }
        }
    }
}

impl IsInboundMessageNew<Tuple3<u64, Stamped<Robot>, Stamped<f64>, NamedFilterState>>
    for DrawInboundMessage
{
    fn new(
        inbound_channel: String,
        msg: Tuple3<u64, Stamped<Robot>, Stamped<f64>, NamedFilterState>,
    ) -> Self {
        match inbound_channel.as_str() {
            "Zipped" => DrawInboundMessage::Zipped(msg),
            _ => panic!("Unknown inbound name {}", inbound_channel),
        }
    }
}

/// State of the draw actor.
#[derive(Clone, Debug, Default)]
pub struct DrawState {}

impl DrawState {
    /// Draw the current state to the console if all information of the most recent timestamp is
    /// available.
    pub fn draw(
        &mut self,
        true_robot: Stamped<Robot>,
        true_range: Stamped<f64>,
        filter_est: NamedFilterState,
    ) {
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

        let true_x = true_robot.value.position;
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

        let p = filter_est.state.clone();
        let r = true_range;
        let range_start = pixels_from_meter(p.pos_vel_acc.mean.x + 0.5, 1.0);
        let range_endpoint = pixels_from_meter(p.pos_vel_acc.mean.x + r.value, 1.0);
        canvas.line_colored(
            range_start.0,
            range_start.1,
            range_endpoint.0,
            range_endpoint.1,
            drawille::PixelColor::Red,
        );

        let std = filter_est.state.pos_vel_acc.covariance[(0, 0)].sqrt();

        let believe_min = pixels_from_meter(p.pos_vel_acc.mean.x - 3.0 * std, 1.0);
        let believe_max = pixels_from_meter(p.pos_vel_acc.mean.x + 3.0 * std, 1.0);

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
