use std::sync::Arc;

use async_trait::async_trait;

use crate::compute::context::Context;
use crate::core::{
    actor::{DynActiveActor, DynActor, DynDormantActor, GenericActor},
    inbound::{NullInbounds, NullMessage, ForwardMessage},
    outbound::{ConnectionEnum, OutboundDistributionTrait, Morph, OutboundChannel},
    runner::RunnerTrait,
    state::StateTrait,
};

/// A periodic actor.
///
/// This is an actor that periodically sends a message to its outbound.
pub type PeriodicActor =
    GenericActor<NullInbounds, PeriodicActorState, PeriodicActorOutbounds, PeriodicRunner>;

impl PeriodicActor {
    /// Create a new periodic actor, with a period of `period` seconds.
    pub fn new_with_period(context: &mut Context, period: f64) -> PeriodicActor {
        PeriodicActor::new_with_state(
            context,
            PeriodicActorState {
                period,
                count: 0,
                time_elapsed: 0.0,
            },
        )
    }
}

impl
    DynActor<
        NullInbounds,
        PeriodicActorState,
        PeriodicActorOutbounds,
        NullMessage<PeriodicActorState, PeriodicActorOutbounds>,
        PeriodicRunner,
    > for PeriodicActor
{
    fn name_hint() -> String {
        "PeriodicActor".to_owned()
    }
}

/// State for a periodic actor.
#[derive(Clone, Debug)]
pub struct PeriodicActorState {
    period: f64,
    count: u32,
    time_elapsed: f64,
}

impl Default for PeriodicActorState {
    fn default() -> Self {
        Self {
            period: 1.0,
            count: 0,
            time_elapsed: 0.0,
        }
    }
}

impl StateTrait for PeriodicActorState {}

/// OutboundChannel OutboundDistribution center for the periodic actor, which consists of a single outbound channel.
pub struct PeriodicActorOutbounds {
    /// Heart beat outbound channel, which sends a messages every `period` seconds.
    pub heart_beat: OutboundChannel<f64>,
}

impl Morph for PeriodicActorOutbounds {
    fn extract(&mut self) -> Self {
        Self {
            heart_beat: self.heart_beat.extract(),
        }
    }

    fn activate(&mut self) {
        self.heart_beat.activate();
    }
}

impl OutboundDistributionTrait for PeriodicActorOutbounds {
    fn from_context_and_parent(context: &mut Context, actor_name: &str) -> Self {
        Self {
            heart_beat: OutboundChannel::<f64>::new(context, "heart_beat".to_owned(), actor_name),
        }
    }
}

/// The custom runner for the periodic actor.
pub struct PeriodicRunner {}

impl
    RunnerTrait<
        NullInbounds,
        PeriodicActorState,
        PeriodicActorOutbounds,
        NullMessage<PeriodicActorState, PeriodicActorOutbounds>,
    > for PeriodicRunner
{
    /// Create a new dormant actor.
    fn new_dormant_actor(
        name: String,
        state: PeriodicActorState,
        _receiver: tokio::sync::mpsc::Receiver<
            NullMessage<PeriodicActorState, PeriodicActorOutbounds>,
        >,
        _forward: std::collections::HashMap<
            String,
            Box<
                dyn ForwardMessage<
                        PeriodicActorState,
                        PeriodicActorOutbounds,
                        NullMessage<PeriodicActorState, PeriodicActorOutbounds>,
                    > + Send
                    + Sync,
            >,
        >,
        outbound: PeriodicActorOutbounds,
    ) -> Box<dyn DynDormantActor + Send + Sync> {
        Box::new(DormantPeriodicActor {
            name: name.clone(),
            init_state: state.clone(),
            outbound,
        })
    }
}

/// A dormant periodic actor.
pub struct DormantPeriodicActor {
    name: String,
    init_state: PeriodicActorState,
    outbound: PeriodicActorOutbounds,
}

impl DynDormantActor for DormantPeriodicActor {
    fn activate(mut self: Box<Self>) -> Box<dyn DynActiveActor + Send> {
        self.outbound.activate();
        Box::new(ActivePeriodicActor {
            name: self.name.clone(),
            init_state: self.init_state.clone(),
            state: None,
            outbound: Arc::new(self.outbound),
        })
    }
}

/// An active periodic actor.
pub struct ActivePeriodicActor {
    name: String,
    init_state: PeriodicActorState,
    state: Option<PeriodicActorState>,
    outbound: Arc<PeriodicActorOutbounds>,
}

#[async_trait]
impl DynActiveActor for ActivePeriodicActor {
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
            (1000.0 * state.period) as u64,
        ));

        let conns = Arc::new(self.outbound.clone());

        loop {
            interval.tick().await;
            if kill.try_recv().is_ok() {
                break;
            }
            state.count += 1;

            if state.count > 100 {
                break;
            }
            state.time_elapsed += interval.period().as_secs_f64();
            println!("tick: {:?}, {:?}", state.count, state.time_elapsed);

            let conns = conns.clone();

            match &conns.heart_beat.connection_register {
                ConnectionEnum::Config(_) => {
                    panic!("Cannot extract config connection")
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
