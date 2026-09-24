use moss_course_ecosystem::{CourseWorld, Daylight, Life, Scenario};

#[test]
fn equal_potential_supply_can_have_different_timing_and_survivors() {
    let mut steady_setup = Scenario::generous_supply();
    steady_setup.daylight = Daylight::new(4, 4).unwrap();
    steady_setup.patches[0].growth_units_per_lit_tick = 3;
    let mut pulsed = CourseWorld::new(Scenario::generous_supply()).unwrap();
    let mut steady = CourseWorld::new(steady_setup).unwrap();
    let (mut pulsed_added, mut steady_added) = (0_u64, 0_u64);
    for _ in 0..12 {
        pulsed.step();
        steady.step();
        pulsed_added += pulsed.snapshot().ledger.added_units;
        steady_added += steady.snapshot().ledger.added_units;
    }
    let pulsed = pulsed.snapshot();
    let steady = steady.snapshot();
    assert_eq!(pulsed.grazers.iter().filter(|g| g.life == Life::Alive).count(), 1);
    assert_eq!(steady.grazers.iter().filter(|g| g.life == Life::Alive).count(), 2);
    assert_eq!((pulsed_added, steady_added), (34, 36));
    assert_eq!(steady.grazers.iter().map(|g| g.reserve_units).collect::<Vec<_>>(), vec![3, 3]);
    assert_eq!(steady.patches[0].biomass_units, 0);
}
