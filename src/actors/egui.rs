use crate::prelude::*;
use crate::CancelRequest;
use crate::OutRequestChannel;
use crate::ReplyMessage;
use crate::RequestWithReplyChannel;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

/// The inbound message for the egui actor.
#[derive(Debug)]
pub struct EguiState<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InReqMsg: IsRequestWithReplyChannel,
    OutRequest,
    OutReply,
> {
    /// Forwards messages to the egui app.
    pub forward_message_to_egui_app: Option<tokio::sync::mpsc::UnboundedSender<Stream<T>>>,
    /// Forwards an incoming request to the egui app.
    pub forward_in_request_to_egui_app: Option<tokio::sync::mpsc::UnboundedSender<InReqMsg>>,
    /// Forwards an outbound reply to the egui app.
    pub forward_out_reply_to_egui_app:
        Option<tokio::sync::mpsc::UnboundedSender<ReplyMessage<OutReply>>>,

    /// Forwards an outbound request from the egui app.
    pub forward_out_request_from_egui_app:
        Option<Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<OutRequest>>>>,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InReqMsg: IsRequestWithReplyChannel,
        OutRequest,
        OutReply,
    > Clone for EguiState<T, InReqMsg, OutRequest, OutReply>
{
    fn clone(&self) -> Self {
        Self {
            forward_message_to_egui_app: self.forward_message_to_egui_app.clone(),
            forward_in_request_to_egui_app: self.forward_in_request_to_egui_app.clone(),
            forward_out_reply_to_egui_app: self.forward_out_reply_to_egui_app.clone(),
            forward_out_request_from_egui_app: self.forward_out_request_from_egui_app.clone(),
        }
    }
}

/// The inbound message stream.
#[derive(Clone, Debug, Default)]
pub struct Stream<T: Default + Debug + Clone + Send + Sync + 'static> {
    /// The message to be forwarded to the egui app.
    pub msg: T,
}

/// The inbound message for the egui actor.
#[derive(Debug)]
pub enum EguiInboundMessage<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InRequest: Debug + Send + Clone + Sync + 'static,
    InReply: Debug + Send + Clone + Sync + 'static,
    OutRequest: Debug + Send + Clone + Sync + 'static,
    OutReply: Debug + Send + Clone + Sync + 'static,
> {
    /// A egui message of generic type T.
    Stream(Stream<T>),
    /// A generic request message.
    Dummy(PhantomData<(InRequest, InReply, OutRequest)>),
    /// A generic request message.
    OutReply(ReplyMessage<OutReply>),
}

/// The inbound message for the egui actor.
#[derive(Debug)]
pub enum EguiInRequestMessage<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InRequest: Debug + Send + Clone + Sync + 'static,
    InReply: Debug + Send + Clone + Sync + 'static,
    OutRequest: Debug + Send + Clone + Sync + 'static,
    OutReply: Debug + Send + Clone + Sync + 'static,
> {
    /// A generic request message.
    InRequest(RequestWithReplyChannel<InRequest, InReply>),
    /// A dummy message.
    Dummy(PhantomData<(T, OutRequest, OutReply)>),
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > Clone for EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    fn clone(&self) -> Self {
        match self {
            EguiInboundMessage::Stream(msg) => EguiInboundMessage::Stream(msg.clone()),
            EguiInboundMessage::Dummy(_) => EguiInboundMessage::Dummy(PhantomData),
            EguiInboundMessage::OutReply(reply) => EguiInboundMessage::OutReply(reply.clone()),
        }
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > IsInboundMessage for EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    type Prop = NullProp;

    type State = EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>;

    type OutboundHub = NullOutbound;

    type OutRequestHub = EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>;

    fn inbound_channel(&self) -> String {
        "stream".to_owned()
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > IsInRequestMessage for EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    type Prop = NullProp;

    type State = EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>;

    type OutboundHub = NullOutbound;

    type OutRequestHub = EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>;

    fn in_request_channel(&self) -> String {
        "in_request".to_owned()
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > IsInboundMessageNew<Stream<T>>
    for EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    fn new(_inbound_name: String, p: Stream<T>) -> Self {
        EguiInboundMessage::<T, InRequest, InReply, OutRequest, OutReply>::Stream(p)
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > IsInboundMessageNew<ReplyMessage<OutReply>>
    for EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    fn new(_inbound_name: String, p: ReplyMessage<OutReply>) -> Self {
        EguiInboundMessage::<T, InRequest, InReply, OutRequest, OutReply>::OutReply(p)
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > IsInRequestMessageNew<RequestWithReplyChannel<InRequest, InReply>>
    for EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    fn new(_inbound_name: String, p: RequestWithReplyChannel<InRequest, InReply>) -> Self {
        EguiInRequestMessage::<T, InRequest, InReply, OutRequest, OutReply>::InRequest(p)
    }
}

/// The inbound hub for the egui actor.
pub struct ViewerInbound<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InRequest: Debug + Send + Clone + Sync + 'static,
    InReply: Debug + Send + Clone + Sync + 'static,
    OutRequest: Debug + Send + Clone + Sync + 'static,
    OutReply: Debug + Send + Clone + Sync + 'static,
> {
    /// The message stream inbound channel
    pub stream:
        InboundChannel<Stream<T>, EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>>,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    >
    IsInboundHub<
        NullProp,
        EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
        NullOutbound,
        EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
        EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
        EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>,
    > for ViewerInbound<T, InRequest, InReply, OutRequest, OutReply>
{
    fn from_builder(
        builder: &mut ActorBuilder<
            NullProp,
            EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
            NullOutbound,
            EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
            EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
            EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>,
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

        Self { stream }
    }
}

/// The inbound hub for the egui actor.
pub struct ViewerInRequest<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InRequest: Debug + Send + Clone + Sync + 'static,
    InReply: Debug + Send + Clone + Sync + 'static,
    OutRequest: Debug + Send + Clone + Sync + 'static,
    OutReply: Debug + Send + Clone + Sync + 'static,
> {
    /// The request inbound channel
    #[allow(clippy::type_complexity)]
    pub request: InRequestChannel<
        RequestWithReplyChannel<InRequest, InReply>,
        EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>,
    >,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    >
    IsInRequestHub<
        NullProp,
        EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
        NullOutbound,
        EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
        EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
        EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>,
    > for ViewerInRequest<T, InRequest, InReply, OutRequest, OutReply>
{
    fn from_builder(
        builder: &mut ActorBuilder<
            NullProp,
            EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
            NullOutbound,
            EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
            EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
            EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>,
        >,
        actor_name: &str,
    ) -> Self {
        let request = InRequestChannel::new(
            builder.context,
            actor_name,
            &builder.request_sender,
            "in_request".to_owned(),
        );
        builder
            .forward_request
            .insert(request.name.clone(), Box::new(request.clone()));

        Self { request }
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > HasOnMessage for EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    /// Forward the message to the egui app.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        _outbound: &Self::OutboundHub,
        request: &Self::OutRequestHub,
    ) {
        {
            let mut recv = state
                .forward_out_request_from_egui_app
                .as_ref()
                .unwrap()
                .lock()
                .unwrap();
            let x = recv.try_recv();
            if let Ok(r) = x {
                request.request.send_request(r);
            }
        }

        match &self {
            EguiInboundMessage::Stream(new_value) => {
                if let Some(sender) = &state.forward_message_to_egui_app {
                    sender.send(new_value.clone()).unwrap();
                }
            }
            EguiInboundMessage::Dummy(_) => {}
            EguiInboundMessage::OutReply(reply) => {
                if let Some(sender) = &state.forward_out_reply_to_egui_app {
                    sender.send(reply.clone()).unwrap();
                }
            }
        }
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > HasOnRequestMessage for EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>
{
    /// Forward the message to the egui app.
    fn on_message(
        self,
        _prop: &Self::Prop,
        state: &mut Self::State,
        _outbound: &Self::OutboundHub,
        request: &Self::OutRequestHub,
    ) {
        {
            let mut recv = state
                .forward_out_request_from_egui_app
                .as_ref()
                .unwrap()
                .lock()
                .unwrap();
            let x = recv.try_recv();
            if let Ok(r) = x {
                request.request.send_request(r);
            }
        }
        match self {
            EguiInRequestMessage::InRequest(request) => {
                println!("EguiInRequestMessage::InRequest");
                if let Some(sender) = &state.forward_in_request_to_egui_app {
                    sender.send(request).unwrap();
                }
            }
            EguiInRequestMessage::Dummy(_) => {}
        }
    }
}

/// OutboundChannel channels for the filter actor.
pub struct EguiOutRequest<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InRequest: Debug + Send + Clone + Sync + 'static,
    InReply: Debug + Send + Clone + Sync + 'static,
    OutRequest: Debug + Send + Clone + Sync + 'static,
    OutReply: Debug + Send + Clone + Sync + 'static,
> {
    /// The outbound request channel
    pub request: OutRequestChannel<
        OutRequest,
        OutReply,
        EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
    >,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > HasActivate for EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>
{
    fn extract(&mut self) -> Self {
        Self {
            request: self.request.extract(),
        }
    }

    fn activate(&mut self) {
        self.request.activate()
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > IsOutRequestHub<EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>>
    for EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>
{
    fn from_parent_and_sender(
        actor_name: &str,
        sender: &tokio::sync::mpsc::UnboundedSender<
            EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
        >,
    ) -> Self {
        Self {
            request: OutRequestChannel::new("request".to_owned(), actor_name, sender),
        }
    }
}

/// The egui actor.
///
/// This is a generic proxy which receives messages and forwards them to the egui app.
pub type EguiActor<T, InRequest, InReply, OutRequest, OutReply> = Actor<
    NullProp,
    ViewerInbound<T, InRequest, InReply, OutRequest, OutReply>,
    ViewerInRequest<T, InRequest, InReply, OutRequest, OutReply>,
    EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
    NullOutbound,
    EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
>;

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Debug + Send + Clone + Sync + 'static,
        InReply: Debug + Send + Clone + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    >
    HasFromPropState<
        NullProp,
        ViewerInbound<T, InRequest, InReply, OutRequest, OutReply>,
        ViewerInRequest<T, InRequest, InReply, OutRequest, OutReply>,
        EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
        NullOutbound,
        EguiInboundMessage<T, InRequest, InReply, OutRequest, OutReply>,
        EguiInRequestMessage<T, InRequest, InReply, OutRequest, OutReply>,
        EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
        DefaultRunner<
            NullProp,
            ViewerInbound<T, InRequest, InReply, OutRequest, OutReply>,
            ViewerInRequest<T, InRequest, InReply, OutRequest, OutReply>,
            EguiState<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
            NullOutbound,
            EguiOutRequest<T, InRequest, InReply, OutRequest, OutReply>,
        >,
    > for EguiActor<T, InRequest, InReply, OutRequest, OutReply>
{
    fn name_hint(_prop: &NullProp) -> String {
        "Egui".to_owned()
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InRequest: Default + Debug + Clone + Send + Sync + 'static,
        InReply: Default + Debug + Clone + Send + Sync + 'static,
        OutRequest: Debug + Send + Clone + Sync + 'static,
        OutReply: Debug + Send + Clone + Sync + 'static,
    > EguiActor<T, InRequest, InReply, OutRequest, OutReply>
{
    /// Create a new egui actor from the builder.
    pub fn from_builder<
        Builder: EguiActorBuilder<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply>,
    >(
        context: &mut Hollywood,
        builder: &Builder,
    ) -> Self {
        Self::from_prop_and_state(
            context,
            NullProp {},
            EguiState::<T, RequestWithReplyChannel<InRequest, InReply>, OutRequest, OutReply> {
                forward_message_to_egui_app: Some(builder.message_to_egui_app_sender()),
                forward_in_request_to_egui_app: Some(builder.in_request_to_egui_app_sender()),
                forward_out_reply_to_egui_app: Some(builder.out_reply_to_egui_app_sender()),
                forward_out_request_from_egui_app: Some(builder.out_request_from_egui_app_recv()),
            },
        )
    }
}

/// The egui actor builder.
pub trait EguiActorBuilder<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InReqMsg: IsRequestWithReplyChannel,
    OutRequest,
    OutReply,
>
{
    /// Returns message sender.
    fn message_to_egui_app_sender(&self) -> tokio::sync::mpsc::UnboundedSender<Stream<T>>;
    /// Returns in request sender.
    fn in_request_to_egui_app_sender(&self) -> tokio::sync::mpsc::UnboundedSender<InReqMsg>;
    /// Returns out reply sender.
    fn out_reply_to_egui_app_sender(
        &self,
    ) -> tokio::sync::mpsc::UnboundedSender<ReplyMessage<OutReply>>;
    /// Returns out request receiver.
    fn out_request_from_egui_app_recv(
        &self,
    ) -> Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<OutRequest>>>;
}

/// A generic builder for the egui actor and app.
pub struct GenericEguiBuilder<
    T: Default + Debug + Clone + Send + Sync + 'static,
    InReqMsg: IsRequestWithReplyChannel,
    OutRequest,
    OutReply,
    Config,
> {
    /// To forward messages from actor to the egui app.
    pub message_to_egui_app_sender: tokio::sync::mpsc::UnboundedSender<Stream<T>>,
    /// To receive messages from the actor.
    pub message_from_actor_recv: tokio::sync::mpsc::UnboundedReceiver<Stream<T>>,

    /// To forward incoming requests to the egui app.
    pub in_request_to_egui_app_sender: tokio::sync::mpsc::UnboundedSender<InReqMsg>,
    /// To receive incoming requests from actor.
    pub in_request_from_actor_recv: tokio::sync::mpsc::UnboundedReceiver<InReqMsg>,

    /// To forward outgoing requests from actor to the egui app.
    pub out_reply_to_egui_app_sender: tokio::sync::mpsc::UnboundedSender<ReplyMessage<OutReply>>,
    /// To receive outgoing requests from the actor.
    pub out_reply_from_actor_recv: tokio::sync::mpsc::UnboundedReceiver<ReplyMessage<OutReply>>,

    /// To forward outgoing requests from the egui app to the actor.
    pub out_request_to_actor_sender: tokio::sync::mpsc::UnboundedSender<OutRequest>,
    /// To receive outgoing requests from the egui app.
    pub out_request_from_egui_app_recv:
        Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<OutRequest>>>,

    /// Pipeline cancel request sender
    pub cancel_request_sender: Option<tokio::sync::mpsc::UnboundedSender<CancelRequest>>,

    /// The config for the egui app.
    pub config: Config,
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InReqMsg: IsRequestWithReplyChannel,
        OutRequest,
        OutReply,
        Config,
    > GenericEguiBuilder<T, InReqMsg, OutRequest, OutReply, Config>
{
    /// Create a new viewer builder.
    pub fn from_config(config: Config) -> Self {
        let (message_to_egui_app_sender, message_from_actor_recv) =
            tokio::sync::mpsc::unbounded_channel();
        let (in_request_to_egui_app_sender, in_request_from_actor_recv) =
            tokio::sync::mpsc::unbounded_channel();
        let (out_request_to_actor_sender, out_request_from_egui_app_recv) =
            tokio::sync::mpsc::unbounded_channel();
        let (out_reply_to_egui_app_sender, out_reply_from_actor_recv) =
            tokio::sync::mpsc::unbounded_channel();

        Self {
            message_to_egui_app_sender,
            message_from_actor_recv,
            in_request_to_egui_app_sender,
            in_request_from_actor_recv,
            out_request_to_actor_sender,
            out_request_from_egui_app_recv: Arc::new(Mutex::new(out_request_from_egui_app_recv)),
            out_reply_to_egui_app_sender,
            out_reply_from_actor_recv,
            cancel_request_sender: None,
            config,
        }
    }
}

impl<
        T: Default + Debug + Clone + Send + Sync + 'static,
        InReqMsg: IsRequestWithReplyChannel,
        OutRequest,
        OutReply,
        Config,
    > EguiActorBuilder<T, InReqMsg, OutRequest, OutReply>
    for GenericEguiBuilder<T, InReqMsg, OutRequest, OutReply, Config>
{
    fn message_to_egui_app_sender(&self) -> tokio::sync::mpsc::UnboundedSender<Stream<T>> {
        self.message_to_egui_app_sender.clone()
    }

    fn in_request_to_egui_app_sender(&self) -> tokio::sync::mpsc::UnboundedSender<InReqMsg> {
        self.in_request_to_egui_app_sender.clone()
    }

    fn out_reply_to_egui_app_sender(
        &self,
    ) -> tokio::sync::mpsc::UnboundedSender<ReplyMessage<OutReply>> {
        self.out_reply_to_egui_app_sender.clone()
    }

    fn out_request_from_egui_app_recv(
        &self,
    ) -> Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<OutRequest>>> {
        self.out_request_from_egui_app_recv.clone()
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
