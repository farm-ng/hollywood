use crate::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;

/// A periodic actor.
///
/// This is an actor that periodically sends a message to its outbound.
pub type Periodic = GenericActor<
    PeriodicProp,
    NullInbound,
    NullInRequests,
    PeriodicState,
    PeriodicOutbound,
    NullOutRequests,
    PeriodicRunner,
>;

impl Periodic {
    /// Create a new periodic actor, with a period of `period` seconds.
    pub fn new_with_period(context: &mut Hollywood, period: f64) -> Periodic {
        Periodic::from_prop_and_state(
            context,
            PeriodicProp {
                period,
                ..Default::default()
            },
            PeriodicState {
                count: 0,
                time_elapsed: 0.0,
            },
        )
    }
}

impl
    HasFromPropState<
        PeriodicProp,
        NullInbound,
        NullInRequests,
        PeriodicState,
        PeriodicOutbound,
        NullMessage,
        NullInRequestMessage,
        NullOutRequests,
        PeriodicRunner,
    > for Periodic
{
    fn name_hint(_prop: &PeriodicProp) -> String {
        "Periodic".to_owned()
    }
}

/// Configuration properties for the periodic actor.
#[derive(Clone, Debug)]
pub struct PeriodicProp {
    period: f64,
    stop_time: f64,
}

impl Default for PeriodicProp {
    fn default() -> Self {
        Self {
            period: 1.0,
            stop_time: 24.0 * 60.0 * 60.0,
        }
    }
}

/// State of the periodic actor.
#[derive(Clone, Debug)]
pub struct PeriodicState {
    count: u32,
    time_elapsed: f64,
}

impl Default for PeriodicState {
    fn default() -> Self {
        Self {
            count: 0,
            time_elapsed: 0.0,
        }
    }
}

/// Outbound hub of periodic actor, which consists of a single outbound channel.
pub struct PeriodicOutbound {
    /// Time stamp outbound channel, which sends a messages every `period`
    /// seconds with the current time stamp.
    pub time_stamp: OutboundChannel<f64>,
}

impl HasActivate for PeriodicOutbound {
    fn extract(&mut self) -> Self {
        Self {
            time_stamp: self.time_stamp.extract(),
        }
    }

    fn activate(&mut self) {
        self.time_stamp.activate();
    }
}

impl IsOutboundHub for PeriodicOutbound {
    fn from_context_and_parent(context: &mut Hollywood, actor_name: &str) -> Self {
        Self {
            time_stamp: OutboundChannel::<f64>::new(context, "time_stamp".to_owned(), actor_name),
        }
    }
}

/// The custom runner for the periodic actor.
pub struct PeriodicRunner {}

impl
    IsRunner<
        PeriodicProp,
        NullInbound,
        NullInRequests,
        PeriodicState,
        PeriodicOutbound,
        NullOutRequests,
        NullMessage,
        NullInRequestMessage,
    > for PeriodicRunner
{
    /// Create a new actor node.
    fn new_actor_node(
        name: String,
        prop: PeriodicProp,
        state: PeriodicState,
        forward_receiver_outbound: (
            std::collections::HashMap<
                String,
                Box<
                    dyn HasForwardMessage<
                            PeriodicProp,
                            PeriodicState,
                            PeriodicOutbound,
                            NullOutRequests,
                            NullMessage,
                        > + Send
                        + Sync,
                >,
            >,
            tokio::sync::mpsc::UnboundedReceiver<NullMessage>,
            PeriodicOutbound,
        ),
        _forward_receiver_request: (
            std::collections::HashMap<
                String,
                Box<
                    dyn HasForwardRequestMessage<
                            PeriodicProp,
                            PeriodicState,
                            PeriodicOutbound,
                            NullOutRequests,
                            NullInRequestMessage,
                        > + Send
                        + Sync,
                >,
            >,
            tokio::sync::mpsc::UnboundedReceiver<NullInRequestMessage>,
            NullOutRequests,
        ),
        _on_exit_fn: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
    ) -> Box<dyn IsActorNode + Send + Sync> {
        Box::new(PeriodicActor {
            name: name.clone(),
            prop,
            init_state: state.clone(),
            state: None,
            outbound: Some(forward_receiver_outbound.2),
        })
    }
}

/// The active periodic actor.
pub struct PeriodicActor {
    name: String,
    prop: PeriodicProp,
    init_state: PeriodicState,
    state: Option<PeriodicState>,
    outbound: Option<PeriodicOutbound>,
}

#[async_trait]
impl IsActorNode for PeriodicActor {
    fn name(&self) -> &String {
        &self.name
    }

    async fn run(&mut self, mut kill: tokio::sync::broadcast::Receiver<()>) {
        let mut outbound = self.outbound.take().unwrap();
        outbound.activate();
        self.state = Some(self.init_state.clone());

        let state = self.state.as_mut().unwrap();

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
            (1000.0 * self.prop.period) as u64,
        ));

        let conns = Arc::new(outbound);

        loop {
            interval.tick().await;
            if kill.try_recv().is_ok() {
                break;
            }
            state.count += 1;

            if state.time_elapsed > self.prop.stop_time {
                break;
            }
            state.time_elapsed += interval.period().as_secs_f64();

            let conns = conns.clone();

            match &conns.time_stamp.connection_register {
                ConnectionEnum::Config(_) => {
                    panic!("Cannot extract connection config")
                }
                ConnectionEnum::Active(active) => {
                    for i in active.maybe_registers.as_ref().unwrap().iter() {
                        i.send_impl(state.time_elapsed);
                    }
                }
            }
        }
    }

    fn on_exit(&mut self) {
        // do nothing
    }
}
