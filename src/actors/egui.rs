use crate::compute::Context;
use crate::core::*;
use std::fmt::Debug;

use self::request::{IsRequestMessage, ReplyMessage, RequestChannel, RequestMessage};

/// The inbound message for the egui actor.
#[derive(Clone, Debug, Default)]
pub struct EguiState<T: Default + Debug + Clone + Send + Sync + 'static, InReqMsg: IsRequestMessage>
{
    /// Forwards messages to the egui app.
    pub forward_message: Option<std::sync::mpsc::Sender<Stream<T>>>,
    /// Forwards an incoming request to the egui app.
    pub forward_in_request: Option<std::sync::mpsc::Sender<InReqMsg>>,
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
    InRequest: Default + Debug + Clone + Send + Sync + 'static,
    InReply: Default + Debug + Clone + Send + Sync + 'static,
> {
    /// A egui message of generic type T.
    Stream(Stream<T>),
    /// A generic request message.
    InRequest(RequestMessage<InRequest, InReply>),
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    > InboundMessageNew<Stream<T>> for EguiInboundMessage<T, InRequest, InReply>
{
    fn new(_inbound_name: String, p: Stream<T>) -> Self {
        EguiInboundMessage::<T, InRequest, InReply>::Stream(p)
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    > InboundMessageNew<RequestMessage<InRequest, InReply>>
    for EguiInboundMessage<T, InRequest, InReply>
{
    fn new(_inbound_name: String, p: RequestMessage<InRequest, InReply>) -> Self {
        EguiInboundMessage::<T, InRequest, InReply>::InRequest(p)
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    > InboundMessage for EguiInboundMessage<T, InRequest, InReply>
{
    type Prop = NullProp;

    type State = EguiState<T, RequestMessage<InRequest, InReply>>;

    type OutboundHub = NullOutbound;

    type RequestHub = NullRequest;

    fn inbound_channel(&self) -> String {
        "msg".to_owned()
    }
}

/// The inbound hub for the egui actor.
pub struct ViewerInbound<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InRequest: Default + Debug + Clone + Send + Sync + 'static,
    InReply: Default + Debug + Clone + Send + Sync + 'static,
> {
    /// The inbound channel for the egui actor.
    pub msg: InboundChannel<Stream<T>, EguiInboundMessage<T, InRequest, InReply>>,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    >
    InboundHub<
        NullProp,
        EguiState<T, RequestMessage<InRequest, InReply>>,
        NullOutbound,
        NullRequest,
        EguiInboundMessage<T, InRequest, InReply>,
    > for ViewerInbound<T, InRequest, InReply>
{
    fn from_builder(
        builder: &mut ActorBuilder<
            NullProp,
            EguiState<T, RequestMessage<InRequest, InReply>>,
            NullOutbound,
            NullRequest,
            EguiInboundMessage<T, InRequest, InReply>,
        >,
        actor_name: &str,
    ) -> Self {
        let msg = InboundChannel::new(
            &mut builder.context,
            actor_name,
            &builder.sender,
            "msg".to_owned(),
        );
        builder
            .forward
            .insert(msg.name.clone(), Box::new(msg.clone()));
        Self { msg }
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    > OnMessage for EguiInboundMessage<T, InRequest, InReply>
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
                    let _ = sender.send(new_value.clone()).unwrap();
                }
            }
            EguiInboundMessage::InRequest(request) => {
                if let Some(sender) = &state.forward_in_request {
                    let _ = sender.send(request.clone()).unwrap();
                }
            }
        }
    }
}

/// The egui actor.
///
/// This is a generic proxy which receives messages and forwards them to the egui app.
pub type EguiActor<T, InRequest, InReply> = Actor<
    NullProp,
    ViewerInbound<T, InRequest, InReply>,
    EguiState<T, RequestMessage<InRequest, InReply>>,
    NullOutbound,
    NullRequest,
>;

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    >
    FromPropState<
        NullProp,
        ViewerInbound<T, InRequest, InReply>,
        EguiState<T, RequestMessage<InRequest, InReply>>,
        NullOutbound,
        EguiInboundMessage<T, InRequest, InReply>,
        NullRequest,
        DefaultRunner<
            NullProp,
            ViewerInbound<T, InRequest, InReply>,
            EguiState<T, RequestMessage<InRequest, InReply>>,
            NullOutbound,
            NullRequest,
        >,
    > for EguiActor<T, InRequest, InReply>
{
    fn name_hint(_prop: &NullProp) -> String {
        "Egui".to_owned()
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
    > EguiActor<T, InRequest, InReply>
{
    /// Create a new egui actor from the builder.
    pub fn from_builder<Builder: EguiActorBuilder<T, RequestMessage<InRequest, InReply>>>(
        context: &mut Context,
        builder: &Builder,
    ) -> Self {
        Self::from_prop_and_state(
            context,
            NullProp {},
            EguiState::<T, RequestMessage<InRequest, InReply>> {
                forward_message: Some(builder.message_sender()),
                forward_in_request: Some(builder.in_request_sender()),
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
    fn in_request_sender(&self) -> std::sync::mpsc::Sender<InReqMsg>;
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
    pub in_request_sender: std::sync::mpsc::Sender<InReqMsg>,
    /// The receiver for incoming requests.
    pub in_request_recv: std::sync::mpsc::Receiver<InReqMsg>,

    /// The config for the egui app.
    pub config: Config,
}

impl<T: Default + Debug + Clone + Send + Sync + 'static, InReqMsg: IsRequestMessage, Config>
    GenericEguiBuilder<T, InReqMsg, Config>
{
    /// Create a new viewer builder.
    pub fn from_config(config: Config) -> Self {
        let (sender, recv) = std::sync::mpsc::channel();
        let (in_request_sender, in_request_recv) = std::sync::mpsc::channel();
        Self {
            message_sender: sender,
            message_recv: recv,
            in_request_sender: in_request_sender,
            in_request_recv: in_request_recv,
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

    fn in_request_sender(&self) -> std::sync::mpsc::Sender<InReqMsg> {
        self.in_request_sender.clone()
    }
}

/// Trait the custom egui app must implement to work with the egui actor.
pub trait EguiAppFromBuilder<Builder>: eframe::App {
    /// Egui app type.
    type Out: EguiAppFromBuilder<Builder> + 'static;

    /// Create a new egui app from the builder.
    fn new(builder: Builder) -> Box<Self::Out>;
}
