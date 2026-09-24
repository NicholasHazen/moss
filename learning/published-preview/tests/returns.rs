//! Worked checks for “Return with a different question.”
//! Save this whole file as work/fieldwork/tests/returns.rs, preserving any
//! investigation.rs already there. This is an integration test, not a rustc lab.
//! From the Moss repository root:
//! cargo +1.93.1 test --offline --locked --manifest-path work/fieldwork/Cargo.toml --test returns
//! python3 learning/scripts/fieldwork.py check --project work/fieldwork
//!
//! Cases 1–3 use separate, nonbiological Bevy worlds. Cases 4–5 execute the
//! actual persistent course library. Case 6 returns to the existing target lab.

mod membership {
    use bevy_ecs::{prelude::*, schedule::ExecutorKind};

    #[derive(Component)]
    struct Identity(u32);
    #[derive(Component)]
    struct Reserve(u32);
    #[derive(Component)]
    struct Active;

    fn decrement(mut rows: Query<(&Identity, &mut Reserve), With<Active>>) {
        for (_, mut reserve) in &mut rows {
            reserve.0 = reserve.0.checked_sub(1).expect("fixture has enough units");
        }
    }

    fn report(world: &mut World) -> Vec<(u32, u32)> {
        let mut query = world.query::<(&Identity, &Reserve)>();
        let mut rows: Vec<_> = query
            .iter(world)
            .map(|(id, reserve)| (id.0, reserve.0))
            .collect();
        rows.sort_by_key(|(id, _)| *id);
        rows
    }

    #[test]
    fn case_1_required_components_and_owned_readings_answer_different_questions() {
        let mut world = World::new();
        let a = world.spawn((Identity(11), Reserve(7), Active)).id();
        let b = world.spawn((Identity(12), Reserve(7))).id();
        let unnamed = world.spawn((Reserve(7), Active)).id();
        let saved = report(&mut world);
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.add_systems(decrement);
        for _ in 0..2 {
            let _ = report(&mut world); // Reading does not run the schedule.
            schedule.run(&mut world);
            let _ = report(&mut world);
        }
        assert_eq!(world.get::<Reserve>(a).unwrap().0, 5);
        assert_eq!(world.get::<Reserve>(b).unwrap().0, 7);
        assert_eq!(world.get::<Reserve>(unnamed).unwrap().0, 7);
        assert_eq!(saved, vec![(11, 7), (12, 7)]);
        assert_eq!(report(&mut world), vec![(11, 5), (12, 7)]);
    }
}

mod projection {
    use bevy_ecs::{prelude::*, schedule::ExecutorKind};

    #[derive(Component)]
    struct Chosen;
    #[derive(Component)]
    struct Sample {
        used: u32,
        capacity: u32,
    }
    #[derive(Resource)]
    struct Display(Option<u32>);
    #[derive(Resource, Default)]
    struct Trace(Vec<Option<u32>>);

    fn project(rows: Query<&Sample, With<Chosen>>, mut display: ResMut<Display>) {
        display.0 = rows.single().ok().and_then(|sample| {
            (sample.capacity != 0).then(|| {
                (u64::from(sample.used.min(sample.capacity)) * 100 / u64::from(sample.capacity))
                    as u32
            })
        });
    }

    fn observe(display: Res<Display>, mut trace: ResMut<Trace>) {
        trace.0.push(display.0);
    }

    enum Order {
        Missing,
        Reversed,
        Correct,
    }

    fn run(order: Order) -> (Option<u32>, Vec<Option<u32>>) {
        let mut world = World::new();
        world.spawn((
            Sample {
                used: 2,
                capacity: 7,
            },
            Chosen,
        ));
        world.spawn(Sample {
            used: 4,
            capacity: 5,
        });
        world.insert_resource(Display(Some(91)));
        world.init_resource::<Trace>();
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        match order {
            Order::Missing => {
                schedule.add_systems(observe);
            }
            Order::Reversed => {
                schedule.add_systems((observe, project).chain());
            }
            Order::Correct => {
                schedule.add_systems((project, observe).chain());
            }
        }
        schedule.run(&mut world);
        (
            world.resource::<Display>().0,
            world.resource::<Trace>().0.clone(),
        )
    }

    #[test]
    fn case_2_final_display_separates_missing_installation_from_wrong_order() {
        assert_eq!(run(Order::Missing), (Some(91), vec![Some(91)]));
        assert_eq!(run(Order::Reversed), (Some(28), vec![Some(91)]));
        assert_eq!(run(Order::Correct), (Some(28), vec![Some(28)]));
    }
}

mod deferred {
    use bevy_ecs::{
        prelude::*,
        schedule::{ExecutorKind, ScheduleBuildSettings},
    };

    #[derive(Component)]
    struct RowId(u32);
    #[derive(Resource, Default)]
    struct Reserved(Vec<Entity>);
    #[derive(Resource, Default)]
    struct Seen {
        before: Vec<u32>,
        after: Vec<u32>,
    }

    fn enqueue(mut commands: Commands, mut reserved: ResMut<Reserved>) {
        reserved.0.push(commands.spawn(RowId(41)).id());
    }

    fn before(rows: Query<&RowId>, mut seen: ResMut<Seen>) {
        seen.before = rows.iter().map(|row| row.0).collect();
        seen.before.sort();
    }

    fn after(rows: Query<&RowId>, mut seen: ResMut<Seen>) {
        seen.after = rows.iter().map(|row| row.0).collect();
        seen.after.sort();
    }

    fn run(intermediate_application: bool) -> (Vec<u32>, Vec<u32>, u32) {
        let mut world = World::new();
        world.init_resource::<Reserved>();
        world.init_resource::<Seen>();
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.set_build_settings(ScheduleBuildSettings {
            auto_insert_apply_deferred: false,
            ..Default::default()
        });
        schedule.set_apply_final_deferred(true);
        if intermediate_application {
            schedule.add_systems((enqueue, before, ApplyDeferred, after).chain());
        } else {
            schedule.add_systems((enqueue, before, after).chain());
        }
        schedule.run(&mut world);
        let saved = world.resource::<Seen>();
        let handle = world.resource::<Reserved>().0[0];
        (
            saved.before.clone(),
            saved.after.clone(),
            world.get::<RowId>(handle).unwrap().0,
        )
    }

    #[test]
    fn case_3_final_existence_does_not_rewrite_an_earlier_observation() {
        assert_eq!(run(true), (vec![], vec![41], 41));
        assert_eq!(run(false), (vec![], vec![], 41));
    }
}

use moss_course_ecosystem::{
    CourseWorld, Daylight, GrazerSeed, Life, PatchSeed, Scenario, SimId, TickLedger,
};

#[test]
fn case_4_actual_course_tick_caps_growth_and_rescues_after_actual_upkeep() {
    let mut world = CourseWorld::new(Scenario {
        daylight: Daylight::new(4, 2).unwrap(),
        grazers: vec![GrazerSeed {
            id: SimId(8),
            reserve_units: 2,
            capacity_units: 6,
            maintenance_units_per_tick: 5,
            meal_units_per_tick: 3,
            feeding_site: Some(SimId(80)),
        }],
        patches: vec![PatchSeed {
            id: SimId(80),
            biomass_units: 1,
            capacity_units: 4,
            growth_units_per_lit_tick: 4,
        }],
        history_limit: 32,
    })
    .unwrap();
    let before = world.snapshot();
    assert_eq!(before.tick, 0);
    world.step();
    let after = world.snapshot();
    assert_eq!(after.tick, 1);
    assert_eq!(
        (after.grazers[0].reserve_units, after.grazers[0].life),
        (3, Life::Alive)
    );
    assert_eq!(after.patches[0].biomass_units, 1);
    assert_eq!(
        after.ledger,
        TickLedger {
            added_units: 3,
            maintenance_units: 2,
            eaten_units: 3,
            starvations: 0,
        }
    );
    let before_stores =
        u64::from(before.grazers[0].reserve_units) + u64::from(before.patches[0].biomass_units);
    let after_stores =
        u64::from(after.grazers[0].reserve_units) + u64::from(after.patches[0].biomass_units);
    assert_eq!(
        after_stores + after.ledger.maintenance_units,
        before_stores + after.ledger.added_units
    );
    assert_eq!(after.history.starvations_between(1, 1), Some(0));
}

#[test]
fn case_5_evicted_death_and_collected_empty_ticks_have_different_evidence() {
    let mut world = CourseWorld::new(Scenario {
        daylight: Daylight::new(4, 0).unwrap(),
        grazers: vec![GrazerSeed {
            id: SimId(9),
            reserve_units: 4,
            capacity_units: 4,
            maintenance_units_per_tick: 1,
            meal_units_per_tick: 0,
            feeding_site: None,
        }],
        patches: vec![],
        history_limit: 0,
    })
    .unwrap();
    for _ in 0..7 {
        world.step();
    }
    let state = world.snapshot();
    assert_eq!(state.grazers[0].life, Life::Dead);
    assert_eq!(state.grazers[0].reserve_units, 0);
    assert!(state.history.events.is_empty());
    assert_eq!(state.history.evicted_events, 5);
    assert_eq!(
        (
            state.history.complete_after_tick,
            state.history.collected_through_tick
        ),
        (4, 7)
    );
    assert_eq!(state.history.starvations_between(4, 7), None);
    assert_eq!(state.history.starvations_between(5, 7), Some(0));
    assert_eq!(state.history.starvations_between(7, 8), None);
}
