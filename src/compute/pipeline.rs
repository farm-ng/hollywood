use crate::compute::topology::Topology;
use crate::prelude::*;
use std::mem::swap;
use tracing::{info, warn};

/// A message to cancel the pipeline.
#[derive(Clone, Debug)]
pub struct CancelRequest;

impl CancelRequest {
    pub(crate) const CANCEL_REQUEST_INBOUND_CHANNEL: &'static str = "CANCEL";
}

impl IsInboundMessage for CancelRequest {
    type Prop = NullProp;
    type State = NullState;
    type OutboundHub = NullOutbound;
    type OutRequestHub = NullOutbound;

    fn inbound_channel(&self) -> String {
        Self::CANCEL_REQUEST_INBOUND_CHANNEL.to_owned()
    }
}

impl IsInboundMessageNew<CancelRequest> for CancelRequest {
    fn new(_inbound_name: String, _: CancelRequest) -> Self {
        CancelRequest
    }
}

/// Compute pipeline, strictly speaking a DAG (directed acyclic graph) of actors. It is created by
/// the [Hollywood::configure()] method.
pub struct Pipeline {
    actors: Vec<Box<dyn IsActorNode + Send>>,
    topology: Topology,
    /// We have this here to keep receiver alive
    pub cancel_request_sender_template: Option<tokio::sync::mpsc::UnboundedSender<CancelRequest>>,
    cancel_request_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<CancelRequest>>,
}

impl Pipeline {
    pub(crate) fn from_context(context: Hollywood) -> Self {
        let mut active = vec![];
        for actor in context.actors.into_iter() {
            active.push(actor);
        }
        let compute_graph = Pipeline {
            actors: active,
            topology: context.topology,
            cancel_request_sender_template: Some(context.cancel_request_sender_template),
            cancel_request_receiver: Some(context.cancel_request_receiver),
        };
        compute_graph.topology.analyze_graph_topology();
        compute_graph
    }

    /// Returns a sender to send cancel requests to the pipeline.
    pub fn get_cancel_request_sender(&self) -> tokio::sync::mpsc::UnboundedSender<CancelRequest> {
        self.cancel_request_sender_template
            .as_ref()
            .unwrap()
            .clone()
    }

    /// Executes the compute graph.
    ///
    /// It consumes the self, starts  execution of the pipeline and returns a future (since it is
    /// an async function) that resolves to the pipeline itself. The future is completed when all
    /// actors have completed their execution.
    ///
    /// In particular, [IsActorNode::run()] is called for each actor in the pipeline in a dedicated
    /// tokio task. Hence, the actors run concurrently.
    ///
    /// TODO:
    ///   Document state of actors before during and after completion, and validate that this is
    ///   indeed the case.
    ///    - All actors are set to its initial state right when this method is called and before
    ///      the actual execution starts.
    ///    - All actors remain their current state when the execution is completed.
    ///    - Repeatable execution of the pipeline shall lead to comparable results.
    ///      
    pub async fn run(mut self) -> Self {
        info!("Pipeline started ...");

        // Set up Ctrl-C handler to cancel the pipeline
        //
        // TODO: Make this configurable, but keep it enabled by default.
        let cancel_requester = self.get_cancel_request_sender();
        ctrlc::set_handler(move || {
            cancel_requester
                .send(CancelRequest)
                .expect("Error sending Ctrl-C triggered cancel request");
        })
        .expect("Error setting Ctrl-C handler");

        let (kill_sender, _) = tokio::sync::broadcast::channel(10);

        let mut handles = vec![];
        let mut actors = vec![];
        let mut rxs = vec![];
        let mut cancel_request_receiver = self.cancel_request_receiver.take().unwrap();

        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel();

        let h_exit = tokio::spawn(async move {
            match cancel_request_receiver.recv().await {
                Some(_) => {
                    info!("Pipeline cancellation requested");
                    let _ = exit_tx.send(cancel_request_receiver);
                }
                None => {
                    warn!("Cancel request channel closed");
                }
            }
        });

        swap(&mut actors, &mut self.actors);
        for mut actor in actors {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let kill_receiver = kill_sender.subscribe();
            let h = tokio::spawn(async move {
                actor.run(kill_receiver).await;
                if tx.send(actor).is_err() {}
            });
            rxs.push(rx);

            handles.push(h);
        }
        match h_exit.await {
            Ok(_) => {}
            Err(err) => {
                warn!("Error in cancel request handler: {}", err);
            }
        }
        kill_sender.send(()).unwrap();
        for h in handles {
            h.await.unwrap();
        }

        let mut r = exit_rx.await.unwrap();

        while r.try_recv().is_ok() {}

        self.cancel_request_receiver = Some(r);

        for rx in rxs {
            match rx.await {
                Ok(a) => {
                    self.actors.push(a);
                }
                Err(err) => {
                    panic!("oh no, actor died: {}", err);
                }
            }
        }

        info!("Pipeline execution finished");
        self
    }

    /// Printers the flow graph of the compute graph.
    pub fn print_flow_graph(&self) {
        self.topology.print_flow_graph();
    }
}
