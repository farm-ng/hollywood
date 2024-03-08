use crate::compute::CancelRequest;
use crate::compute::Context;
use crate::core::*;
use std::fmt::Debug;

use self::request::IsRequestMessage;
use self::request::RequestMessage;

/// The inbound message for the egui actor.
#[derive(Clone, Debug, Default)]
pub struct EguiState<T: Default + Debug + Clone + Send + Sync + 'static, InReqMsg: IsRequestMessage>
{
    /// Forwards messages to the egui app.
    pub forward_message: Option<std::sync::mpsc::Sender<Stream<T>>>,
    /// Forwards an incoming request to the egui app.
    pub forward_request: Option<std::sync::mpsc::Sender<InReqMsg>>,
}

/// The inbound message stream.
#[derive(Clone, Debug, Default)]
pub struct Stream<T: Default + Debug + Clone + Send + Sync + 'static> {
    /// The message to be forwarded to the egui app.
    pub msg: T,
}

/// The inbound message for the egui actor.
#[derive(Clone, Debug)]
pub enum EguiInboundMessage<
    T: Default + Debug + Clone + Send + Sync + 'static,
    Request: Default + Debug + Clone + Send + Sync + 'static,
    Reply: Default + Debug + Clone + Send + Sync + 'static,
> {
    /// A egui message of generic type T.
    Stream(Stream<T>),
    /// A generic request message.
    Request(RequestMessage<Request, Reply>),
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    > InboundMessageNew<Stream<T>> for EguiInboundMessage<T, Request, Reply>
{
    fn new(_inbound_name: String, p: Stream<T>) -> Self {
        EguiInboundMessage::<T, Request, Reply>::Stream(p)
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    > InboundMessageNew<RequestMessage<Request, Reply>> for EguiInboundMessage<T, Request, Reply>
{
    fn new(_inbound_name: String, p: RequestMessage<Request, Reply>) -> Self {
        EguiInboundMessage::<T, Request, Reply>::Request(p)
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    > InboundMessage for EguiInboundMessage<T, Request, Reply>
{
    type Prop = NullProp;

    type State = EguiState<T, RequestMessage<Request, Reply>>;

    type OutboundHub = NullOutbound;

    type RequestHub = NullRequest;

    fn inbound_channel(&self) -> String {
        "stream".to_owned()
    }
}

/// The inbound hub for the egui actor.
pub struct ViewerInbound<
    T: Default + Debug + Clone + Send + Sync + 'static,
    Request: Default + Debug + Clone + Send + Sync + 'static,
    Reply: Default + Debug + Clone + Send + Sync + 'static,
> {
    /// The message stream inbound channel
    pub stream: InboundChannel<Stream<T>, EguiInboundMessage<T, Request, Reply>>,
    /// The request inbound channel
    pub request:
        InboundChannel<RequestMessage<Request, Reply>, EguiInboundMessage<T, Request, Reply>>,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    >
    InboundHub<
        NullProp,
        EguiState<T, RequestMessage<Request, Reply>>,
        NullOutbound,
        NullRequest,
        EguiInboundMessage<T, Request, Reply>,
    > for ViewerInbound<T, Request, Reply>
{
    fn from_builder(
        builder: &mut ActorBuilder<
            NullProp,
            EguiState<T, RequestMessage<Request, Reply>>,
            NullOutbound,
            NullRequest,
            EguiInboundMessage<T, Request, Reply>,
        >,
        actor_name: &str,
    ) -> Self {
        let stream = InboundChannel::new(
            builder.context,
            actor_name,
            &builder.sender,
            "stream".to_owned(),
        );
        builder
            .forward
            .insert(stream.name.clone(), Box::new(stream.clone()));

        let request = InboundChannel::new(
            builder.context,
            actor_name,
            &builder.sender,
            "in_rquest".to_owned(),
        );
        builder
            .forward
            .insert(request.name.clone(), Box::new(request.clone()));

        Self { stream, request }
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    > OnMessage for EguiInboundMessage<T, Request, Reply>
{
    /// Forward the message to the egui app.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        _outbound: &Self::OutboundHub,
        _request: &Self::RequestHub,
    ) {
        match &self {
            EguiInboundMessage::Stream(new_value) => {
                if let Some(sender) = &state.forward_message {
                    sender.send(new_value.clone()).unwrap();
                }
            }
            EguiInboundMessage::Request(request) => {
                if let Some(sender) = &state.forward_request {
                    sender.send(request.clone()).unwrap();
                }
            }
        }
    }
}

/// The egui actor.
///
/// This is a generic proxy which receives messages and forwards them to the egui app.
pub type EguiActor<T, Request, Reply> = Actor<
    NullProp,
    ViewerInbound<T, Request, Reply>,
    EguiState<T, RequestMessage<Request, Reply>>,
    NullOutbound,
    NullRequest,
>;

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    >
    FromPropState<
        NullProp,
        ViewerInbound<T, Request, Reply>,
        EguiState<T, RequestMessage<Request, Reply>>,
        NullOutbound,
        EguiInboundMessage<T, Request, Reply>,
        NullRequest,
        DefaultRunner<
            NullProp,
            ViewerInbound<T, Request, Reply>,
            EguiState<T, RequestMessage<Request, Reply>>,
            NullOutbound,
            NullRequest,
        >,
    > for EguiActor<T, Request, Reply>
{
    fn name_hint(_prop: &NullProp) -> String {
        "Egui".to_owned()
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        Request: Default + Debug + Clone + Send + Sync + 'static,
        Reply: Default + Debug + Clone + Send + Sync + 'static,
    > EguiActor<T, Request, Reply>
{
    /// Create a new egui actor from the builder.
    pub fn from_builder<Builder: EguiActorBuilder<T, RequestMessage<Request, Reply>>>(
        context: &mut Context,
        builder: &Builder,
    ) -> Self {
        Self::from_prop_and_state(
            context,
            NullProp {},
            EguiState::<T, RequestMessage<Request, Reply>> {
                forward_message: Some(builder.message_sender()),
                forward_request: Some(builder.request_sender()),
            },
        )
    }
}

/// The egui actor builder.
pub trait EguiActorBuilder<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InReqMsg: IsRequestMessage,
>
{
    /// Returns message sender.
    fn message_sender(&self) -> std::sync::mpsc::Sender<Stream<T>>;
    /// Returns in request sender.
    fn request_sender(&self) -> std::sync::mpsc::Sender<InReqMsg>;
}

/// A generic builder for the egui actor and app.
pub struct GenericEguiBuilder<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InReqMsg: IsRequestMessage,
    Config,
> {
    /// The message sender for the egui actor to forward messages to the egui app.
    pub message_sender: std::sync::mpsc::Sender<Stream<T>>,
    /// The message recv for the egui app.
    pub message_recv: std::sync::mpsc::Receiver<Stream<T>>,
    /// To forward incoming  requests to the egui app.
    pub request_sender: std::sync::mpsc::Sender<InReqMsg>,
    /// The receiver for incoming requests.
    pub request_recv: std::sync::mpsc::Receiver<InReqMsg>,
    /// Pipeline cancel request sender
    pub cancel_request_sender: Option<tokio::sync::mpsc::Sender<CancelRequest>>,

    /// The config for the egui app.
    pub config: Config,
}

impl<T: Default + Debug + Clone + Send + Sync + 'static, InReqMsg: IsRequestMessage, Config>
    GenericEguiBuilder<T, InReqMsg, Config>
{
    /// Create a new viewer builder.
    pub fn from_config(config: Config) -> Self {
        let (sender, recv) = std::sync::mpsc::channel();
        let (request_sender, request_recv) = std::sync::mpsc::channel();

        Self {
            message_sender: sender,
            message_recv: recv,
            request_sender,
            request_recv,
            cancel_request_sender: None,
            config,
        }
    }
}

impl<T: Default + Debug + Clone + Send + Sync + 'static, InReqMsg: IsRequestMessage, Config>
    EguiActorBuilder<T, InReqMsg> for GenericEguiBuilder<T, InReqMsg, Config>
{
    fn message_sender(&self) -> std::sync::mpsc::Sender<Stream<T>> {
        self.message_sender.clone()
    }

    fn request_sender(&self) -> std::sync::mpsc::Sender<InReqMsg> {
        self.request_sender.clone()
    }
}

/// Trait the custom egui app must implement to work with the egui actor.
pub trait EguiAppFromBuilder<Builder>: eframe::App {
    /// Egui app type.
    type Out: EguiAppFromBuilder<Builder> + 'static;
    /// Gui state
    type State;

    /// Create a new egui app from the builder.
    fn new(builder: Builder, init_state: Self::State) -> Box<Self::Out>;
}
