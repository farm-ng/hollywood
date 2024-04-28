use hollywood::actors::printer::PrinterProp;
use hollywood::actors::Periodic;
use hollywood::actors::Printer;
use hollywood::prelude::*;

/// Run the tick print example
pub async fn run_tick_print_example() {
    let pipeline = Hollywood::configure(&mut |context| {
        let mut timer = Periodic::new_with_period(context, 1.0);
        let mut time_printer = Printer::<f64>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "time".to_string(),
            },
            NullState::default(),
        );
        timer.outbound.time_stamp.connect_with_adapter(
            context,
            |t| 10.0 * t,
            &mut time_printer.inbound.printable,
        );
    });

    pipeline.print_flow_graph();
    pipeline.run().await;
}

fn main() {
    tracing_subscriber::fmt::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_tick_print_example().await;
        })
}
