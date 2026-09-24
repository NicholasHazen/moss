//! Deliberate intervening changes exercise action-time checks, not just helpers.

use super::*;
use crate::mobile::{Cell, MobileActor, MobileEventKind, MobileState, PatchTarget, Position};
use crate::{FounderTrait, GrazerPlacement, SimId, UpkeepTrait};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn observe_and_choose(course: &mut CourseWorld) {
    let mut schedule = Schedule::default();
    schedule.add_systems((mobile::perception::observe, mobile::perception::choose).chain());
    schedule.run(&mut course.world);
}

fn birth_fixture() -> CourseWorld {
    let initial = MobileScenario::short_journey();
    let mut scenario = initial.scenario().clone();
    let mut population = initial.population().clone();
    let mut space = initial.space().clone();
    scenario.daylight = Daylight::new(1, 1).unwrap();
    scenario.grazers.push({
        let mut seed = scenario.grazers[0].clone();
        seed.id = SimId(2);
        seed
    });
    scenario.patches[0].biomass_units = 0;
    scenario.patches[0].capacity_units = 20;
    scenario.patches[0].growth_units_per_lit_tick = 6;
    population.founders.push(FounderTrait {
        id: SimId(2),
        upkeep: UpkeepTrait::new(1).unwrap(),
    });
    space.grazers[0].cell = space.patches[0].cell;
    space.grazers.push(GrazerPlacement {
        id: SimId(2),
        ..space.grazers[0]
    });
    CourseWorld::new_mobile(MobileScenario::new(scenario, population, space)).unwrap()
}

#[test]
fn feeding_rechecks_current_contact_after_a_derived_token_becomes_stale() {
    let mut course = CourseWorld::new_mobile(MobileScenario::short_journey()).unwrap();
    course.step();
    let patch = course.world.resource::<Roster>().patches[0];
    course.world.get_mut::<Position>(patch).unwrap().0.x = 1;
    let mut contacts = Schedule::default();
    contacts.add_systems(mobile::derive_contacts);
    contacts.run(&mut course.world);
    assert_eq!(course.snapshot().grazers[0].feeding_site, Some(SimId(100)));
    course.world.get_mut::<Position>(patch).unwrap().0.x = 2;
    let before = course.mobile_snapshot();
    let mut meal = Schedule::default();
    meal.add_systems(feeding::feed);
    meal.run(&mut course.world);
    assert_eq!(course.mobile_snapshot(), before);
}

#[test]
fn travel_uses_the_owned_destination_not_a_targets_changed_global_cell() {
    let initial = MobileScenario::short_journey();
    let mut space = initial.space().clone();
    space.grazers[0].motion.max_cells_per_tick = 3;
    let mut course = CourseWorld::new_mobile(MobileScenario::new(
        initial.scenario().clone(),
        initial.population().clone(),
        space,
    ))
    .unwrap();
    course.world.resource_mut::<Clock>().tick = 1;
    observe_and_choose(&mut course);
    let patch = course.world.resource::<Roster>().patches[0];
    course.world.get_mut::<Position>(patch).unwrap().0 = Cell { x: 0, y: 1 };
    let mut travel = Schedule::default();
    travel.add_systems(mobile::movement::travel);
    travel.run(&mut course.world);
    let state = course.mobile_snapshot().unwrap();
    assert_eq!(state.grazers[0].cell, Cell { x: 2, y: 0 });
    assert_eq!(state.base.grazers[0].reserve_units, 3);
    assert_eq!(state.ledger.travel_cells, 2);
}

#[test]
fn a_removed_patch_kind_rejects_travel_and_repeated_travel_has_no_second_charge() {
    let mut course = CourseWorld::new_mobile(MobileScenario::short_journey()).unwrap();
    course.world.resource_mut::<Clock>().tick = 1;
    observe_and_choose(&mut course);
    let patch = course.world.resource::<Roster>().patches[0];
    course.world.entity_mut(patch).remove::<Patch>();
    let mut travel = Schedule::default();
    travel.add_systems(mobile::movement::travel);
    travel.run(&mut course.world);
    let actor = course.world.resource::<Roster>().grazers[0];
    assert_eq!(
        course.world.get::<Position>(actor).unwrap().0,
        Cell { x: 0, y: 0 }
    );
    assert_eq!(course.world.get::<Grazer>(actor).unwrap().reserve_units, 5);
    assert_eq!(
        course.world.resource::<MobileState>().ledger.travel_cells,
        0
    );

    let mut fresh = CourseWorld::new_mobile(MobileScenario::short_journey()).unwrap();
    fresh.step();
    let before = fresh.mobile_snapshot();
    let mut late = Schedule::default();
    late.add_systems(mobile::movement::travel);
    late.run(&mut fresh.world);
    assert_eq!(fresh.mobile_snapshot(), before);
}

#[test]
fn a_visible_newborn_cannot_observe_choose_or_travel_in_its_birth_tick() {
    let mut course = birth_fixture();
    course.step();
    observe_and_choose(&mut course);
    let child = course.world.resource::<Roster>().grazers[2];
    assert_eq!(
        course
            .world
            .get::<MobileActor>(child)
            .unwrap()
            .observed_tick,
        None
    );
    assert_eq!(course.world.get::<MobileActor>(child).unwrap().target, None);
    // Even an erroneous late intent assignment must not grant an early action.
    course.world.get_mut::<Position>(child).unwrap().0 = Cell { x: 1, y: 0 };
    course.world.get_mut::<MobileActor>(child).unwrap().target = Some(PatchTarget {
        patch: SimId(100),
        observed_cell: Cell { x: 2, y: 0 },
        observed_tick: 1,
    });
    let before = course.mobile_snapshot();
    let mut late = Schedule::default();
    late.add_systems(mobile::movement::travel);
    late.run(&mut course.world);
    assert_eq!(course.mobile_snapshot(), before);
    assert_eq!(course.world.get::<Grazer>(child).unwrap().reserve_units, 4);
    assert!(
        !course
            .world
            .resource::<MobileState>()
            .history
            .snapshot()
            .events
            .iter()
            .any(|event| matches!(
                event.kind,
                MobileEventKind::Travelled {
                    grazer: SimId(101),
                    ..
                }
            ))
    );
}

#[test]
fn births_recheck_parent_cells_instead_of_trusting_stale_contact_tokens() {
    let mut course = birth_fixture();
    course.world.resource_mut::<Clock>().tick = 1;
    let mut preparation = Schedule::default();
    preparation.add_systems(
        (
            supply::grow,
            mobile::perception::observe,
            mobile::perception::choose,
            mobile::derive_contacts,
        )
            .chain(),
    );
    preparation.run(&mut course.world);
    let second = course.world.resource::<Roster>().grazers[1];
    assert_eq!(
        course.world.get::<Grazer>(second).unwrap().feeding_site,
        Some(SimId(100))
    );
    course.world.get_mut::<Position>(second).unwrap().0 = Cell { x: 1, y: 0 };
    let before = course.mobile_snapshot();
    let mut births = Schedule::default();
    births.add_systems(reproduction::resolve_births);
    births.run(&mut course.world);
    births.apply_deferred(&mut course.world);
    assert_eq!(course.mobile_snapshot(), before);
    assert_eq!(
        course.world.query::<&Grazer>().iter(&course.world).count(),
        2
    );
}

#[test]
fn mobile_birth_event_exhaustion_rejects_before_any_parent_or_queue_mutation() {
    let mut course = birth_fixture();
    course.world.resource_mut::<Clock>().tick = 1;
    let mut preparation = Schedule::default();
    preparation.add_systems(
        (
            supply::grow,
            mobile::perception::observe,
            mobile::perception::choose,
            mobile::derive_contacts,
        )
            .chain(),
    );
    preparation.run(&mut course.world);
    course
        .world
        .resource_mut::<MobileState>()
        .history
        .exhaust_eviction_counter_for_test();
    let before = course.mobile_snapshot();
    let mut births = Schedule::default();
    births.set_apply_final_deferred(false);
    births.add_systems(reproduction::resolve_births);
    assert!(catch_unwind(AssertUnwindSafe(|| births.run(&mut course.world))).is_err());
    births.apply_deferred(&mut course.world);
    course.world.flush();
    assert_eq!(course.mobile_snapshot(), before);
    assert_eq!(
        course.world.query::<&Grazer>().iter(&course.world).count(),
        2
    );
}
