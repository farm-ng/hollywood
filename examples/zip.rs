use hollywood::actors::periodic;
use hollywood::actors::printer::PrinterProp;
use hollywood::actors::zip::Tuple2;
use hollywood::actors::zip::Zip2State;
use hollywood::actors::zip::ZipPair;
use hollywood::actors::Printer;
use hollywood::actors::Zip2;
use hollywood::prelude::*;

pub async fn run_tick_print_example() {
    let pipeline = Hollywood::configure(&mut |context| {
        let mut periodic = periodic::Periodic::new_with_period(context, 1.0);

        let mut zip = Zip2::<u64, String, String>::from_prop_and_state(
            context,
            NullProp::default(),
            Zip2State::default(),
        );
        let mut printer = Printer::<Tuple2<u64, String, String>>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "zipped".to_string(),
            },
            NullState::default(),
        );

        periodic.outbound.time_stamp.connect_with_adapter(
            context,
            |t| ZipPair {
                key: t as u64,
                value: "hello".to_string(),
            },
            &mut zip.inbound.item0,
        );
        periodic.outbound.time_stamp.connect_with_adapter(
            context,
            |t| ZipPair {
                key: 2 * t as u64,
                value: "world".to_string(),
            },
            &mut zip.inbound.item1,
        );

        zip.outbound
            .zipped
            .connect(context, &mut printer.inbound.printable);
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
