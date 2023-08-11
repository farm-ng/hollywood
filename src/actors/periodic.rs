use std::sync::Arc;

use async_trait::async_trait;

use crate::compute::context::Context;
use crate::core::{
    actor::{ActorFacade, ActorNode, DormantActorNode, GenericActor},
    inbound::{ForwardMessage, NullInbound, NullMessage},
    outbound::{ConnectionEnum, Morph, OutboundChannel, OutboundHub},
    runner::Runner,
    value::Value,
};

/// A periodic actor.
///
/// This is an actor that periodically sends a message to its outbound.
pub type Periodic =
    GenericActor<PeriodicProp, NullInbound, PeriodicState, PeriodicOutbound, PeriodicRunner>;

impl Periodic {
    /// Create a new periodic actor, with a period of `period` seconds.
    pub fn new_with_period(context: &mut Context, period: f64) -> Periodic {
        Periodic::new_with_state(
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
    ActorFacade<
        PeriodicProp,
        NullInbound,
        PeriodicState,
        PeriodicOutbound,
        NullMessage<PeriodicProp, PeriodicState, PeriodicOutbound>,
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

impl Value for PeriodicProp {}

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

impl Value for PeriodicState {}

/// Outbound hub of periodic actor, which consists of a single outbound channel.
pub struct PeriodicOutbound {
    /// Time stamp outbound channel, which sends a messages every `period`
    /// seconds with the current time stamp.
    pub time_stamp: OutboundChannel<f64>,
}

impl Morph for PeriodicOutbound {
    fn extract(&mut self) -> Self {
        Self {
            time_stamp: self.time_stamp.extract(),
        }
    }

    fn activate(&mut self) {
        self.time_stamp.activate();
    }
}

impl OutboundHub for PeriodicOutbound {
    fn from_context_and_parent(context: &mut Context, actor_name: &str) -> Self {
        Self {
            time_stamp: OutboundChannel::<f64>::new(context, "time_stamp".to_owned(), actor_name),
        }
    }
}

/// The custom runner for the periodic actor.
pub struct PeriodicRunner {}

impl
    Runner<
        PeriodicProp,
        NullInbound,
        PeriodicState,
        PeriodicOutbound,
        NullMessage<PeriodicProp, PeriodicState, PeriodicOutbound>,
    > for PeriodicRunner
{
    /// Create a new dormant actor.
    fn new_dormant_actor(
        name: String,
        prop: PeriodicProp,
        state: PeriodicState,
        _receiver: tokio::sync::mpsc::Receiver<
            NullMessage<PeriodicProp, PeriodicState, PeriodicOutbound>,
        >,
        _forward: std::collections::HashMap<
            String,
            Box<
                dyn ForwardMessage<
                        PeriodicProp,
                        PeriodicState,
                        PeriodicOutbound,
                        NullMessage<PeriodicProp, PeriodicState, PeriodicOutbound>,
                    > + Send
                    + Sync,
            >,
        >,
        outbound: PeriodicOutbound,
    ) -> Box<dyn DormantActorNode + Send + Sync> {
        Box::new(DormantPeriodic {
            name: name.clone(),
            prop,
            init_state: state.clone(),
            outbound,
        })
    }
}

/// The dormant periodic actor.
pub struct DormantPeriodic {
    name: String,
    prop: PeriodicProp,
    init_state: PeriodicState,
    outbound: PeriodicOutbound,
}

impl DormantActorNode for DormantPeriodic {
    fn activate(mut self: Box<Self>) -> Box<dyn ActorNode + Send> {
        self.outbound.activate();
        Box::new(ActivePeriodic {
            name: self.name.clone(),
            prop: self.prop.clone(),
            init_state: self.init_state.clone(),
            state: None,
            outbound: Arc::new(self.outbound),
        })
    }
}

/// The active periodic actor.
pub struct ActivePeriodic {
    name: String,
    prop: PeriodicProp,
    init_state: PeriodicState,
    state: Option<PeriodicState>,
    outbound: Arc<PeriodicOutbound>,
}

#[async_trait]
impl ActorNode for ActivePeriodic {
    fn name(&self) -> &String {
        &self.name
    }

    fn reset(&mut self) {
        self.state = Some(self.init_state.clone());
    }

    async fn run(&mut self, mut kill: tokio::sync::broadcast::Receiver<()>) {
        self.reset();

        let state = self.state.as_mut().unwrap();

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
            (1000.0 * self.prop.period) as u64,
        ));

        let conns = Arc::new(self.outbound.clone());

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
}
