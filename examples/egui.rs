use std::time::Duration;

use hollywood::actors::egui::{
    EguiActor, EguiAppFromBuilder, GenericEguiBuilder, Stream,
};
use hollywood::compute::Context;
use hollywood::core::request::RequestMessage;
use hollywood::core::*;
use hollywood_macros::*;

#[derive(Clone, Debug)]
#[actor_inputs(ContentGeneratorInbound,{NullProp, ContentGeneratorState, ContentGeneratorOutbound, NullRequest})]
pub enum ContentGeneratorInboundMessage {
    Tick(f64),
}

impl OnMessage for ContentGeneratorInboundMessage {
    /// Process the inbound tick message.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        _request: &Self::RequestHub,
    ) {
        match &self {
            ContentGeneratorInboundMessage::Tick(new_value) => {
                match state.reset_request.try_recv() {
                    Ok(_) => {
                        state.offset = -*new_value;
                    }
                    Err(_) => {}
                }

                let x = *new_value + state.offset;

                let s = Stream {
                    msg: PlotMessage::SinPlot((x, x.sin())),
                };
                outbound.plot_message.send(s);
                let c = Stream {
                    msg: PlotMessage::SinPlot((x, x.cos())),
                };
                outbound.plot_message.send(c);
            }
        }
    }
}

impl InboundMessageNew<f64> for ContentGeneratorInboundMessage {
    fn new(_inbound_name: String, msg: f64) -> Self {
        ContentGeneratorInboundMessage::Tick(msg)
    }
}

#[derive(Debug)]
pub struct ContentGeneratorState {
    pub reset_request: tokio::sync::broadcast::Receiver<()>,
    pub offset: f64,
}

#[actor(ContentGeneratorInboundMessage)]
type ContentGenerator = Actor<
    NullProp,
    ContentGeneratorInbound,
    ContentGeneratorState,
    ContentGeneratorOutbound,
    NullRequest,
>;

/// OutboundChannel channels for the filter actor.
#[actor_outputs]
pub struct ContentGeneratorOutbound {
    pub plot_message: OutboundChannel<Stream<PlotMessage>>,
}

#[derive(Clone, Debug)]
pub enum PlotMessage {
    SinPlot((f64, f64)),
    CosPlot((f64, f64)),
}

impl Default for PlotMessage {
    fn default() -> Self {
        PlotMessage::SinPlot((0.0, 0.0))
    }
}

struct EguiAppExampleAppConfig {
    reset_side_channel_tx: tokio::sync::broadcast::Sender<()>,
}

type EguiAppExampleBuilder =
    GenericEguiBuilder<PlotMessage, RequestMessage<String, String>, EguiAppExampleAppConfig>;

pub struct EguiAppExample {
    pub message_recv: std::sync::mpsc::Receiver<Stream<PlotMessage>>,
    pub request_recv: std::sync::mpsc::Receiver<RequestMessage<String, String>>,
    pub reset_side_channel_tx: tokio::sync::broadcast::Sender<()>,

    pub x: f64,
    pub sin_value: f64,
    pub cos_value: f64,
}

impl EguiAppFromBuilder<EguiAppExampleBuilder> for EguiAppExample {
    fn new(builder: EguiAppExampleBuilder, _dummy_example_state: String) -> Box<EguiAppExample> {
        Box::new(EguiAppExample {
            message_recv: builder.message_recv,
            request_recv: builder.request_recv,
            reset_side_channel_tx: builder.config.reset_side_channel_tx,
            x: 0.0,
            sin_value: 0.0,
            cos_value: 0.0,
        })
    }

    type Out = EguiAppExample;

    type State = String;
}
use eframe::egui;
impl eframe::App for EguiAppExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        loop {
            match self.message_recv.try_recv() {
                Ok(value) => match value.msg {
                    PlotMessage::SinPlot((x, y)) => {
                        self.x = x;
                        self.sin_value = y;
                    }
                    PlotMessage::CosPlot((x, y)) => {
                        self.x = x;
                        self.cos_value = y;
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, egui!");
            ui.label(format!("x: {}", self.x));
            ui.label(format!("sin(x): {}", self.sin_value));
            ui.label(format!("cos(y): {}", self.cos_value));

            if ui.button("Reset").clicked() {
                self.reset_side_channel_tx.send(()).unwrap();
                // let (reply_channel_sender, reply_receiver) = tokio::sync::oneshot::channel();

                // tokio::spawn(async move {
                //     let reply = reply_receiver.await.unwrap();
                //     println!("Reply: {}", reply.reply);
                // });
            }
        });

        ctx.request_repaint_after(Duration::from_secs_f64(0.1));
    }
}

pub async fn run_viewer_example() {
    let (reset_side_channel_tx, _) = tokio::sync::broadcast::channel(1);
    let mut builder = EguiAppExampleBuilder::from_config(EguiAppExampleAppConfig {
        reset_side_channel_tx: reset_side_channel_tx.clone(),
    });

    // Pipeline configuration
    let pipeline = hollywood::compute::Context::configure(&mut |context| {
        // Actor creation:
        // 1. Periodic timer to drive the simulation
        let mut timer = hollywood::actors::Periodic::new_with_period(context, 0.1);
        // 2. The content generator of the example
        let mut content_generator = ContentGenerator::from_prop_and_state(
            context,
            NullProp::default(),
            ContentGeneratorState {
                reset_request: reset_side_channel_tx.subscribe(),
                offset: 0.0,
            },
        );
        // 3. The egui actor
        let mut egui_actor =
            EguiActor::<PlotMessage, String, String>::from_builder(context, &builder);

        // // Pipeline connections:
        timer
            .outbound
            .time_stamp
            .connect(context, &mut content_generator.inbound.tick);
        content_generator
            .outbound
            .plot_message
            .connect(context, &mut egui_actor.inbound.stream);
    });

    // The cancel_requester is used to cancel the pipeline.
    builder.cancel_request_sender = pipeline.cancel_request_sender_template.clone();

    // Plot the pipeline graph to the console.
    pipeline.print_flow_graph();

    // Pipeline execution:

    // 1. Run the pipeline on a separate thread.
    let pipeline_handle = tokio::spawn(pipeline.run());
    // 2. Run the viewer on the main thread. This is a blocking call.
    run_egui_app_on_man_thread::<EguiAppExampleBuilder, EguiAppExample>(builder);
    // 3. Wait for the pipeline to finish.
    pipeline_handle.await.unwrap();
}

// Run the egui app on the main thread.
pub fn run_egui_app_on_man_thread<
    Builder: 'static,
    V: EguiAppFromBuilder<Builder, State = String>,
>(
    builder: Builder,
) {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        renderer: eframe::Renderer::Wgpu,

        ..Default::default()
    };
    eframe::run_native(
        "Egui actor",
        options,
        Box::new(|_cc| {
            let s: String = "example_state".to_owned();
            V::new(builder, s)
        }),
    )
    .unwrap();
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_viewer_example().await;
        })
}
