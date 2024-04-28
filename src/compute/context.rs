use std::marker::PhantomData;
use std::sync::Arc;

use crate::compute::topology::Topology;
use crate::core::outbound::OutboundConnection;
use crate::prelude::*;

/// The context of the compute graph which is used to configure the network topology.
///
/// It is an opaque type created by the Hollywood::configure() method.
pub struct Hollywood {
    pub(crate) actors: Vec<Box<dyn IsActorNode + Send>>,
    pub(crate) topology: Topology,
    pub(crate) cancel_request_sender_template: tokio::sync::mpsc::UnboundedSender<CancelRequest>,
    pub(crate) cancel_request_receiver: tokio::sync::mpsc::UnboundedReceiver<CancelRequest>,
}

impl Hollywood {
    /// Create a new Hollywood context.
    ///
    /// This is the main entry point to configure the compute graph. The network topology is defined
    /// by the user within the callback function.
    ///
    /// Example:
    ///
    /// ```rust
    /// # use hollywood::actors::printer::PrinterProp;
    /// # use hollywood::actors::Nudge;
    /// # use hollywood::actors::Printer;
    /// # use hollywood::prelude::*;
    /// let pipeline = Hollywood::configure(&mut |context| {
    ///     let mut nudge = Nudge::<String>::new(context, "nudge".to_owned());
    ///     let mut nudge_printer = Printer::<String>::from_prop_and_state(
    ///         context,
    ///         PrinterProp {
    ///             topic: "nudge: ".to_string(),
    ///         },
    ///         NullState::default(),
    ///     );
    ///     nudge
    ///         .outbound
    ///         .nudge
    ///         .connect(context, &mut nudge_printer.inbound.printable);
    /// });
    /// ```
    pub fn configure(callback: &mut dyn FnMut(&mut Hollywood)) -> Pipeline {
        let mut context = Hollywood::new();
        callback(&mut context);
        Pipeline::from_context(context)
    }

    /// Returns a sender to send cancel requests to the pipeline.
    pub fn get_cancel_request_sender(
        &mut self,
    ) -> tokio::sync::mpsc::UnboundedSender<CancelRequest> {
        self.cancel_request_sender_template.clone()
    }

    /// Registers an outbound channel to receive cancel requests from.
    ///
    /// Upon receiving a cancel request, the execution of the pipeline will be stopped.
    pub fn register_cancel_requester(&mut self, outbound: &mut OutboundChannel<CancelRequest>) {
        outbound
            .connection_register
            .push(Arc::new(OutboundConnection {
                sender: self.cancel_request_sender_template.clone(),
                inbound_channel: CancelRequest::CANCEL_REQUEST_INBOUND_CHANNEL.to_owned(),
                phantom: PhantomData {},
            }));
    }

    fn new() -> Self {
        let (cancel_request_sender_template, cancel_request_receiver) =
            tokio::sync::mpsc::unbounded_channel();
        Self {
            actors: vec![],
            topology: Topology::new(),
            cancel_request_sender_template,
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
        T0: Clone + std::fmt::Debug + Sync + Send + 'static,
        T1: Clone + std::fmt::Debug + Sync + Send + 'static,
        M: IsInboundMessage,
    >(
        &mut self,
        outbound: &mut OutboundChannel<T0>,
        inbound: &mut InboundChannel<T1, M>,
    ) {
        self.topology.connect(outbound, inbound);
    }
}
