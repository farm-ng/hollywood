use crate::prelude::*;
use std::fmt::Debug;
use std::fmt::Display;

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

impl<T: Default + Debug + Display + Clone + Sync + Send + 'static> HasOnMessage
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

impl<T: Default + Debug + Display + Clone + Sync + Send + 'static> IsInboundMessageNew<T>
    for PrinterInboundMessage<T>
{
    fn new(_inbound_name: String, msg: T) -> Self {
        PrinterInboundMessage::Printable(msg)
    }
}

/// Printer actor.
pub type Printer<T> = Actor<PrinterProp, PrinterInbound<T>, NullState, NullOutbound, NullRequest>;

impl<T: Clone + Sync + Default + Send + 'static + Debug + Display>
    HasFromPropState<
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
