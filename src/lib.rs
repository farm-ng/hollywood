#![deny(missing_docs)]
//! # Hollywood
//! 
//! Hollywood is an actor framework for Rust.
//! 
//! Actors are stateful entities that communicate with each other by sending messages. An actor
//! actor receives streams of messages through its inbound channels, processes them and sends
//! messages to other actors through its outbound channels. Actors are either stateless so that
//! its outbound streams are a pure function of its inbound streams, or have an internal state which
//! is updated by incoming messages and may influences the content of its outbound messages.
//! 
//! Actors are arranged in a compute pipeline (or a directed acyclic graph to be more precise). The
//! first layer of the graph consists of a set of one or more source actors whose outbound streams
//! are fed by an external resource. A typical example of a source actors, include sensor drivers,
//! or log file readers. Terminal notes in the pipeline are actors which have either no outbound
//! channels or which outbound channels are not connected. Terminal actors are often sink notes
//! which feed data to an external resource. Example of a sink actor are robot manipulators,
//! log file writer or visualization components.
//! 
//! ## Module Overview
//! 
//! The library is organized in the following modules:
//! 
//! - The [macros] module contains the three macros that are used to define an actors with
//!   minimal boilerplate.
//! - The [core] module contains the core structs and traits of the library. [Actor](core::Actor) 
//!   is a generic struct that represents an actor. [InboundHub](core::InboundHub) is the trait
//!   which represents the collection of inbound channels of an actor. Similarly,
//!   [OutboundHub](core::OutboundHub) is the trait which represents the collection of outbound
//!   channels of an actor.
//! 
//!   Most importantly, [OnMessage](core::OnMessage) is the main entry point for user code and sets
//!   the behavior of a user-defines actor. [OnMessage::on_message()](core::OnMessage::on_message())
//!   processes incoming messages, updates the actor's state and sends messages to downstream actors
//!   in the pipeline.
//! 
//! - The [compute] module contains the [Context](compute::Context) and 
//!   [Pipeline](compute::Pipeline) which are used to configure a set of actors, connect 
//!   them into a graph and to execute flow.
//! 
//! ## Example: moving average
//! 
//! We will show how to implement a moving average actor that computes the moving
//! average of a stream of numbers. The complete code for this example can be found in the
//! [examples/moving_average.rs].
//! 
//! ### Outbound hub
//! 
//! First we define the outbound hub of the actor:
//! 
//! ```ignore
//! #[actor_outputs]
//! pub struct MovingAverageOutbound {
//!     /// Running average of the input stream.
//!     pub average: OutboundChannel<f64>,
//! }
//! ```
//! 
//! It contains of a single outbound channels, to send the calculated average to the outside world.
//! 
//! ### Properties and state
//! 
//! Next we define the immutable properties and the internal state of the actor:
//! 
//! ```ignore
//! /// Properties the MovingAverage actor.
//! #[derive(Clone, Debug)]
//! pub struct MovingAverageProp {
//!     /// alpha value for the moving average
//!     pub alpha: f64,
//! }
//! 
//! impl Default for MovingAverageProp {
//!     fn default() -> Self {
//!         MovingAverageProp {
//!             alpha: 0.5,
//!         }
//!     }
//! }
//! 
//! impl Value for MovingAverageProp {}
//! 
//! /// State of the MovingAverage actor.
//! #[derive(Clone, Debug, Default)]
//! pub struct MovingAverageState {
//!     /// current moving average
//!     pub moving_average: f64,
//! }
//! 
//! impl Value for MovingAverageState {}
//! ```
//! 
//! Properties can be understood as the configuration of the actor which are specified when the 
//! actor is created. The state is the internal state of the actor that is updated when the actor 
//! receives messages. Default implementations for the state define the initial state of the actor.
//! Note that MovingAverageState implements [Default] trait through the derive macro, and hence
//! moving_average is initialized to 0.0 which is the default value for f64. An explicit
//! implementation of the [Default] trait can be used to set the values of member fields as done for
//! the [examples::moving_average::MovingAverageProp] struct here.
//! 
//! ### Inbound hub
//! 
//! Next we define the inbound hub of the actor. The inbound MovingAverageMessage enum contains a
//! single variant that carries a float [actor_inputs](macros::actor_inputs) macro generates the 
//! inbound hub struct which contains a single inbound channel.
//! 
//! ```ignore
//! /// Inbound message for the MovingAverage actor.
//! #[derive(Clone, Debug)]
//! #[actor_inputs(MovingAverageInbound, {MovingAverageProp, MovingAverageState, 
//!                                        MovingAverageOutbound})]
//! pub enum MovingAverageMessage {
//!     /// a float value
//!     Value(f64),
//! }
//! 
//! impl OnMessage for MovingAverageMessage {
//!     /// Process the inbound time-stamp message.
//!     fn on_message(&self, prop: &Self::Prop, state: &mut Self::State, outbound: &Self::Outbound) 
//!     {
//!         match &self {
//!             MovingAverageMessage::Value(new_value) => {
//!                 state.moving_average =
//!                     (prop.alpha * new_value) + (1.0 - prop.alpha) * state.moving_average;
//!                 outbound.average.send(state.moving_average);
//!                 if new_value > &prop.timeout {
//!                     outbound.cancel_request.send(());
//!                 }
//!             }
//!         }
//!     }
//! }
//! 
//! impl InboundMessageNew<f64> for MovingAverageMessage {
//!     fn new(_inbound_name: String, msg: f64) -> Self {
//!         MovingAverageMessage::Value(msg)
//!     }
//! }
//! ```
//! 
//! The moving average is calculated from the stream of values received on this channel.
//! OnMessage trait implementation the actual business logic of the actor is implemented.
//! 
//! ### The actor
//! 
//! Finally we define the actor itself:
//! 
//! ```ignore
//! /// The MovingAverage actor.
//! ///
//! #[actor(MovingAverageMessage)]
//! type MovingAverage =
//!     Actor<MovingAverageInbound, MovingAverageOutbound, MovingAverageProp,  MovingAverageState>;
//! ```
//! 
//! ### Configure and execute the pipeline
//! 
//! In order to make use of the actor we need to send messages to its inbound hub and process the
//! messages received on its outbound hub.
//! 
//! To do so, we will make use of two predefined actors. The [Periodic](actors::Periodic) actor is 
//! and vector without any input channels. It produces a stream of timestamps at a given interval and 
//! publishes them through its `time_stamp` outbound channel.
//! 
//! The [Printer](actors::Printer) actor is a sink actor. It prints the messages it receives on its 
//! only inbound channel to the console.
//! 
//! Now we can provide the code snippets to configure and execute the compute pipeline:
//! 
//! ```rust
//! # use hollywood::actors::{Periodic, Printer, PrinterProp};
//! # use hollywood::core::ActorFacade;
//! # use hollywood::compute::Context;
//! # use hollywood::examples::moving_average::{MovingAverage, MovingAverageProp};
//! let pipeline = Context::configure(&mut |context| {
//!     let mut timer = Periodic::new_with_period(context, 1.0);
//!     let mut moving_average = MovingAverage::new_default_init_state(
//!         context,
//!         MovingAverageProp {
//!             alpha: 0.3,
//!             ..Default::default()
//!         },
//!     );
//!     let mut time_printer = Printer::<f64>::new_default_init_state(
//!         context,
//!         PrinterProp {
//!             topic: "time".to_string(),
//!         },
//!     );
//!     let mut average_printer = Printer::<f64>::new_default_init_state(
//!         context,
//!         PrinterProp {
//!             topic: "average".to_string(),
//!         },
//!     );
//!     timer
//!         .outbound
//!         .time_stamp
//!         .connect(context, &mut moving_average.inbound.value);
//!     timer
//!         .outbound
//!         .time_stamp
//!         .connect(context, &mut time_printer.inbound.printable); 
//!     moving_average
//!         .outbound
//!         .average
//!         .connect(context, &mut average_printer.inbound.printable);
//! });
//! pipeline.print_flow_graph();   
//! pipeline.run();
//! ```
//! 
//! The [Pipeline::print_flow_graph()](compute::Pipeline::print_flow_graph()) method prints the 
//! topology of the compute pipeline to the console.
//! 
//! ``` text
//! *   Periodic_0   *                                     
//! |   time_stamp   |                                     
//!         ⡏⠉⠑⠒⠢⠤⠤⣀⣀                                      
//!         ⡇        ⠉⠉⠑⠒⠢⠤⠤⣀⣀                             
//!         ⠁                 ⠁                            
//! |     Value      |                  |   Printable    | 
//! *MovingAverage_0 *                  *Printer(time)_0 * 
//! |    average     |                   
//!         ⡇                                              
//!         ⡇                                              
//!         ⠁                                              
//! |   Printable    |                                     
//! *Printer(average)*       
//! ```
//! 
//! Let's interpret the print. The outbound channel of the Periodic actor is connected to the 
//! inbound channel of the MovingAverage actor and the inbound channel of the `Printer(time)` actor. 
//! Furthermore, the outbound channel of the MovingAverage actor is connected to the inbound channel
//! of the `Printer(average)` actor.
//! 
//! ## Pipeline topology and channel connections
//! 
//! The compute pipeline is a acyclic directed graph (DAG). Coarsely, we introduced the topology of 
//! the graph by a set of actors and the connections between them. In a more detailed view, the 
//! graph consists of three types of nodes: a set of actors, a set of inbound channels and a set of
//! outbound channels. Each inbound channel and outbound channel is associated with exactly one
//! actor. Sometimes we refer to an actor and its associated inbound/outbound channels as
//! a super node. Futhermore, there are three types of edges in the graph: a set of static edges
//! which link the concrete inbound channels to its actor, a set of static edges which link an actor
//! to its outbound channels, as well as a set of dynamic edges which connect the outbound channel 
//! of one actor to a compatible inbound channel of another actor downstream.
//! 
//! These channel connections are configured at runtime using 
//! [OutboundChannel::connect()](core::OutboundChannel::connect) during the pipeline configuration
//! step. In the simple example above, we had a 1:2 and a 1:1 connection: The process_time_stamp 
//! outbound channel of the Periodic actor was connected to two inbound channels. The average 
//! outbound channel of the MovingAverage actor was connected to one inbound channel. In general, 
//! channel connections are n:m. Each outbound channel can be connected to zero, one or more inbound 
//! channels. Similarly, each inbound channel can be connected to zero, one or more outbound 
//! channels. If an outbound channel is connected to multiple inbound channels, the messages are 
//! broadcasted to all connected inbound channels. This is the main reason why 
//! [InboundMessage](core::InboundMessage) must be [Clone].
//! 
//! The types of connected outbound channels must match the type of the connected inbound channel.
//! An inbound channel is uniquely identified by a **variant** of the 
//! [InboundMessage](core::InboundMessage) enum. Messages received from connected outbound channels
//! are merged into a single stream and processed in the corresponding match arm (for that
//! **variant**) within the [OnMessage::on_message()](core::OnMessage::on_message()) method in a
//! uniform manner, regardless of the outbound channel (and actor) the message originated 
//! from.
//! 
/// The core framework concepts such as actors, state, inbound, outbound and runners.
pub mod core;

/// The compute context and compute graph.
pub mod compute;

/// Introspection
pub mod introspect;

/// Library of actors.
pub mod actors;

/// Library of actors.
pub mod examples;

/// Convenience macros for hollywood to define new actor types.
/// 
/// In order to minimize potential of compile time errors, the macros are best implemented in the
/// following order:
/// 
/// 1. [actor_outputs](macros::actor_outputs)
/// 2. [actor_inputs](macros::actor_inputs) which depends on 1.
/// 3. [actor](macros::actor) which depends on 1. and 2.
/// 
/// The documentation in this module is rather technical. For a more practical introduction, please
/// refer to the examples in the root of the [crate](crate#example-moving-average).
pub mod macros {

    /// This macro generates the boilerplate for the outbound hub struct it is applied to. 
    /// 
    /// Macro template:
    ///
    /// ``` text
    /// #[actor_outputs]
    /// pub struct OUTBOUND {
    ///     pub CHANNEL0: OutboundChannel<TYPE0>,
    ///     pub CHANNEL1: OutboundChannel<TYPE1>,
    ///     ...
    /// }
    /// ```
    /// 
    /// Here, OUTBOUND is the user-specified name of the struct. The struct shall be defined right
    /// after the macro invocation. (Indeed, these types of macros are called "attribute macros".
    /// They are applied to the item directly following them, in this case a struct.) The outbound 
    /// struct consists of a zero, one or more outbound channels. Each outbound channel has a 
    /// user-specified name CHANNEL* and a user specified type TYPE*.
    /// 
    /// Effect: The macro generates the [OutboundHub](crate::core::OutboundHub) and 
    /// [Morph](crate::core::Morph) implementations for the provided struct OUTBOUND.
    ///
    /// This is the first of three macros to define an actor. The other two are [macro@actor_inputs] 
    /// and [macro@actor].
    ///
    pub use hollywood_macros::actor_outputs;

    /// This macro generates the boilerplate for the inbound hub of an actor.
    ///
    /// Macro template:
    ///
    /// ``` text
    /// #[derive(Clone, Debug)]
    /// actor_inputs(INBOUND,(PROP, STATE, OUTBOUND));
    /// pub enum INBOUND_MESSAGE {
    ///   VARIANT0(TYPE0),
    ///   VARIANT1(TYPE1),
    ///   ...
    /// }
    /// ```
    /// 
    /// INBOUND_MESSAGE is the user-specified name of an enum which shall be defined right below the 
    /// macro invocation. The enum shall consist of a zero, one or more message variants. Each 
    /// variant has a user-specified name VARIENT* and type TYPE*.
    ///
    /// Prerequisite:
    ///   - The OUTBOUND struct is defined and implements [OutboundHub](crate::core::OutboundHub) 
    ///     and [Morph](crate::core::Morph), typically using the [macro@actor_outputs] macro.
    ///   - The PROP and STATE structs are defined and implement the [Value](crate::core::Value) 
    ///     trait.
    ///
    /// Effects:
    ///   - The macro defines the struct INBOUND that contains an inbound channel field for each
    ///     variant of the INBOUND_MESSAGE enum, and implements the 
    ///     [InboundHub](crate::core::InboundHub) trait for it.
    ///   - Implements the [InboundMessage](crate::core::InboundMessage) trait for INBOUND_MESSAGE.
    ///
    /// This is the second of three macros to define an actor. The other two are 
    /// [macro@actor_outputs] and [macro@actor].
    pub use hollywood_macros::actor_inputs;

    /// This macro generates the boilerplate to define an new actor type.
    ///
    /// Macro template:
    ///
    /// ``` text
    /// #[actor(INBOUND_MESSAGE)]
    /// type ACTOR = Actor<PROP, INBOUND, STATE, OUTBOUND>;
    /// ```
    /// 
    /// Here, ACTOR is the user-specified name of the actor type. The actor type shall be defined
    /// right after the macro invocation as an alias of [Actor](crate::core::Actor).
    ///
    /// Prerequisites:
    ///   - The OUTBOUND struct is defined and implements (OutboundHub)[crate::core::OutboundHub] 
    ///     and [Morph](crate::core::Morph), e.g. using the [actor_outputs] macro.
    ///   - The INBOUND_MESSAGE enum is defined and implements 
    ///     [InboundMessage](crate::core::InboundMessage), as well as the INBOUND
    ///     struct is defined and implements the [InboundHub](crate::core::InboundHub) trait, e.g. 
    ///     through the [macro@actor_inputs] macro.
    ///   - The PROP and STATE structs are defined and implement the [Value](crate::core::Value)
    ///     trait.
    ///
    /// Effect:
    ///   - This macro implements the [ActorFacade](crate::core::ActorFacade) trait for the ACTOR 
    ///     type.
    ///
    /// This is the last of the three macros that need to be used to define a new actor type. The 
    /// first one is [macro@actor_outputs], the second one is [macro@actor_inputs].
    pub use hollywood_macros::actor;
}
