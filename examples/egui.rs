use std::time::Duration;

use hollywood::actors::egui::EguiActor;
use hollywood::actors::egui::EguiAppFromBuilder;
use hollywood::actors::egui::GenericEguiBuilder;
use hollywood::actors::egui::Stream;
use hollywood::prelude::*;

#[derive(Clone, Debug)]
#[actor_inputs(
    ContentGeneratorInbound,
    {NullProp, ContentGeneratorState, ContentGeneratorOutbound, ContentGeneratorOutRequest,
        ContentGeneratorInRequestMessage
    })]
pub enum ContentGeneratorInboundMessage {
    Tick(f64),
    Reply(ReplyMessage<String>),
}

/// Request to reset the content generator.
#[derive(Debug)]
pub enum ContentGeneratorInRequestMessage {
    /// Request
    Reset(RequestWithReplyChannel<(), f64>),
}

impl IsInRequestMessage for ContentGeneratorInRequestMessage {
    type Prop = NullProp;

    type State = ContentGeneratorState;

    type OutboundHub = ContentGeneratorOutbound;

    type OutRequestHub = ContentGeneratorOutRequest;

    fn in_request_channel(&self) -> String {
        "reset".to_owned()
    }
}

impl HasOnRequestMessage for ContentGeneratorInRequestMessage {
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        _outbound: &Self::OutboundHub,
        _request: &Self::OutRequestHub,
    ) {
        match self {
            ContentGeneratorInRequestMessage::Reset(msg) => {
                state.offset = -state.last_x;
                msg.reply(|_| state.last_x);
            }
        }
    }
}

impl HasOnMessage for ContentGeneratorInboundMessage {
    /// Process the inbound tick message.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        outbound: &Self::OutboundHub,
        request: &Self::OutRequestHub,
    ) {
        match &self {
            ContentGeneratorInboundMessage::Tick(new_value) => {
                state.last_x = *new_value;

                let x = *new_value + state.offset;

                let s = Stream {
                    msg: PlotMessage::SinPlot((x, x.sin())),
                };
                outbound.plot_message.send(s);
                let c = Stream {
                    msg: PlotMessage::SinPlot((x, x.cos())),
                };
                outbound.plot_message.send(c);

                if x > 2.0 && x < 2.1 {
                    request.example_request.send_request("foo:".to_owned());
                }
            }
            ContentGeneratorInboundMessage::Reply(r) => {
                println!("Reply received {}", r.reply);
            }
        }
    }
}

impl IsInboundMessageNew<f64> for ContentGeneratorInboundMessage {
    fn new(_inbound_name: String, msg: f64) -> Self {
        ContentGeneratorInboundMessage::Tick(msg)
    }
}

impl IsInboundMessageNew<ReplyMessage<String>> for ContentGeneratorInboundMessage {
    fn new(_inbound_name: String, msg: ReplyMessage<String>) -> Self {
        ContentGeneratorInboundMessage::Reply(msg)
    }
}

impl IsInRequestMessageNew<RequestWithReplyChannel<(), f64>> for ContentGeneratorInRequestMessage {
    fn new(_inbound_name: String, msg: RequestWithReplyChannel<(), f64>) -> Self {
        ContentGeneratorInRequestMessage::Reset(msg)
    }
}

#[derive(Debug)]
pub struct ContentGeneratorState {
    pub last_x: f64,
    pub offset: f64,
}

/// Out request channels for the content generator actor.
#[actor_out_requests]
pub struct ContentGeneratorOutRequest {
    pub example_request: OutRequestChannel<String, String, ContentGeneratorInboundMessage>,
}

/// The content generator actor.
#[actor(ContentGeneratorInboundMessage, ContentGeneratorInRequestMessage)]
type ContentGenerator = Actor<
    NullProp,
    ContentGeneratorInbound,
    ContentGeneratorInRequest,
    ContentGeneratorState,
    ContentGeneratorOutbound,
    ContentGeneratorOutRequest,
>;

/// Outbound channels for the content generator actor.
#[actor_outputs]
pub struct ContentGeneratorOutbound {
    pub plot_message: OutboundChannel<Stream<PlotMessage>>,
}

pub struct ContentGeneratorInRequest {
    pub reset: InRequestChannel<RequestWithReplyChannel<(), f64>, ContentGeneratorInRequestMessage>,
}

impl
    IsInRequestHub<
        NullProp,
        ContentGeneratorState,
        ContentGeneratorOutbound,
        ContentGeneratorOutRequest,
        ContentGeneratorInboundMessage,
        ContentGeneratorInRequestMessage,
    > for ContentGeneratorInRequest
{
    fn from_builder(
        builder: &mut ActorBuilder<
            NullProp,
            ContentGeneratorState,
            ContentGeneratorOutbound,
            ContentGeneratorOutRequest,
            ContentGeneratorInboundMessage,
            ContentGeneratorInRequestMessage,
        >,
        actor_name: &str,
    ) -> Self {
        let reset = InRequestChannel::new(
            builder.context,
            actor_name,
            &builder.request_sender.clone(),
            "reset".to_owned(),
        );
        builder
            .forward_request
            .insert(reset.name.clone(), Box::new(reset.clone()));

        Self { reset }
    }
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

struct EguiAppExampleAppConfig {}

type EguiAppExampleBuilder = GenericEguiBuilder<
    PlotMessage,
    RequestWithReplyChannel<String, String>,
    (),
    f64,
    EguiAppExampleAppConfig,
>;

pub struct EguiAppExample {
    pub message_recv: tokio::sync::mpsc::UnboundedReceiver<Stream<PlotMessage>>,
    pub in_request_recv:
        tokio::sync::mpsc::UnboundedReceiver<RequestWithReplyChannel<String, String>>,
    pub out_reply_recv: tokio::sync::mpsc::UnboundedReceiver<ReplyMessage<f64>>,
    pub out_request_sender: tokio::sync::mpsc::UnboundedSender<()>,
    pub cancel_request_sender: tokio::sync::mpsc::UnboundedSender<hollywood::CancelRequest>,

    pub x: f64,
    pub sin_value: f64,
    pub cos_value: f64,
}

impl EguiAppFromBuilder<EguiAppExampleBuilder> for EguiAppExample {
    fn new(builder: EguiAppExampleBuilder, _dummy_example_state: String) -> Box<EguiAppExample> {
        Box::new(EguiAppExample {
            message_recv: builder.message_from_actor_recv,
            out_reply_recv: builder.out_reply_from_actor_recv,
            in_request_recv: builder.in_request_from_actor_recv,
            out_request_sender: builder.out_request_to_actor_sender,
            cancel_request_sender: builder.cancel_request_sender.unwrap(),
            x: 0.0,
            sin_value: 0.0,
            cos_value: 0.0,
        })
    }

    type Out = EguiAppExample;

    type State = String;
}
use eframe::egui;
use hollywood::OutRequestChannel;
use hollywood::ReplyMessage;
use hollywood::RequestWithReplyChannel;
impl eframe::App for EguiAppExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(value) = self.message_recv.try_recv() {
            match value.msg {
                PlotMessage::SinPlot((x, y)) => {
                    self.x = x;
                    self.sin_value = y;
                }
                PlotMessage::CosPlot((x, y)) => {
                    self.x = x;
                    self.cos_value = y;
                }
            }
        }
        while let Ok(value) = self.out_reply_recv.try_recv() {
            println!("Reply: {:?}", value);
        }
        while let Ok(value) = self.in_request_recv.try_recv() {
            //println!("Request: {:?}", value);

            value.reply(|_| "reply".to_owned());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, egui!");
            ui.label(format!("x: {}", self.x));
            ui.label(format!("sin(x): {}", self.sin_value));
            ui.label(format!("cos(y): {}", self.cos_value));

            if ui.button("Reset").clicked() {
                self.out_request_sender.send(()).unwrap();
            }
        });

        ctx.request_repaint_after(Duration::from_secs_f64(0.1));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cancel_request_sender
            .send(hollywood::CancelRequest::Cancel(()))
            .unwrap();
    }
}

pub async fn run_viewer_example() {
    let mut builder = EguiAppExampleBuilder::from_config(EguiAppExampleAppConfig {});

    // Pipeline configuration
    let pipeline = Hollywood::configure(&mut |context| {
        // Actor creation:
        // 1. Periodic timer to drive the simulation
        let mut timer = hollywood::actors::Periodic::new_with_period(context, 0.1);
        // 2. The content generator of the example
        let mut content_generator = ContentGenerator::from_prop_and_state(
            context,
            NullProp::default(),
            ContentGeneratorState {
                last_x: 0.0,
                offset: 0.0,
            },
        );
        // 3. The egui actor
        let mut egui_actor =
            EguiActor::<PlotMessage, String, String, (), f64>::from_builder(context, &builder);

        // Pipeline connections:
        timer
            .outbound
            .time_stamp
            .connect(context, &mut content_generator.inbound.tick);
        content_generator
            .outbound
            .plot_message
            .connect(context, &mut egui_actor.inbound.stream);

        egui_actor
            .out_requests
            .request
            .connect(context, &mut content_generator.in_requests.reset);

        content_generator
            .out_requests
            .example_request
            .connect(context, &mut egui_actor.in_requests.request);
    });

    // The cancel_requester is used to cancel the pipeline.
    builder
        .cancel_request_sender
        .clone_from(&pipeline.cancel_request_sender_template);

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
