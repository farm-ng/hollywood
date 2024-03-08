use std::fmt::Debug;
use std::fmt::Display;

use hollywood_macros::actor_inputs;

use crate::core::request::NullRequest;
use crate::core::Actor;
use crate::core::ActorBuilder;
use crate::core::DefaultRunner;
use crate::core::FromPropState;
use crate::core::InboundChannel;
use crate::core::InboundHub;
use crate::core::InboundMessage;
use crate::core::InboundMessageNew;
use crate::core::NullOutbound;
use crate::core::NullState;
use crate::core::OnMessage;

/// Configuration properties for the printer actor.
#[derive(Clone, Debug)]
pub struct PrinterProp {
    /// Topic to print. It will be printed before the message.
    pub topic: String,
}

impl Default for PrinterProp {
    fn default() -> Self {
        PrinterProp {
            topic: "generic".to_owned(),
        }
    }
}

/// Inbound message for the printer actor.
#[derive(Clone, Debug)]
#[actor_inputs(PrinterInbound<T>, {PrinterProp, NullState, NullOutbound, NullRequest})]
pub enum PrinterInboundMessage<T: Default + Debug + Display + Clone + Sync + Send + 'static> {
    /// Printable message.
    Printable(T),
}

impl<T: Default + Debug + Display + Clone + Sync + Send + 'static> OnMessage
    for PrinterInboundMessage<T>
{
    fn on_message(
        self,
        prop: &PrinterProp,
        _state: &mut Self::State,
        _outputs: &Self::OutboundHub,
        _request: &Self::RequestHub,
    ) {
        match self {
            PrinterInboundMessage::Printable(printable) => {
                println!("{}: {}", prop.topic, printable);
            }
        }
    }
}

impl<T: Default + Debug + Display + Clone + Sync + Send + 'static> InboundMessageNew<T>
    for PrinterInboundMessage<T>
{
    fn new(_inbound_name: String, msg: T) -> Self {
        PrinterInboundMessage::Printable(msg)
    }
}

/// Printer actor.
pub type Printer<T> = Actor<PrinterProp, PrinterInbound<T>, NullState, NullOutbound, NullRequest>;

impl<T: Clone + Sync + Default + Send + 'static + Debug + Display>
    FromPropState<
        PrinterProp,
        PrinterInbound<T>,
        NullState,
        NullOutbound,
        PrinterInboundMessage<T>,
        NullRequest,
        DefaultRunner<PrinterProp, PrinterInbound<T>, NullState, NullOutbound, NullRequest>,
    > for Printer<T>
{
    fn name_hint(prop: &PrinterProp) -> String {
        format!("Printer({})", prop.topic)
    }
}
