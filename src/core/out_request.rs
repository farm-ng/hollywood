use crate::core::connection::request_connection::RequestConnection;
use crate::core::connection::RequestConnectionEnum;
use crate::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

/// A request hub is used to send requests to other actors which will reply later.
pub trait IsOutRequestHub<M: IsInboundMessage>: Send + Sync + 'static + HasActivate {
    /// Create a new request hub for an actor.
    fn from_parent_and_sender(
        actor_name: &str,
        sender: &tokio::sync::mpsc::UnboundedSender<M>,
    ) -> Self;
}

/// A request message with a reply channel.
#[derive(Debug)]
pub struct RequestWithReplyChannel<Request, Reply> {
    /// The request.
    pub request: Request,
    /// The reply channel.
    pub reply_channel: tokio::sync::oneshot::Sender<ReplyMessage<Reply>>,
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
    /// Reply to the request immediately.
    pub fn reply<F>(self, func: F)
    where
        F: FnOnce(Request) -> Reply,
    {
        let reply_struct = self.reply_later();
        let reply = func(reply_struct.request);
        reply_struct
            .reply_channel
            .send(ReplyMessage { reply })
            .unwrap();
    }

    /// Reply to the request later using the provided reply channel.
    pub fn reply_later(self) -> ReplyLater<Request, Reply> {
        ReplyLater::<Request, Reply> {
            request: self.request,
            reply_channel: self.reply_channel,
        }
    }
}

/// A request with a reply channel.
pub struct ReplyLater<Request, Reply> {
    /// The request.
    pub request: Request,
    /// The reply channel.
    pub reply_channel: tokio::sync::oneshot::Sender<ReplyMessage<Reply>>,
}

impl<Request, Reply: Debug> ReplyLater<Request, Reply> {
    /// Send the reply to the request.
    pub fn send_reply(self, reply: Reply) {
        self.reply_channel.send(ReplyMessage { reply }).unwrap();
    }
}

/// A reply to a request.
#[derive(Debug, Clone, Default)]
pub struct ReplyMessage<Reply> {
    /// The reply value.
    pub reply: Reply,
}

/// OutRequestChannel is a connections for messages which are sent to a downstream actor.
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
    /// Create a new request channel for actor in provided context.    
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

    /// Connect the request channel from this actor to the inbound channel of another actor.
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

    /// Send a message to the connected inbound channels of other actors.
    pub fn send_request(&self, msg: Request) {
        let (reply_sender, reply_receiver) = tokio::sync::oneshot::channel();
        let msg = RequestWithReplyChannel {
            request: msg,
            reply_channel: reply_sender,
        };
        self.connection_register.send(msg);

        let sender = self.sender.clone();
        let name = self.name.clone();

        tokio::spawn(async move {
            let r = match reply_receiver.await {
                Ok(r) => r,
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            };
            match sender.send(M::new(name, r)) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            }
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
