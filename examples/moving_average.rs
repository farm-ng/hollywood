use hollywood::actors::printer::PrinterProp;
use hollywood::actors::{Periodic, Printer};
use hollywood::compute::Context;
use hollywood::core::ActorFacade;

use hollywood::examples::moving_average::{MovingAverage, MovingAverageProp};

///
pub async fn run_moving_average_example() {
    let pipeline = Context::configure(&mut |context| {
        let mut timer = Periodic::new_with_period(context, 1.0);
        let mut moving_average = MovingAverage::new_default_init_state(
            context,
            MovingAverageProp {
                alpha: 0.3,
                ..Default::default()
            },
        );
        let mut time_printer = Printer::<f64>::new_default_init_state(
            context,
            PrinterProp {
                topic: "time".to_string(),
            },
        );
        let mut average_printer = Printer::<f64>::new_default_init_state(
            context,
            PrinterProp {
                topic: "average".to_string(),
            },
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

        context.register_cancel_requester(&mut moving_average.outbound.cancel_request);
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
            run_moving_average_example().await;
        })
}
