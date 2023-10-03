use std::fmt::{Debug, Display};

use crate::core::{
    Actor, ActorBuilder, DefaultRunner, FromPropState, InboundChannel, InboundHub, InboundMessage,
    InboundMessageNew, NullOutbound, NullState, OnMessage, Value,
};

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

impl Value for PrinterProp {}

/// Inbound message for the printer actor.
#[derive(Clone, Debug)]
pub enum PrinterInboundMessage<T: Display + Clone + Sync + Send + 'static> {
    /// Printable message.
    Printable(T),
}

impl<T: Debug + Display + Clone + Sync + Send + 'static> OnMessage for PrinterInboundMessage<T> {
    fn on_message(&self, prop: &PrinterProp, _state: &mut Self::State, _outputs: &Self::OutboundHub) {
        match self {
            PrinterInboundMessage::Printable(printable) => {
                println!("{}: {}", prop.topic, printable);
            }
        }
    }
}

impl<T: Debug + Display + Clone + Sync + Send + 'static> InboundMessageNew<T>
    for PrinterInboundMessage<T>
{
    fn new(_inbound_name: String, msg: T) -> Self {
        PrinterInboundMessage::Printable(msg)
    }
}

/// Generic printer actor.
pub type Printer<T> = Actor<PrinterProp, PrinterInbound<T>, NullState, NullOutbound>;

impl<T: Clone + Sync + Send + 'static + Debug + Display + Default>
    FromPropState<
        PrinterProp,
        PrinterInbound<T>,
        NullState,
        NullOutbound,
        PrinterInboundMessage<T>,
        DefaultRunner<PrinterProp, PrinterInbound<T>, NullState, NullOutbound>,
    > for Printer<T>
{
    fn name_hint(prop: &PrinterProp) -> String {
        format!("Printer({})", prop.topic)
    }
}

/// Builder for the generic printer.
pub struct PrinterInbound<T: Debug + Display + Clone + Sync + Send + 'static> {
    /// Inbound channel to receive printable messages.
    pub printable: InboundChannel<T, PrinterInboundMessage<T>>,
}

impl<T: Debug + Display + Clone + Sync + Send + 'static> InboundMessage
    for PrinterInboundMessage<T>
{
    type Prop = PrinterProp;
    type State = NullState;
    type OutboundHub = NullOutbound;

    fn inbound_channel(&self) -> String {
        match self {
            PrinterInboundMessage::Printable(_) => "Printable".to_owned(),
        }
    }
}

impl<T: Clone + Debug + Display + Default + Sync + Send + 'static>
    InboundHub<PrinterProp, NullState, NullOutbound, PrinterInboundMessage<T>>
    for PrinterInbound<T>
{
    fn from_builder(
        builder: &mut ActorBuilder<PrinterProp, NullState, NullOutbound, PrinterInboundMessage<T>>,
        actor_name: &str,
    ) -> Self {
        let m = InboundChannel::new(
            builder.context,
            actor_name.clone(),
            &builder.sender,
            PrinterInboundMessage::Printable(T::default()).inbound_channel(),
        );
        builder.forward.insert(m.name.clone(), Box::new(m.clone()));

        PrinterInbound { printable: m }
    }
}
