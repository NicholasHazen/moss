use moss_course_ecosystem::{
    Cell, CourseWorld, Daylight, EventKind, FounderTrait, GrazerPlacement,
    GrazerSeed, GridBounds, HuntConfig, HuntEventKind, HunterSeed,
    MobileScenario, Motion, PatchPlacement, PatchSeed, PopulationConfig,
    RestPolicy, Scenario, SimId, SpatialConfig, UpkeepTrait,
};

fn fixture(with_hunters: bool, first_hunter_capacity: u32) -> MobileScenario {
    let cell = Cell { x: 0, y: 0 };
    let motion = Motion {
        sensing_radius: 0, max_cells_per_tick: 0, travel_units_per_cell: 1,
    };
    let scenario = Scenario {
        daylight: Daylight::new(1, 0).unwrap(),
        grazers: [2, 1].into_iter().map(|id| GrazerSeed {
            id: SimId(id), reserve_units: 5, capacity_units: 8,
            maintenance_units_per_tick: 1, meal_units_per_tick: 2,
            feeding_site: None,
        }).collect(),
        patches: vec![PatchSeed {
            id: SimId(1000), biomass_units: 10, capacity_units: 10,
            growth_units_per_lit_tick: 0,
        }],
        history_limit: 128,
    };
    let population = PopulationConfig {
        founders: [2, 1].into_iter().map(|id| FounderTrait {
            id: SimId(id), upkeep: UpkeepTrait::new(1).unwrap(),
        }).collect(),
        maturation_ticks: 2, cooldown_ticks: 3,
        contribution_units_per_parent: 2, birth_cost_units_per_parent: 1,
        child_capacity_units: 8, child_meal_units_per_tick: 2,
        max_living: 8, mutation_cycle: vec![0], history_limit: 128,
    };
    let space = SpatialConfig {
        bounds: GridBounds { width_cells: 1, height_cells: 1 },
        grazers: [2, 1].into_iter().map(|id| GrazerPlacement {
            id: SimId(id), cell, motion,
        }).collect(),
        patches: vec![PatchPlacement { id: SimId(1000), cell }],
        newborn_motion: motion,
    };
    let rest = RestPolicy {
        maximum_fatigue_points: 10, effort_points_per_cell: 2,
        enter_at_points: 6, exit_at_points: 2,
        minimum_rest_ticks: 2, recovery_points_per_tick: 2, history_limit: 128,
    };
    let initial = MobileScenario::new(scenario, population, space).with_rest(rest);
    if !with_hunters { return initial; }
    initial.with_hunters(HuntConfig::new([101, 100].into_iter().map(|id| HunterSeed {
        id: SimId(id), reserve_units: 4,
        capacity_units: if id == 100 { first_hunter_capacity } else { 8 },
        maintenance_units_per_tick: 1, attack_units: 1,
        attack_effort_points: 2, cell, motion,
    }).collect()))
}

#[test]
fn one_capture_cancels_the_preys_later_opportunities() {
    let mut control = CourseWorld::new_mobile(fixture(false, 8)).unwrap();
    control.step();
    let control_report = control.mobile_snapshot().unwrap();
    assert_eq!(control_report.population.totals.births, 1);
    assert_eq!(control_report.base.grazers.len(), 3);
    assert_eq!(control_report.base.ledger.eaten_units, 4);
    assert_eq!(control_report.base.patches[0].biomass_units, 6);

    let mut world = CourseWorld::new_mobile(fixture(true, 8)).unwrap();
    world.step();
    let mobile = world.mobile_snapshot().unwrap();
    let hunt = world.hunt_snapshot().unwrap();
    assert_eq!((mobile.run, mobile.tick), (hunt.run, hunt.tick));
    assert_eq!(hunt.tick, 1);
    assert_eq!((hunt.initial_hunters, hunt.living_hunters), (2, 2));
    assert_eq!(hunt.hunters.iter().map(|h| (h.id, h.reserve_units)).collect::<Vec<_>>(),
        vec![(SimId(100), 6), (SimId(101), 3)]);
    assert_eq!(hunt.hunters.iter().map(|h| h.fatigue_points).collect::<Vec<_>>(),
        vec![Some(2), Some(0)]);
    assert_eq!(mobile.base.grazers.iter().map(|g| (g.id, g.reserve_units)).collect::<Vec<_>>(),
        vec![(SimId(2), 6)]);
    assert_eq!(mobile.base.patches[0].biomass_units, 8);
    assert_eq!((mobile.base.ledger.eaten_units, mobile.population.totals.births), (2, 0));
    assert_eq!(mobile.population.totals.starvations, 0);
    assert_eq!((hunt.totals.captures, hunt.totals.hunter_starvations), (1, 0));
    assert_eq!((hunt.ledger.attack_units, hunt.ledger.transferred_units), (1, 4));
    let choices: Vec<_> = hunt.history.events.iter().filter_map(|e| match e.kind {
        HuntEventKind::TargetChanged { hunter, to: Some(prey), .. } => Some((hunter, prey)),
        _ => None,
    }).collect();
    assert_eq!(choices, vec![(SimId(100), SimId(1)), (SimId(101), SimId(1))]);
    let captures: Vec<_> = hunt.history.events.iter().filter_map(|e| match e.kind {
        HuntEventKind::Captured { hunter, prey, cell, transferred_units, attack_units, effort_points } =>
            Some((e.run, e.tick, hunter, prey, cell, transferred_units, attack_units, effort_points)),
        _ => None,
    }).collect();
    assert_eq!(captures, vec![(hunt.run, 1, SimId(100), SimId(1), Cell { x: 0, y: 0 }, 4, 1, 2)]);
    let meals: Vec<_> = mobile.base.history.events.iter().filter_map(|e| match e.kind {
        EventKind::Meal { grazer, units, .. } => Some((grazer, units)),
        _ => None,
    }).collect();
    assert_eq!(meals, vec![(SimId(2), 2)]);
    assert_eq!(hunt.history.captures_between(1, 1), Some(1));
    let ending_stores: u64 = mobile.base.grazers.iter().map(|g| u64::from(g.reserve_units)).sum::<u64>()
        + mobile.base.patches.iter().map(|p| u64::from(p.biomass_units)).sum::<u64>()
        + hunt.hunters.iter().map(|h| u64::from(h.reserve_units)).sum::<u64>();
    assert_eq!(ending_stores, 23);
    assert_eq!(ending_stores + mobile.base.ledger.maintenance_units
        + hunt.ledger.maintenance_units + mobile.ledger.travel_units
        + hunt.ledger.travel_units + hunt.ledger.attack_units
        + mobile.population.ledger.dissipated_units, 28 + mobile.base.ledger.added_units);
    assert_eq!(world.hunt_snapshot().unwrap(), hunt);
    assert_eq!(world.mobile_snapshot().unwrap(), mobile);
}

#[test]
fn a_rejected_whole_transfer_leaves_the_prey_for_the_next_hunter() {
    let mut world = CourseWorld::new_mobile(fixture(true, 5)).unwrap();
    world.step();
    let hunt = world.hunt_snapshot().unwrap();
    assert_eq!(hunt.hunters.iter().map(|h| (h.id, h.reserve_units)).collect::<Vec<_>>(),
        vec![(SimId(100), 3), (SimId(101), 6)]);
    assert_eq!(hunt.hunters.iter().map(|h| h.fatigue_points).collect::<Vec<_>>(),
        vec![Some(0), Some(2)]);
    assert_eq!((hunt.ledger.captures, hunt.ledger.attack_units, hunt.ledger.transferred_units), (1, 1, 4));
    let winners: Vec<_> = hunt.history.events.iter().filter_map(|e| match e.kind {
        HuntEventKind::Captured { hunter, prey, .. } => Some((hunter, prey)),
        _ => None,
    }).collect();
    assert_eq!(winners, vec![(SimId(101), SimId(1))]);
    let mobile = world.mobile_snapshot().unwrap();
    assert_eq!(mobile.base.grazers.iter().map(|g| g.id).collect::<Vec<_>>(), vec![SimId(2)]);
    assert_eq!(mobile.base.patches[0].biomass_units, 8);
    assert_eq!(mobile.population.totals.births, 0);
}
