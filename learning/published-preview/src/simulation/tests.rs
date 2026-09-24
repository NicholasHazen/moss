//! Focused tests of boundaries unavailable through the completed-tick public API.

use std::panic::{AssertUnwindSafe, catch_unwind};

use super::*;
use crate::population::PopulationState;
use crate::{EventKind, FounderTrait, SimId, UpkeepTrait};

fn fixture() -> CourseWorld {
    let mut scenario = Scenario::limited_supply();
    scenario.daylight = Daylight::new(4, 4).unwrap();
    for seed in &mut scenario.grazers {
        seed.reserve_units = 5;
        seed.maintenance_units_per_tick = 1;
    }
    scenario.patches[0].capacity_units = 20;
    scenario.patches[0].growth_units_per_lit_tick = 6;
    let founders = scenario
        .grazers
        .iter()
        .map(|seed| FounderTrait {
            id: seed.id,
            upkeep: UpkeepTrait::new(1).unwrap(),
        })
        .collect();
    CourseWorld::new_population(scenario, PopulationConfig::new(founders)).unwrap()
}

fn birth_only() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    // Preserve the queue for an explicit before/after assertion in these tests.
    schedule.set_apply_final_deferred(false);
    schedule.add_systems(reproduction::resolve_births);
    schedule
}

#[test]
fn deferred_birth_is_invisible_until_flush_and_cannot_repeat_in_the_same_tick() {
    let mut course = fixture();
    course.world.resource_mut::<Clock>().tick = 1;
    let mut births = birth_only();
    births.run(&mut course.world);
    let mut grazers = course.world.query::<&Body>();
    assert_eq!(grazers.iter(&course.world).count(), 2);
    let child = *course.world.resource::<Roster>().grazers.last().unwrap();
    assert!(course.world.get::<Body>(child).is_none());
    births.apply_deferred(&mut course.world);
    assert_eq!(grazers.iter(&course.world).count(), 3);
    assert_eq!(course.world.get::<Body>(child).unwrap().reserve_units, 4);
    let before = course.snapshot();
    let population_before = course.population_snapshot();
    births.run(&mut course.world);
    births.apply_deferred(&mut course.world);
    assert_eq!(course.snapshot(), before);
    assert_eq!(course.population_snapshot(), population_before);
}

#[test]
fn a_visible_newborn_still_cannot_pay_maintenance_in_its_birth_tick() {
    let mut course = fixture();
    course.step();
    let before = course.snapshot();
    let mut late = Schedule::default();
    late.add_systems(lifecycle::maintain);
    late.run(&mut course.world);
    let after = course.snapshot();
    assert_eq!(after.grazers[2].id, SimId(101));
    assert_eq!(after.grazers[2].reserve_units, 4);
    assert_eq!(
        after.ledger.maintenance_units - before.ledger.maintenance_units,
        2
    );
    assert!(!after.history.events.iter().any(|event| matches!(
        event.kind,
        EventKind::Maintenance {
            grazer: SimId(101),
            ..
        }
    )));
}

#[test]
fn a_visible_newborn_still_cannot_eat_in_its_birth_tick_with_abundant_food() {
    let mut course = fixture();
    course.step();
    let patch = course.world.resource::<Roster>().patches[0];
    course.world.get_mut::<Patch>(patch).unwrap().biomass_units = 20;
    let before = course.snapshot();
    let mut late = Schedule::default();
    late.add_systems(feeding::feed);
    late.run(&mut course.world);
    let after = course.snapshot();
    assert_eq!(after.grazers[2].id, SimId(101));
    assert_eq!(after.grazers[2].reserve_units, 4);
    assert_eq!(after.ledger.eaten_units - before.ledger.eaten_units, 4);
    assert_eq!(after.patches[0].biomass_units, 16);
    assert!(!after.history.events.iter().any(|event| matches!(
        event.kind,
        EventKind::Meal {
            grazer: SimId(101),
            ..
        }
    )));
}

#[test]
fn exhausted_birth_preflight_neither_debits_parents_nor_queues_a_child() {
    for case in 0..9 {
        let mut course = fixture();
        course.world.resource_mut::<Clock>().tick = if case == 0 { u64::MAX } else { 1 };
        {
            let mut state = course.world.resource_mut::<PopulationState>();
            match case {
                0 => {} // first eligible tick would overflow
                1 => state.config.maturation_ticks = u64::MAX,
                2 => state.config.cooldown_ticks = u64::MAX,
                3 => state.totals.births = u64::MAX,
                4 => state.ledger.births = u64::MAX,
                5 => state.ledger.transferred_units = u64::MAX,
                6 => state.ledger.dissipated_units = u64::MAX,
                7 => state.next_child_id = None,
                8 => state.history.exhaust_eviction_counter_for_test(),
                _ => unreachable!(),
            }
        }
        let before = course.snapshot();
        let population_before = course.population_snapshot();
        let mut births = birth_only();
        assert!(
            catch_unwind(AssertUnwindSafe(|| births.run(&mut course.world))).is_err(),
            "case {case}"
        );
        // A rejected transaction must remain rejected after queued commands flush.
        births.apply_deferred(&mut course.world);
        course.world.flush();
        assert_eq!(course.snapshot(), before, "case {case}");
        assert_eq!(
            course.population_snapshot(),
            population_before,
            "case {case}"
        );
        assert_eq!(course.world.query::<&Body>().iter(&course.world).count(), 2);
    }
}
