use crate::prelude::*;
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
    NullInRequests,
    NullState,
    NudgeOutbound<Item>,
    NullOutRequests,
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
        NullInRequests,
        NullState,
        NudgeOutbound<Item>,
        NullMessage,
        NullInRequestMessage,
        NullOutRequests,
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
    IsRunner<
        NudgeProp<Item>,
        NullInbound,
        NullInRequests,
        NullState,
        NudgeOutbound<Item>,
        NullOutRequests,
        NullMessage,
        NullInRequestMessage,
    > for NudgeRunner
{
    /// Create a new actor node.
    fn new_actor_node(
        name: String,
        prop: NudgeProp<Item>,
        state: NullState,
        forward_receiver_outbound: (
            std::collections::HashMap<
                String,
                Box<
                    dyn HasForwardMessage<
                            NudgeProp<Item>,
                            NullState,
                            NudgeOutbound<Item>,
                            NullOutRequests,
                            NullMessage,
                        > + Send
                        + Sync,
                >,
            >,
            tokio::sync::mpsc::UnboundedReceiver<NullMessage>,
            NudgeOutbound<Item>,
        ),
        _forward_receiver_request: (
            std::collections::HashMap<
                String,
                Box<
                    dyn HasForwardRequestMessage<
                            NudgeProp<Item>,
                            NullState,
                            NudgeOutbound<Item>,
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
        Box::new(NudgeActor::<Item> {
            name: name.clone(),
            prop,
            init_state: state.clone(),
            state: None,
            outbound: Some(forward_receiver_outbound.2),
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
                    i.send_impl(self.prop.item.clone());
                }
            }
        }
    }

    fn on_exit(&mut self) {
        // Do nothing
    }
}
