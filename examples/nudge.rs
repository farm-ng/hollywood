use hollywood::actors::printer::PrinterProp;
use hollywood::actors::Nudge;
use hollywood::actors::Printer;
use hollywood::prelude::*;

pub async fn run_tick_print_example() {
    let pipeline = Hollywood::configure(&mut |context| {
        let mut nudge = Nudge::<String>::new(context, "nudge".to_owned());
        let mut nudge_printer = Printer::<String>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "nudge: ".to_string(),
            },
            NullState::default(),
        );
        nudge
            .outbound
            .nudge
            .connect(context, &mut nudge_printer.inbound.printable);
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
