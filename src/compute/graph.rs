use std::mem::swap;

use crate::compute::{Context,Topology};
use crate::core::{DynActiveActor, InboundMessage, InboundMessageNew, NullOutbounds, NullState};

/// Message to request an exiting a running compute graph.
#[derive(Debug, Clone)]
pub enum CancelRequest {
    /// Request to exit the compute graph.
    Cancel(()),
}

impl CancelRequest {
    /// Unique name for cancel request inbound channel.
    pub const CANCEL_REQUEST_NAME: &str = "CANCEL";
}

impl InboundMessage for CancelRequest {
    type State = NullState;
    type OutboundDistribution = NullOutbounds;

    /// This messages is only meant to use for the exit request inbound of the context.
    /// Hence, the inbound name is the constant EXIT_REQUEST_NAMER.
    fn inbound_name(&self) -> String {
        Self::CANCEL_REQUEST_NAME.to_owned()
    }
}

impl InboundMessageNew<()> for CancelRequest {
    fn new(_inbound_name: String, _: ()) -> Self {
        CancelRequest::Cancel(())
    }
}

/// Compute graph, strictly speaking a DAG (directed acyclic graph) of actors.
pub struct ComputeGraph {
    actors: Vec<Box<dyn DynActiveActor + Send>>,
    topology: Topology,
    cancel_request_receiver: Option<tokio::sync::mpsc::Receiver<CancelRequest>>,
}

impl ComputeGraph {
    pub(crate) fn from_context(context: Context) -> Self {
        let mut active = vec![];
        for actor in context.actors.into_iter() {
            active.push(actor.activate());
        }
        let compute_graph = ComputeGraph {
            actors: active,
            topology: context.topology,
            cancel_request_receiver: Some(context.cancel_request_receiver),
        };
        compute_graph.topology.analyze_graph_topology();
        compute_graph
    }

    /// Executes the compute graph.
    pub async fn run(mut self) -> Self {
        println!("START NEW RUN");
        let (kill_sender, _) = tokio::sync::broadcast::channel(10);

        let mut handles = vec![];
        let mut actors = vec![];
        let mut rxs = vec![];
        let mut cancel_request_receiver = self.cancel_request_receiver.take().unwrap();

        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel();

        let h_exit = tokio::spawn(async move {
            let msg = cancel_request_receiver.recv().await.unwrap();
            match msg {
                CancelRequest::Cancel(_) => {
                    println!("Cancel requested");

                    let _ = exit_tx.send(cancel_request_receiver);
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
        h_exit.await.unwrap();
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

        println!("END RUN");
        self
    }
}
