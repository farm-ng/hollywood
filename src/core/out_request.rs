use crate::core::connection::request_connection::RequestConnection;
use crate::core::connection::RequestConnectionEnum;
use crate::prelude::*;
use linear_type::Linear;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::warn;

/// A request hub is used to send requests to other actors which will reply later.
pub trait IsOutRequestHub<M: IsInboundMessage>: Send + Sync + 'static + HasActivate {
    /// Create a new request hub for an actor.
    fn from_parent_and_sender(
        actor_name: &str,
        sender: &tokio::sync::mpsc::UnboundedSender<M>,
    ) -> Self;
}

/// A request message with a reply channel.
///
/// Sending a reply will consume this struct.
///
/// ### Panics:
///
/// The struct cannot be dropped before a reply is sent. The reply must be sent using
/// [RequestWithReplyChannel::reply] or [RequestWithReplyChannel::reply_from_request]. If the
/// struct is dropped without a reply sent, this thread will panic. In this case, the thread of the
/// reply receiver will continue with a warning.
/// However, the reply does not need to be sent immediately. This struct can be stored and
/// forwarded, but it must be sent before the request is dropped.
///
/// The intention of the panic behavior is to make it easier to catch subtle programming
/// errors early where a reply is not sent by mistake. The stacktrace will show the location
/// where the request struct was dropped (before a reply was sent).
///
/// This behavior might change in the future.
#[derive(Debug)]
pub struct RequestWithReplyChannel<Request, Reply> {
    /// The request.
    pub request: Request,
    pub(crate) reply_channel: Linear<tokio::sync::oneshot::Sender<ReplyMessage<Reply>>>,
}

/// A trait for request messages.
pub trait IsRequestWithReplyChannel: Send + Sync + 'static + Debug {
    /// The request type.
    type Request;
    /// The reply type.
    type Reply;
}

impl<
        Request: Send + Sync + 'static + Clone + Debug,
        Reply: Send + Sync + 'static + Clone + Debug,
    > IsRequestWithReplyChannel for RequestWithReplyChannel<Request, Reply>
{
    type Request = Reply;
    type Reply = Reply;
}

impl<Request, Reply: Debug> RequestWithReplyChannel<Request, Reply> {
    /// Reply to the request using the provided function.
    pub fn reply_from_request<F>(self, func: F)
    where
        F: FnOnce(Request) -> Reply,
    {
        let request = self.request;
        let reply = func(request);

        let reply_channel = self.reply_channel.into_inner();
        reply_channel.send(ReplyMessage { reply }).unwrap();
    }

    /// Reply to the request.
    pub fn reply(self, reply: Reply) {
        let reply_channel = self.reply_channel.into_inner();
        reply_channel.send(ReplyMessage { reply }).unwrap();
    }
}

/// A reply to a request.
#[derive(Debug, Clone, Default)]
pub struct ReplyMessage<Reply> {
    /// The reply value.
    pub reply: Reply,
}

/// OutRequestChannel is a connections for sending requests to other actors (and receiving replies
/// later).
pub struct OutRequestChannel<Request, Reply, M: IsInboundMessage> {
    /// Unique name of the request channel.
    pub name: String,
    /// Name of the actor that sends the request messages.
    pub actor_name: String,

    pub(crate) connection_register: RequestConnectionEnum<RequestWithReplyChannel<Request, Reply>>,
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<M>,
}

impl<Request, Reply, M: IsInboundMessage> HasActivate for OutRequestChannel<Request, Reply, M> {
    fn extract(&mut self) -> Self {
        Self {
            name: self.name.clone(),
            actor_name: self.actor_name.clone(),
            connection_register: self.connection_register.extract(),
            sender: self.sender.clone(),
        }
    }

    fn activate(&mut self) {
        self.connection_register.activate();
    }
}

impl<
        Request: Clone + Send + Sync + std::fmt::Debug + 'static,
        Reply: Clone + Send + Sync + std::fmt::Debug + 'static,
        M: IsInboundMessageNew<ReplyMessage<Reply>>,
    > OutRequestChannel<Request, Reply, M>
{
    /// Creates a new out-request channel for the actor.
    pub fn new(
        name: String,
        actor_name: &str,
        sender: &tokio::sync::mpsc::UnboundedSender<M>,
    ) -> Self {
        Self {
            name: name.clone(),
            actor_name: actor_name.to_owned(),
            connection_register: RequestConnectionEnum::new(),
            sender: sender.clone(),
        }
    }

    /// Connects the out-request channel from this actor to the in-request channel of another actor.
    pub fn connect<Me: IsInRequestMessageNew<RequestWithReplyChannel<Request, Reply>>>(
        &mut self,
        _ctx: &mut Hollywood,
        inbound: &mut InRequestChannel<RequestWithReplyChannel<Request, Reply>, Me>,
    ) {
        self.connection_register.push(Arc::new(RequestConnection {
            sender: inbound.sender.as_ref().clone(),
            inbound_channel: inbound.name.clone(),
            phantom: PhantomData {},
        }));
    }

    /// Sends a request message to the connected in-request channel of other actors.
    pub fn send_request(&self, msg: Request) {
        let (reply_sender, reply_receiver) = tokio::sync::oneshot::channel();
        let msg = RequestWithReplyChannel {
            request: msg,
            reply_channel: Linear::new(reply_sender),
        };
        self.connection_register.send(msg);

        let sender = self.sender.clone();
        let name = self.name.clone();

        tokio::spawn(async move {
            match reply_receiver.await {
                Ok(r) => match sender.send(M::new(name, r)) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Error sending request: {:?}", e);
                    }
                },
                Err(e) => {
                    warn!("Reply receiver error: {:?}", e);
                }
            };
        });
    }
}

/// An empty request hub - used for actors that do not have any request channels.
#[derive(Debug, Clone, Default)]
pub struct NullOutRequests {}

impl<M: IsInboundMessage> IsOutRequestHub<M> for NullOutRequests {
    fn from_parent_and_sender(
        _actor_name: &str,
        _sender: &tokio::sync::mpsc::UnboundedSender<M>,
    ) -> Self {
        Self {}
    }
}

impl HasActivate for NullOutRequests {
    fn extract(&mut self) -> Self {
        Self {}
    }

    fn activate(&mut self) {}
}
