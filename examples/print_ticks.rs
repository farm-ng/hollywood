use hollywood::actors::printer::PrinterProp;
use hollywood::actors::{Periodic, Printer};
use hollywood::compute::Context;
use hollywood::core::ActorFacade;

///
pub async fn run_tick_print_example() {
    let pipeline = Context::configure(&mut |context| {
        let mut timer = Periodic::new_with_period(context, 1.0);
        let mut time_printer = Printer::<f64>::new_default_init_state(
            context,
            PrinterProp {
                topic: "time".to_string(),
            },
        );
        timer
            .outbound
            .time_stamp
            .connect(context, &mut time_printer.inbound.printable);

    });

    pipeline.print_flow_graph();
    pipeline.run().await;
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_tick_print_example().await;
        })
}
