use hollywood::actors::printer::PrinterProp;
use hollywood::actors::Periodic;
use hollywood::actors::Printer;
use hollywood::compute::Context;
use hollywood::core::*;
use hollywood::examples::one_dim_robot::{
    DrawActor, Filter, NamedFilterState, Robot, Sim, SimState, Stamped,
};

async fn run_robot_example() {
    let pipeline = Context::configure(&mut |context| {
        let mut timer = Periodic::new_with_period(context, 0.25);
        let mut sim = Sim::new_with_state(
            context,
            NullProp {},
            SimState {
                shutdown_time: 10.0,
                time: 0.0,
                true_robot: Robot {
                    position: -2.0,
                    velocity: 0.4,
                },
            },
        );
        let mut filter = Filter::new_default_init_state(context, NullProp {});
        let mut filter_state_printer = Printer::<NamedFilterState>::new_default_init_state(
            context,
            PrinterProp {
                topic: "filter state".to_owned(),
            },
        );
        let mut truth_printer = Printer::<Stamped<Robot>>::new_default_init_state(
            context,
            PrinterProp {
                topic: "truth".to_owned(),
            },
        );

        let mut draw_actor = DrawActor::new_default_init_state(context, NullProp {});

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
        sim.outbound
            .true_robot
            .connect(context, &mut draw_actor.inbound.true_pos);
        sim.outbound
            .true_range
            .connect(context, &mut draw_actor.inbound.true_range);
        sim.outbound
            .true_robot
            .connect(context, &mut truth_printer.inbound.printable);
        context.register_cancel_requester(&mut sim.outbound.cancel_request);

        filter
            .outbound
            .updated_state
            .connect(context, &mut filter_state_printer.inbound.printable);
        filter
            .outbound
            .updated_state
            .connect(context, &mut draw_actor.inbound.filter_est);
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
