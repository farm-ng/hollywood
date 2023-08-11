use std::marker::PhantomData;
use std::sync::Arc;

use crate::compute::{CancelRequest, Pipeline, Topology};
use crate::core::{
    DormantActorNode, InboundChannel, InboundMessage, OutboundChannel, OutboundConnection,
};

/// The context of the compute graph which is used to configure the network topology.
///
/// It is an opaque type created by the Context::configure() method.
pub struct Context {
    pub(crate) actors: Vec<Box<dyn DormantActorNode + Send>>,
    pub(crate) topology: Topology,
    pub(crate) cancel_request_request_inbound: InboundChannel<bool, CancelRequest>,
    pub(crate) cancel_request_receiver: tokio::sync::mpsc::Receiver<CancelRequest>,
}

impl Context {
    const CONTEXT_NAME: &str = "CONTEXT";

    /// Create a new context.
    ///
    /// This is the main entry point to configure the compute graph. The network topology is defined
    /// by the user within the callback function.
    pub fn configure(callback: &mut dyn FnMut(&mut Context)) -> Pipeline {
        let mut context = Context::new();
        callback(&mut context);
        Pipeline::from_context(context)
    }

    /// Registers an outbound channel for cancel request.
    ///
    /// Upon receiving a cancel request the registered outbound channel, the execution of the
    /// pipeline will be stopped.
    pub fn register_cancel_requester(&mut self, outbound: &mut OutboundChannel<()>) {
        outbound
            .connection_register
            .push(Arc::new(OutboundConnection {
                sender: self.cancel_request_request_inbound.sender.clone(),
                inbound_channel: self.cancel_request_request_inbound.name.clone(),
                phantom: PhantomData {},
            }));
    }

    fn new() -> Self {
        let (exit_request_sender, cancel_request_receiver) = tokio::sync::mpsc::channel(1);
        Self {
            actors: vec![],
            topology: Topology::new(),
            cancel_request_request_inbound: InboundChannel::<bool, CancelRequest> {
                name: CancelRequest::CANCEL_REQUEST_INBOUND_CHANNEL .to_owned(),
                actor_name: Self::CONTEXT_NAME.to_owned(),
                sender: exit_request_sender,
                phantom: std::marker::PhantomData {},
            },
            cancel_request_receiver,
        }
    }

    pub(crate) fn add_new_unique_name(&mut self, name_hint: String) -> String {
        self.topology.add_new_unique_name(name_hint)
    }

    pub(crate) fn assert_unique_inbound_name(
        &mut self,
        unique_inbound_name: String,
        actor_name: &str,
    ) {
        self.topology
            .assert_unique_inbound_name(unique_inbound_name, actor_name)
    }

    pub(crate) fn assert_unique_outbound_name(
        &mut self,
        unique_outbound_name: String,
        actor_name: &str,
    ) {
        self.topology
            .assert_unique_outbound_name(unique_outbound_name, actor_name);
    }

    pub(crate) fn connect_impl<
        T: Default + Clone + std::fmt::Debug + Sync + Send + 'static,
        M: InboundMessage,
    >(
        &mut self,
        outbound: &mut OutboundChannel<T>,
        inbound: &mut InboundChannel<T, M>,
    ) {
        self.topology.connect(outbound, inbound);
    }
}
