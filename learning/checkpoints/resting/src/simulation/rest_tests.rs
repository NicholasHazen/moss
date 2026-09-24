//! Late-phase calls deliberately test eligibility and per-execution guards.

use super::*;
use crate::mobile::rest::{RestActor, RestState};
use crate::mobile::{MobileActor, PatchTarget, Position};
use crate::{Activity, Cell, FounderTrait, GrazerPlacement, RestPolicy, SimId, UpkeepTrait};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn pair() -> CourseWorld {
    let initial = MobileScenario::resting_journey();
    let mut scenario = initial.scenario().clone();
    let mut population = initial.population().clone();
    let mut space = initial.space().clone();
    let mut second = scenario.grazers[0].clone();
    second.id = SimId(2);
    scenario.grazers.push(second);
    population.founders.push(FounderTrait {
        id: SimId(2),
        upkeep: UpkeepTrait::new(1).unwrap(),
    });
    space.grazers[0].cell = space.patches[0].cell;
    space.grazers.push(GrazerPlacement {
        id: SimId(2),
        ..space.grazers[0]
    });
    CourseWorld::new_mobile(
        MobileScenario::new(scenario, population, space).with_rest(RestPolicy::default()),
    )
    .unwrap()
}

#[test]
fn late_newborn_cannot_transition_or_recover_even_if_given_a_rest_intention() {
    let mut course = pair();
    course.step();
    let child = *course.world.resource::<Roster>().grazers.last().unwrap();
    let child_id = course.world.get::<Identity>(child).unwrap().0;
    assert!(child_id.0 > 100);
    course
        .world
        .get_mut::<MobileActor>(child)
        .unwrap()
        .rest
        .as_mut()
        .unwrap()
        .fatigue_points = 6;
    let before = course.rest_snapshot();
    let mut decide = Schedule::default();
    decide.add_systems(mobile::rest::decide);
    decide.run(&mut course.world);
    assert_eq!(course.rest_snapshot(), before);
    course
        .world
        .get_mut::<MobileActor>(child)
        .unwrap()
        .rest
        .as_mut()
        .unwrap()
        .activity = Activity::Resting { remaining_ticks: 2 };
    let before = course.rest_snapshot();
    let reserve = course.world.get::<Grazer>(child).unwrap().reserve_units;
    let mut execute = Schedule::default();
    execute.add_systems(mobile::rest::execute);
    execute.run(&mut course.world);
    assert_eq!(course.rest_snapshot(), before);
    assert_eq!(
        course.world.get::<Grazer>(child).unwrap().reserve_units,
        reserve
    );
    assert_eq!(course.rest_snapshot().unwrap().totals.rest_actions, 0);
}

#[test]
fn decisions_do_not_spend_commitment_and_repeated_effects_do_not_recover_twice() {
    let mut course = CourseWorld::new_mobile(MobileScenario::resting_journey()).unwrap();
    for _ in 0..4 {
        course.step();
    }
    let before = course.rest_snapshot();
    let mut decide = Schedule::default();
    decide.add_systems(mobile::rest::decide);
    for _ in 0..5 {
        decide.run(&mut course.world);
        assert_eq!(course.rest_snapshot(), before);
    }
    let mut execute = Schedule::default();
    execute.add_systems(mobile::rest::execute);
    execute.run(&mut course.world);
    assert_eq!(course.rest_snapshot(), before);
    // Advance the clock without running effects: policy evaluation still owes the action.
    course.world.resource_mut::<Clock>().tick = 5;
    let before = course.rest_snapshot();
    for _ in 0..5 {
        decide.run(&mut course.world);
        assert_eq!(course.rest_snapshot(), before);
    }
    execute.run(&mut course.world);
    let before = course.rest_snapshot();
    assert_eq!(
        before.as_ref().unwrap().grazers[0].activity,
        Activity::Resting { remaining_ticks: 0 }
    );
    decide.run(&mut course.world); // A late decision cannot wake after this tick's effect.
    assert_eq!(course.rest_snapshot(), before);
}

#[test]
fn stale_targets_cannot_grant_resting_meals_or_births() {
    let mut course = pair();
    course.world.resource_mut::<Clock>().tick = 1;
    let mut prepare = Schedule::default();
    prepare.add_systems(
        (
            mobile::perception::observe,
            mobile::perception::choose,
            mobile::derive_contacts,
        )
            .chain(),
    );
    prepare.run(&mut course.world);
    let entities = course.world.resource::<Roster>().grazers.clone();
    for entity in entities {
        course.world.get_mut::<MobileActor>(entity).unwrap().rest = Some(RestActor {
            fatigue_points: 6,
            activity: Activity::Resting { remaining_ticks: 2 },
            ..RestActor::default()
        });
    }
    let before = course.mobile_snapshot();
    let rest = course.rest_snapshot();
    let mut actions = Schedule::default();
    actions.add_systems((feeding::feed, reproduction::resolve_births, ApplyDeferred).chain());
    actions.run(&mut course.world);
    actions.apply_deferred(&mut course.world);
    course.world.flush();
    assert_eq!(course.mobile_snapshot(), before);
    assert_eq!(course.rest_snapshot(), rest);
    assert_eq!(
        course.world.query::<&Grazer>().iter(&course.world).count(),
        2
    );
}

#[test]
fn effort_counter_exhaustion_precedes_position_reserve_or_mobile_event_mutation() {
    let mut course = CourseWorld::new_mobile(MobileScenario::resting_journey()).unwrap();
    course.world.resource_mut::<Clock>().tick = 1;
    let actor = course.world.resource::<Roster>().grazers[0];
    course.world.get_mut::<MobileActor>(actor).unwrap().target = Some(PatchTarget {
        patch: SimId(100),
        observed_cell: Cell { x: 5, y: 0 },
        observed_tick: 1,
    });
    course
        .world
        .resource_mut::<RestState>()
        .totals
        .effort_points = u64::MAX;
    let mobile = course.mobile_snapshot();
    let rest = course.rest_snapshot();
    let mut travel = Schedule::default();
    travel.add_systems(mobile::movement::travel);
    assert!(catch_unwind(AssertUnwindSafe(|| travel.run(&mut course.world))).is_err());
    assert_eq!(course.mobile_snapshot(), mobile);
    assert_eq!(course.rest_snapshot(), rest);
    assert_eq!(
        course.world.get::<Position>(actor).unwrap().0,
        Cell { x: 0, y: 0 }
    );
}
