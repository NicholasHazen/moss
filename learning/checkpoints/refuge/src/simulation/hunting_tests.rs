//! Deliberate late adapters expose transaction atomicity before structural cleanup.

use super::*;
use crate::hunting::{self, HuntState, Hunter, PreyTarget};
use crate::mobile::rest::RestActor;
use crate::mobile::{MobileActor, Position};
use crate::{Activity, Cell, SimId};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn prepared() -> CourseWorld {
    let mut course = CourseWorld::new_mobile(MobileScenario::hunting_contention()).unwrap();
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            begin_tick,
            lifecycle::maintain,
            mobile::perception::observe,
            hunting::perception::observe,
            mobile::perception::choose,
            hunting::perception::choose,
            mobile::derive_contacts,
        )
            .chain(),
    );
    schedule.run(&mut course.world);
    course
}
fn capture(course: &mut CourseWorld) {
    let mut schedule = Schedule::default();
    schedule.add_systems(hunting::capture::capture);
    schedule.run(&mut course.world);
}

#[test]
fn immediate_capture_blocks_repeat_reward_late_meal_and_second_terminal_cause() {
    let mut course = prepared();
    let prey = course.world.resource::<Roster>().grazers[0];
    let winner = course.world.resource::<Roster>().hunters[0];
    // Leave room for a second whole prey so capacity cannot mask the action guard.
    course.world.get_mut::<Body>(winner).unwrap().capacity_units = 16;
    capture(&mut course);
    assert_eq!(
        course.world.get::<Body>(prey).unwrap().terminal,
        Terminal::Captured { hunter: SimId(100) }
    );
    assert_eq!(course.world.get::<Body>(prey).unwrap().reserve_units, 0);
    let before = (course.mobile_snapshot(), course.hunt_snapshot());
    capture(&mut course);
    assert_eq!((course.mobile_snapshot(), course.hunt_snapshot()), before);
    // Even a bad late target edit cannot buy a second accepted action this tick.
    course.world.get_mut::<Hunter>(winner).unwrap().target = Some(PreyTarget {
        prey: SimId(2),
        observed_cell: Cell { x: 0, y: 0 },
        observed_tick: 1,
    });
    let before = course.hunt_snapshot();
    capture(&mut course);
    assert_eq!(course.hunt_snapshot(), before);
    let mut later = Schedule::default();
    later.add_systems(
        (
            feeding::feed,
            lifecycle::starve,
            reproduction::resolve_births,
            ApplyDeferred,
        )
            .chain(),
    );
    later.run(&mut course.world);
    assert_eq!(
        course.world.get::<Body>(prey).unwrap().terminal,
        Terminal::Captured { hunter: SimId(100) }
    );
    assert_eq!(course.snapshot().ledger.starvations, 0);
    assert_eq!(course.population_snapshot().unwrap().totals.births, 0);
    assert_eq!(course.world.get::<Body>(prey).unwrap().reserve_units, 0);
}

#[test]
fn exhausted_capture_evidence_leaves_every_participant_counter_and_queue_unchanged() {
    for history in [false, true] {
        let mut course = prepared();
        if history {
            course
                .world
                .resource_mut::<HuntState>()
                .history
                .exhaust_for_test();
        } else {
            course.world.resource_mut::<HuntState>().totals.captures = u64::MAX;
        }
        let before = (
            course.mobile_snapshot(),
            course.rest_snapshot(),
            course.hunt_snapshot(),
        );
        let mut schedule = Schedule::default();
        schedule.set_apply_final_deferred(false);
        schedule.add_systems(hunting::capture::capture);
        assert!(catch_unwind(AssertUnwindSafe(|| schedule.run(&mut course.world))).is_err());
        schedule.apply_deferred(&mut course.world);
        course.world.flush();
        assert_eq!(
            (
                course.mobile_snapshot(),
                course.rest_snapshot(),
                course.hunt_snapshot()
            ),
            before
        );
        assert_eq!(course.world.query::<&Body>().iter(&course.world).count(), 4);
    }
}

#[test]
fn remaining_effort_and_resting_activity_reject_before_cost_or_reward() {
    for resting in [false, true] {
        let mut course = prepared();
        let entities = course.world.resource::<Roster>().hunters.clone();
        for entity in entities {
            course.world.get_mut::<MobileActor>(entity).unwrap().rest = Some(RestActor {
                fatigue_points: 9,
                activity: if resting {
                    Activity::Resting { remaining_ticks: 2 }
                } else {
                    Activity::Foraging
                },
                ..Default::default()
            });
        }
        let before = (
            course.mobile_snapshot(),
            course.rest_snapshot(),
            course.hunt_snapshot(),
        );
        capture(&mut course);
        assert_eq!(
            (
                course.mobile_snapshot(),
                course.rest_snapshot(),
                course.hunt_snapshot()
            ),
            before
        );
    }
}

#[test]
fn a_resting_prey_is_eligible_but_a_visible_newborn_is_not() {
    let mut course = prepared();
    let prey = course.world.resource::<Roster>().grazers[0];
    course
        .world
        .get_mut::<MobileActor>(prey)
        .unwrap()
        .rest
        .as_mut()
        .unwrap()
        .activity = Activity::Resting { remaining_ticks: 2 };
    capture(&mut course);
    assert_eq!(course.hunt_snapshot().unwrap().totals.captures, 1);
    let initial = MobileScenario::hunting_contention();
    let mut config = initial.hunters().unwrap().clone();
    for hunter in &mut config.hunters {
        hunter.cell = Cell { x: 1, y: 0 };
        hunter.motion.sensing_radius = 0;
    }
    let mut course = CourseWorld::new_mobile(initial.with_hunters(config)).unwrap();
    course.step();
    let child = *course.world.resource::<Roster>().grazers.last().unwrap();
    let id = course.world.get::<Identity>(child).unwrap().0;
    assert_eq!(id, SimId(1001));
    let hunters = course.world.resource::<Roster>().hunters.clone();
    for hunter in hunters {
        course.world.get_mut::<Position>(hunter).unwrap().0 = Cell { x: 0, y: 0 };
        course.world.get_mut::<Hunter>(hunter).unwrap().target = Some(PreyTarget {
            prey: id,
            observed_cell: Cell { x: 0, y: 0 },
            observed_tick: 1,
        });
    }
    let before = (
        course.mobile_snapshot(),
        course.rest_snapshot(),
        course.hunt_snapshot(),
    );
    capture(&mut course);
    assert_eq!(
        (
            course.mobile_snapshot(),
            course.rest_snapshot(),
            course.hunt_snapshot()
        ),
        before
    );
    assert_eq!(course.world.get::<Body>(child).unwrap().reserve_units, 4);
}
