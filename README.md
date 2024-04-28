# hollywood

Hollywood is an actor framework for Rust -  with focus on representing actors with
heterogeneous inputs and outputs which are arranged in a non-cyclic compute graph/pipeline. The
design intend is simplicity and minimal boilerplate code. Hence, Hollywood is an abstraction over
async rust in general and the asynchronous tokio runtime in particular. If you do not seek
such an abstraction, you may want to use tokio (or another async runtime) directly.

Actors are stateful entities that communicate with each other by sending messages. An actor
actor receives streams of messages through its inbound channels, processes them and sends
messages to other actors through its outbound channels. Actors are either stateless so that
its outbound streams are a pure function of its inbound streams, or have an internal state which
is updated by incoming messages and may influences the content of its outbound messages.

Actors are arranged in a compute pipeline (or a directed acyclic graph to be more precise). The
first layer of the graph consists of a set of one or more source actors whose outbound streams
are fed by an external resource. A typical example of a source actors, include sensor drivers,
or log file readers. Terminal notes in the pipeline are actors which have either no outbound
channels or which outbound channels are not connected. Terminal actors are often sink notes
which feed data to an external resource. Example of a sink actor are robot manipulators,
log file writer or visualization components.

In addition to the feed-forward connections between actors, actors can also communicate with
each other through a set of request-reply channels. There is no restriction on which actor pairs
can be connected with such request-reply channels. For example, a request-reply channel can
be use to create a feedback loop in the compute flow.

## Example

The following example demonstrates how to create a simple pipeline with a periodic source actor
which feeds a moving average actor. The moving average actor is connected to two printer actors
which print the time stamp and the moving average to the console.

```rust
use hollywood::actors::{Periodic, Printer, PrinterProp};
use hollywood::prelude::*;
use hollywood::example_actors::moving_average::{MovingAverage, MovingAverageProp, MovingAverageState};

let pipeline = Hollywood::configure(&mut |context| {
    let mut timer = Periodic::new_with_period(context, 1.0);
    let mut moving_average = MovingAverage::from_prop_and_state(
        context,
        MovingAverageProp {
            alpha: 0.3,
            timeout: 5.0,
        },
        MovingAverageState::default(),
    );
    let mut time_printer = Printer::<f64>::from_prop_and_state(
        context,
        PrinterProp {
            topic: "time".to_string(),
        },
        NullState::default(),
    );
    let mut average_printer = Printer::<f64>::from_prop_and_state(
        context,
        PrinterProp {
            topic: "average".to_string(),
        },
        NullState::default(),
    );
    timer
        .outbound
        .time_stamp
        .connect(context, &mut moving_average.inbound.value);
    timer
        .outbound
        .time_stamp
        .connect(context, &mut time_printer.inbound.printable);
    moving_average
        .outbound
        .average
        .connect(context, &mut average_printer.inbound.printable);
});
pipeline.print_flow_graph();   
pipeline.run();
```

The output of the `print_flow_graph` method is:

```plaintext
*Periodic_0*
|   time_stamp   |
        ⡏⠉⠑⠒⠢⠤⠤⣀⣀
        ⡇        ⠉⠉⠑⠒⠢⠤⠤⣀⣀
        ⠁                 ⠁
|     Value      |                  |   Printable    |
*MovingAverage_0*                  *Printer(time)_0*
|    average     |
        ⡇
        ⡇
        ⠁
|   Printable    |
*Printer(average)*  
```
