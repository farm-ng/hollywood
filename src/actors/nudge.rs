use crate::core::runner::Runner;
use crate::prelude::*;
use crate::ConnectionEnum;
use crate::GenericActor;
use async_trait::async_trait;
use std::fmt::Debug;

/// Prop for the nudge actor.
#[derive(Clone, Debug, Default)]
pub struct NudgeProp<Item: Clone> {
    /// The attached item.
    pub item: Item,
}

/// A nudge actor.
///
/// All it does is to send a nudge message containing the item once.
pub type Nudge<Item> = GenericActor<
    NudgeProp<Item>,
    NullInbound,
    NullState,
    NudgeOutbound<Item>,
    NullRequest,
    NudgeRunner,
>;

impl<Item: Default + Sync + Send + Debug + 'static + Clone> Nudge<Item> {
    /// Create a new nudge actor
    pub fn new(context: &mut Hollywood, item: Item) -> Nudge<Item> {
        Nudge::from_prop_and_state(context, NudgeProp::<Item> { item }, NullState::default())
    }
}

impl<Item: Default + Sync + Send + Clone + Debug + 'static>
    HasFromPropState<
        NudgeProp<Item>,
        NullInbound,
        NullState,
        NudgeOutbound<Item>,
        NullMessage<NudgeProp<Item>, NullState, NudgeOutbound<Item>, NullRequest>,
        NullRequest,
        NudgeRunner,
    > for Nudge<Item>
{
    fn name_hint(_prop: &NudgeProp<Item>) -> String {
        "Nudge".to_owned()
    }
}

/// Nudge outbound hub
#[actor_outputs]
pub struct NudgeOutbound<Item: 'static + Default + Clone + Send + Sync + std::fmt::Debug> {
    /// Nudge outbound channel.
    pub nudge: OutboundChannel<Item>,
}

/// The custom nudge runner
pub struct NudgeRunner {}

impl<Item: Default + Sync + Send + Clone + Debug + 'static>
    Runner<
        NudgeProp<Item>,
        NullInbound,
        NullState,
        NudgeOutbound<Item>,
        NullRequest,
        NullMessage<NudgeProp<Item>, NullState, NudgeOutbound<Item>, NullRequest>,
    > for NudgeRunner
{
    /// Create a new actor node.
    fn new_actor_node(
        name: String,
        prop: NudgeProp<Item>,
        state: NullState,
        _receiver: tokio::sync::mpsc::Receiver<
            NullMessage<NudgeProp<Item>, NullState, NudgeOutbound<Item>, NullRequest>,
        >,
        _forward: std::collections::HashMap<
            String,
            Box<
                dyn HasForwardMessage<
                        NudgeProp<Item>,
                        NullState,
                        NudgeOutbound<Item>,
                        NullRequest,
                        NullMessage<NudgeProp<Item>, NullState, NudgeOutbound<Item>, NullRequest>,
                    > + Send
                    + Sync,
            >,
        >,
        outbound: NudgeOutbound<Item>,
        _request: NullRequest,
    ) -> Box<dyn IsActorNode + Send + Sync> {
        Box::new(NudgeActor::<Item> {
            name: name.clone(),
            prop,
            init_state: state.clone(),
            state: None,
            outbound: Some(outbound),
        })
    }
}

/// The nudge actor.
pub struct NudgeActor<Item: Clone + 'static + Default + Clone + Send + Sync + std::fmt::Debug> {
    name: String,
    prop: NudgeProp<Item>,
    init_state: NullState,
    state: Option<NullState>,
    outbound: Option<NudgeOutbound<Item>>,
}

#[async_trait]
impl<Item: 'static + Default + Clone + Send + Sync + std::fmt::Debug> IsActorNode
    for NudgeActor<Item>
{
    fn name(&self) -> &String {
        &self.name
    }

    async fn run(&mut self, mut _kill: tokio::sync::broadcast::Receiver<()>) {
        let mut outbound = self.outbound.take().unwrap();
        outbound.activate();
        self.state = Some(self.init_state.clone());

        match &outbound.nudge.connection_register {
            ConnectionEnum::Config(_) => {
                panic!("Cannot extract connection config")
            }
            ConnectionEnum::Active(active) => {
                for i in active.maybe_registers.as_ref().unwrap().iter() {
                    println!("NudgeActor: sending");
                    i.send_impl(self.prop.item.clone());
                }
            }
        }
    }
}
