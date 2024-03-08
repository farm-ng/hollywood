use hollywood::actors::printer::PrinterProp;
use hollywood::actors::Periodic;
use hollywood::actors::Printer;
use hollywood::compute::Context;
use hollywood::core::FromPropState;
use hollywood::core::NullState;

use hollywood::example_actors::moving_average::MovingAverage;
use hollywood::example_actors::moving_average::MovingAverageProp;
use hollywood::example_actors::moving_average::MovingAverageState;

///
pub async fn run_moving_average_example() {
    let pipeline = Context::configure(&mut |context| {
        let mut timer = Periodic::new_with_period(context, 1.0);
        let mut moving_average = MovingAverage::from_prop_and_state(
            context,
            MovingAverageProp {
                alpha: 0.3,
                ..Default::default()
            },
            MovingAverageState {
                moving_average: 0.0,
            },
        );
        let mut time_printer = Printer::<f64>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "time".to_string(),
            },
            NullState {},
        );
        let mut average_printer = Printer::<f64>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "average".to_string(),
            },
            NullState {},
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
