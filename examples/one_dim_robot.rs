use hollywood::actors::printer::PrinterProp;
use hollywood::actors::zip::Zip3State;
use hollywood::actors::zip::ZipPair;
use hollywood::actors::Periodic;
use hollywood::actors::Printer;
use hollywood::actors::Zip3;
use hollywood::example_actors::one_dim_robot::draw::DrawState;
use hollywood::example_actors::one_dim_robot::filter::FilterState;
use hollywood::example_actors::one_dim_robot::DrawActor;
use hollywood::example_actors::one_dim_robot::Filter;
use hollywood::example_actors::one_dim_robot::NamedFilterState;
use hollywood::example_actors::one_dim_robot::Robot;
use hollywood::example_actors::one_dim_robot::Sim;
use hollywood::example_actors::one_dim_robot::SimState;
use hollywood::example_actors::one_dim_robot::Stamped;
use hollywood::prelude::*;

async fn run_robot_example() {
    let pipeline = Hollywood::configure(&mut |context| {
        let mut timer = Periodic::new_with_period(context, 0.1);
        let mut sim = Sim::from_prop_and_state(
            context,
            NullProp {},
            SimState {
                shutdown_time: 15.0,
                time: 0.0,
                seq: 0,
                true_robot: Robot {
                    position: -2.0,
                    velocity: 0.4,
                },
            },
        );
        let mut filter = Filter::from_prop_and_state(context, NullProp {}, FilterState::default());
        let mut filter_state_printer = Printer::<NamedFilterState>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "filter state".to_owned(),
            },
            NullState::default(),
        );
        let mut truth_printer = Printer::<Stamped<Robot>>::from_prop_and_state(
            context,
            PrinterProp {
                topic: "truth".to_owned(),
            },
            NullState::default(),
        );

        let mut zip = Zip3::from_prop_and_state(context, NullProp {}, Zip3State::default());

        let mut draw = DrawActor::from_prop_and_state(context, NullProp {}, DrawState::default());

        timer
            .outbound
            .time_stamp
            .connect(context, &mut sim.inbound.time_stamp);

        sim.outbound
            .noisy_velocity
            .connect(context, &mut filter.inbound.noisy_velocity);
        sim.outbound
            .noisy_range
            .connect(context, &mut filter.inbound.noisy_range);
        sim.outbound.true_robot.connect_with_adapter(
            context,
            |x| ZipPair {
                key: x.seq,
                value: x,
            },
            &mut zip.inbound.item0,
        );
        sim.outbound.true_range.connect_with_adapter(
            context,
            |x| ZipPair {
                key: x.seq,
                value: x,
            },
            &mut zip.inbound.item1,
        );
        sim.outbound
            .true_robot
            .connect(context, &mut truth_printer.inbound.printable);

        sim.out_requests
            .ping_pong
            .connect(context, &mut filter.in_requests.ping_pong_request);
        context.register_cancel_requester(&mut sim.outbound.cancel_request);

        filter
            .outbound
            .updated_state
            .connect(context, &mut filter_state_printer.inbound.printable);
        filter.outbound.updated_state.connect_with_adapter(
            context,
            |x| ZipPair {
                key: x.state.seq,
                value: x,
            },
            &mut zip.inbound.item2,
        );

        zip.outbound
            .zipped
            .connect(context, &mut draw.inbound.zipped);
    });

    pipeline.print_flow_graph();

    let _pipeline = pipeline.run().await;
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_robot_example().await;
        })
}
