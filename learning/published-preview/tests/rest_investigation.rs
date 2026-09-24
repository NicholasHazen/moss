use moss_course_ecosystem::{Activity, Cell, CourseWorld, MobileScenario, RestPolicy};

fn policy(minimum_rest_ticks: u32, recovery_points_per_tick: u32) -> RestPolicy {
    RestPolicy {
        maximum_fatigue_points: 10,
        effort_points_per_cell: 2,
        enter_at_points: 6,
        exit_at_points: 2,
        minimum_rest_ticks,
        recovery_points_per_tick,
        history_limit: 128,
    }
}

#[test]
fn fast_recovery_does_not_cancel_the_second_committed_action() {
    let scenario = MobileScenario::resting_journey().with_rest(policy(2, 6));
    let mut world = CourseWorld::new_mobile(scenario).unwrap();
    for _ in 0..3 { world.step(); }
    let third = world.rest_snapshot().unwrap();
    assert_eq!(third.grazers[0].fatigue_points, 6);
    assert_eq!(third.grazers[0].activity, Activity::Foraging);
    assert!(world.mobile_snapshot().unwrap().grazers[0].target.is_some());

    world.step();
    let fourth = world.rest_snapshot().unwrap();
    let fourth_mobile = world.mobile_snapshot().unwrap();
    assert_eq!((fourth.tick, fourth.grazers[0].fatigue_points), (4, 0));
    assert_eq!(fourth.grazers[0].activity, Activity::Resting { remaining_ticks: 1 });
    assert_eq!((fourth.ledger.rest_actions, fourth.ledger.recovered_points), (1, 6));
    assert_eq!(fourth_mobile.grazers[0].cell, Cell { x: 3, y: 0 });
    assert!(fourth_mobile.grazers[0].target.is_none());
    assert_eq!(fourth_mobile.base.grazers[0].reserve_units, 13);
    assert_eq!((fourth_mobile.ledger.travel_cells, fourth_mobile.base.ledger.eaten_units), (0, 0));
    assert_eq!(fourth_mobile.base.ledger.maintenance_units, 1);
    assert_eq!(world.rest_snapshot().unwrap(), fourth);
    assert_eq!(world.mobile_snapshot().unwrap(), fourth_mobile);

    world.step();
    let fifth = world.rest_snapshot().unwrap();
    let fifth_mobile = world.mobile_snapshot().unwrap();
    assert_eq!(fifth.grazers[0].activity, Activity::Resting { remaining_ticks: 0 });
    assert_eq!((fifth.grazers[0].fatigue_points, fifth.ledger.recovered_points), (0, 0));
    assert_eq!((fifth.ledger.rest_actions, fifth.totals.rest_actions), (1, 2));
    assert_eq!(fifth_mobile.grazers[0].cell, Cell { x: 3, y: 0 });
    assert!(fifth_mobile.grazers[0].target.is_none());
    assert_eq!(fifth_mobile.base.grazers[0].reserve_units, 12);
    assert_eq!((fifth_mobile.ledger.travel_cells, fifth_mobile.base.ledger.eaten_units), (0, 0));

    world.step();
    let sixth = world.rest_snapshot().unwrap();
    let sixth_mobile = world.mobile_snapshot().unwrap();
    assert_eq!(sixth.grazers[0].activity, Activity::Foraging);
    assert_eq!(sixth.grazers[0].fatigue_points, 2);
    assert_eq!((sixth.ledger.woke, sixth.ledger.rest_actions), (1, 0));
    assert_eq!(sixth.grazers[0].last_rest_tick, Some(5));
    assert_eq!(sixth.grazers[0].last_transition_tick, Some(6));
    assert_eq!(sixth_mobile.grazers[0].observed_cell, Some(Cell { x: 3, y: 0 }));
    assert_eq!(sixth_mobile.grazers[0].observed_tick, Some(6));
    assert_eq!(sixth_mobile.grazers[0].target.unwrap().observed_tick, 6);
    assert_eq!(sixth_mobile.grazers[0].cell, Cell { x: 4, y: 0 });
    assert_eq!(sixth_mobile.base.grazers[0].reserve_units, 10);
}

#[test]
fn two_and_three_actions_change_arrival_and_later_opportunities() {
    let mut two = CourseWorld::new_mobile(
        MobileScenario::resting_journey().with_rest(policy(2, 2)),
    ).unwrap();
    let mut three = CourseWorld::new_mobile(
        MobileScenario::resting_journey().with_rest(policy(3, 2)),
    ).unwrap();
    for _ in 0..6 { two.step(); three.step(); }
    let two_mobile = two.mobile_snapshot().unwrap();
    let three_mobile = three.mobile_snapshot().unwrap();
    assert_eq!((two_mobile.tick, three_mobile.tick), (6, 6));
    assert_eq!((two_mobile.grazers[0].cell.x, two_mobile.base.grazers[0].reserve_units), (4, 10));
    assert_eq!((three_mobile.grazers[0].cell.x, three_mobile.base.grazers[0].reserve_units), (3, 11));
    assert_eq!(two.rest_snapshot().unwrap().grazers[0].activity, Activity::Foraging);
    assert_eq!(three.rest_snapshot().unwrap().grazers[0].activity, Activity::Resting { remaining_ticks: 0 });

    two.step(); three.step();
    let two_mobile = two.mobile_snapshot().unwrap();
    let three_mobile = three.mobile_snapshot().unwrap();
    assert_eq!((two_mobile.grazers[0].cell.x, two_mobile.base.ledger.eaten_units), (5, 2));
    assert_eq!((three_mobile.grazers[0].cell.x, three_mobile.base.ledger.eaten_units), (4, 0));
    assert_eq!(three_mobile.base.patches[0].biomass_units, 20);

    two.step(); three.step();
    let two_mobile = two.mobile_snapshot().unwrap();
    let three_mobile = three.mobile_snapshot().unwrap();
    assert_eq!((two_mobile.grazers[0].cell.x, two_mobile.base.grazers[0].reserve_units), (5, 9));
    assert_eq!((three_mobile.grazers[0].cell.x, three_mobile.base.grazers[0].reserve_units), (5, 9));
    assert_eq!(three_mobile.base.ledger.eaten_units, 2);
    assert_eq!(two.rest_snapshot().unwrap().grazers[0].activity, Activity::Resting { remaining_ticks: 1 });
    assert_eq!(three.rest_snapshot().unwrap().grazers[0].activity, Activity::Foraging);

    two.step(); three.step();
    assert_eq!(two.snapshot().grazers[0].reserve_units, 8);
    assert_eq!(three.snapshot().grazers[0].reserve_units, 10);
}
