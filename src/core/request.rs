use std::fmt::Debug;
use std::{marker::PhantomData, sync::Arc};

use crate::compute::Context;

use super::connection::request_connection::RequestConnection;
use super::connection::RequestConnectionEnum;
use super::{InboundChannel, InboundMessage, InboundMessageNew, Morph};

/// A request hub is used to send requests to other actors.
pub trait RequestHub<M: InboundMessage>: Send + Sync + 'static + Morph {
    /// Create a new request hub for an actor.
    fn from_context_and_parent(actor_name: &str, sender: &tokio::sync::mpsc::Sender<M>) -> Self;
}

/// A request message with a reply channel.
#[derive(Debug, Clone, Default)]
pub struct RequestMessage<Request, Reply> {
    request: Request,
    reply_channel: Option<Arc<tokio::sync::oneshot::Sender<ReplyMessage<Reply>>>>,
}

impl<Request, Reply: Debug> RequestMessage<Request, Reply> {
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
        let reply_channel = Arc::into_inner(
            self.reply_channel
                .expect("self.reply must not be None. This is a bug in the hollywood crate."),
        )
        .expect("self.reply must have a ref count of 1. This is a bug in the hollywood crate.");
        ReplyLater::<Request, Reply> {
            request: self.request,
            reply_channel,
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

/// OutboundChannel is a connections for messages which are sent to a downstream actor.
pub struct RequestChannel<Request, Reply, M: InboundMessage> {
    /// Unique name of the outbound.
    pub name: String,
    /// Name of the actor that sends the outbound messages.
    pub actor_name: String,
    pub(crate) connection_register: RequestConnectionEnum<RequestMessage<Request, Reply>>,

    pub(crate) sender: tokio::sync::mpsc::Sender<M>,
}

impl<Request, Reply, M: InboundMessage> Morph for RequestChannel<Request, Reply, M> {
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
        M: InboundMessageNew<ReplyMessage<Reply>>,
    > RequestChannel<Request, Reply, M>
{
    /// Create a new outbound for actor in provided context.    
    pub fn new(name: String, actor_name: &str, sender: &tokio::sync::mpsc::Sender<M>) -> Self {
        Self {
            name: name.clone(),
            actor_name: actor_name.to_owned(),
            connection_register: RequestConnectionEnum::new(),
            sender: sender.clone(),
        }
    }

    /// Connect the outbound channel from this actor to the inbound channel of another actor.
    pub fn connect<Me: InboundMessageNew<RequestMessage<Request, Reply>>>(
        &mut self,
        _ctx: &mut Context,
        inbound: &mut InboundChannel<RequestMessage<Request, Reply>, Me>,
    ) {
        self.connection_register.push(Arc::new(RequestConnection {
            sender: inbound.sender.clone(),
            inbound_channel: inbound.name.clone(),
            phantom: PhantomData {},
        }));
    }

    /// Send a message to the connected inbound channels to other actors.
    pub fn send_request(&self, msg: Request) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let msg = RequestMessage {
            request: msg,
            reply_channel: Some(Arc::new(sender)),
        };
        self.connection_register.send(msg);

        let sender = self.sender.clone();
        let name = self.name.clone();

        tokio::spawn(async move {
            let r = receiver.await.unwrap();
            sender.send(M::new(name, r)).await
        });
    }
}

/// An empty request hub - used for actors that do not have any request channels.
#[derive(Debug, Clone, Default)]
pub struct NullRequest {}

impl<M: InboundMessage> RequestHub<M> for NullRequest {
    fn from_context_and_parent(_actor_name: &str, _sender: &tokio::sync::mpsc::Sender<M>) -> Self {
        Self {}
    }
}

impl Morph for NullRequest {
    fn extract(&mut self) -> Self {
        Self {}
    }

    fn activate(&mut self) {}
}