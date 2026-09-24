// Save as work/fieldwork/tests/refuge_investigation.rs in your continuing project.
// From the Moss repository root:
// cargo +1.93.1 test --offline --locked --manifest-path work/fieldwork/Cargo.toml --test refuge_investigation
use moss_course_ecosystem::{
    Cell, CourseWorld, Daylight, FounderTrait, GrazerPlacement, GrazerSeed,
    GridBounds, HuntConfig, HunterSeed, MobileScenario, Motion, PatchPlacement,
    PatchSeed, PopulationConfig, RefugeConfig, RefugeId, RefugeSeed,
    Scenario, SimId, SpatialConfig, UpkeepTrait,
};

fn fixture(refuge_cell: Option<Cell>) -> MobileScenario {
    let start = Cell { x: 0, y: 0 };
    let destination = Cell { x: 1, y: 0 };
    let motion = Motion {
        sensing_radius: 1, max_cells_per_tick: 1, travel_units_per_cell: 1,
    };
    let scenario = Scenario {
        daylight: Daylight::new(1, 0).unwrap(),
        grazers: vec![GrazerSeed {
            id: SimId(1), reserve_units: 6, capacity_units: 8,
            maintenance_units_per_tick: 1, meal_units_per_tick: 2,
            feeding_site: None,
        }],
        patches: vec![PatchSeed {
            id: SimId(1000), biomass_units: 5, capacity_units: 5,
            growth_units_per_lit_tick: 0,
        }],
        history_limit: 128,
    };
    let population = PopulationConfig {
        founders: vec![FounderTrait {
            id: SimId(1), upkeep: UpkeepTrait::new(1).unwrap(),
        }],
        maturation_ticks: 2, cooldown_ticks: 3,
        contribution_units_per_parent: 2, birth_cost_units_per_parent: 1,
        child_capacity_units: 8, child_meal_units_per_tick: 2,
        max_living: 8, mutation_cycle: vec![0], history_limit: 128,
    };
    let space = SpatialConfig {
        bounds: GridBounds { width_cells: 2, height_cells: 1 },
        grazers: vec![GrazerPlacement { id: SimId(1), cell: start, motion }],
        patches: vec![PatchPlacement { id: SimId(1000), cell: destination }],
        newborn_motion: motion,
    };
    let initial = MobileScenario::new(scenario, population, space)
        .with_hunters(HuntConfig::new(vec![HunterSeed {
            id: SimId(100), reserve_units: 4, capacity_units: 10,
            maintenance_units_per_tick: 1, attack_units: 1,
            attack_effort_points: 2, cell: destination,
            motion: Motion { max_cells_per_tick: 0, ..motion },
        }]));
    match refuge_cell {
        None => initial,
        Some(cell) => initial.with_refuges(RefugeConfig::new(vec![RefugeSeed {
            id: RefugeId(7), cell,
        }])),
    }
}

#[test]
fn entering_refuge_rejects_a_previously_observed_same_cell_capture() {
    let plain = fixture(None);
    let protected = fixture(Some(Cell { x: 1, y: 0 }));
    assert_eq!(plain.scenario(), protected.scenario());
    assert_eq!(plain.population(), protected.population());
    assert_eq!(plain.space(), protected.space());
    assert_eq!(plain.hunters(), protected.hunters());
    let mut control = CourseWorld::new_mobile(plain).unwrap();
    control.step();
    let control_mobile = control.mobile_snapshot().unwrap();
    let control_hunt = control.hunt_snapshot().unwrap();
    assert!(control.refuge_snapshot().is_none());
    assert!(control_mobile.base.grazers.is_empty());
    assert_eq!(control_mobile.base.patches[0].biomass_units, 5);
    assert_eq!(control_hunt.hunters[0].reserve_units, 6);
    assert_eq!((control_hunt.ledger.captures, control_hunt.ledger.attack_units,
        control_hunt.ledger.transferred_units), (1, 1, 4));
    assert_eq!(control_mobile.base.ledger.eaten_units, 0);

    let mut world = CourseWorld::new_mobile(protected).unwrap();
    world.step();
    let mobile = world.mobile_snapshot().unwrap();
    let hunt = world.hunt_snapshot().unwrap();
    let shelter = world.refuge_snapshot().unwrap();
    assert_eq!((shelter.run, shelter.tick), (mobile.run, 1));
    assert_eq!((hunt.run, hunt.tick), (mobile.run, 1));
    assert_eq!(hunt.hunters[0].visible_prey[0].prey, SimId(1));
    assert_eq!(hunt.hunters[0].visible_prey[0].cell, Cell { x: 0, y: 0 });
    assert_eq!(hunt.hunters[0].visible_prey[0].reserve_units, 5);
    let target = hunt.hunters[0].target.unwrap();
    assert_eq!((target.prey, target.observed_tick), (SimId(1), 1));
    assert_eq!(target.observed_cell, Cell { x: 0, y: 0 });
    assert_eq!(hunt.hunters[0].cell, Cell { x: 1, y: 0 });
    assert_eq!(mobile.grazers[0].cell, hunt.hunters[0].cell);
    assert_eq!((shelter.grazers[0].id, shelter.grazers[0].refuge,
        shelter.grazers[0].assessed_tick), (SimId(1), Some(RefugeId(7)), 1));
    assert_eq!(shelter.grazers[0].cell, Cell { x: 1, y: 0 });
    assert_eq!(mobile.base.grazers[0].reserve_units, 6);
    assert_eq!(mobile.base.patches[0].biomass_units, 3);
    assert_eq!(mobile.base.ledger.eaten_units, 2);
    assert_eq!(hunt.hunters[0].reserve_units, 3);
    assert_eq!(hunt.hunters[0].last_capture_tick, None);
    assert_eq!((hunt.ledger.captures, hunt.ledger.attack_units,
        hunt.ledger.transferred_units), (0, 0, 0));
    assert_eq!(hunt.history.captures_between(1, 1), Some(0));
    assert_eq!((mobile.base.ledger.maintenance_units, mobile.ledger.travel_units,
        hunt.ledger.maintenance_units, hunt.ledger.travel_units), (1, 1, 1, 0));
    let stores = u64::from(mobile.base.grazers[0].reserve_units)
        + u64::from(mobile.base.patches[0].biomass_units)
        + u64::from(hunt.hunters[0].reserve_units);
    assert_eq!(stores, 12);
    assert_eq!(stores + mobile.base.ledger.maintenance_units
        + hunt.ledger.maintenance_units + mobile.ledger.travel_units
        + hunt.ledger.travel_units + hunt.ledger.attack_units
        + mobile.population.ledger.dissipated_units,
        15 + mobile.base.ledger.added_units);
}

#[test]
fn leaving_refuge_needs_a_fresh_observation_before_capture() {
    let mut world = CourseWorld::new_mobile(fixture(Some(Cell { x: 0, y: 0 }))).unwrap();
    let initial = world.refuge_snapshot().unwrap();
    assert_eq!((initial.grazers[0].refuge, initial.grazers[0].assessed_tick),
        (Some(RefugeId(7)), 0));
    world.step();
    let departure = world.refuge_snapshot().unwrap();
    let mobile = world.mobile_snapshot().unwrap();
    let hunt = world.hunt_snapshot().unwrap();
    assert_eq!((departure.grazers[0].refuge, departure.grazers[0].assessed_tick), (None, 1));
    assert_eq!(departure.grazers[0].cell, Cell { x: 1, y: 0 });
    assert_eq!(hunt.hunters[0].observed_tick, Some(1));
    assert!(hunt.hunters[0].visible_prey.is_empty());
    assert_eq!(hunt.hunters[0].target, None);
    assert_eq!(mobile.grazers[0].cell, hunt.hunters[0].cell);
    assert_eq!(mobile.base.grazers[0].reserve_units, 6);
    assert_eq!(hunt.hunters[0].reserve_units, 3);
    assert_eq!(hunt.ledger.captures, 0);
    assert_eq!(hunt.ledger.attack_units, 0);
    assert_eq!(mobile.base.patches[0].biomass_units, 3);

    world.step();
    let later_hunt = world.hunt_snapshot().unwrap();
    assert_eq!(later_hunt.hunters[0].observed_tick, Some(2));
    assert_eq!(later_hunt.hunters[0].visible_prey[0].prey, SimId(1));
    assert_eq!(later_hunt.hunters[0].visible_prey[0].reserve_units, 5);
    assert_eq!(later_hunt.hunters[0].last_capture_tick, Some(2));
    assert_eq!(later_hunt.hunters[0].reserve_units, 6);
    assert_eq!((later_hunt.ledger.captures, later_hunt.ledger.attack_units,
        later_hunt.ledger.transferred_units), (1, 1, 5));
    assert_eq!(later_hunt.history.captures_between(1, 2), Some(1));
    assert!(world.refuge_snapshot().unwrap().grazers.is_empty());
    assert!(world.mobile_snapshot().unwrap().base.grazers.is_empty());
    assert_eq!(departure.tick, 1);
    assert_eq!(departure.grazers[0].id, SimId(1));
}

// Independent adjacent investigation: shelter crossed between sampled cells.
#[test]
fn crossing_shelter_does_not_protect_an_unprotected_destination() {
    let base = fixture(None);
    let mut space = base.space().clone();
    space.bounds.width_cells = 3;
    space.grazers[0].motion.sensing_radius = 2;
    space.grazers[0].motion.max_cells_per_tick = 2;
    space.patches[0].cell = Cell { x: 2, y: 0 };
    let mut hunters = base.hunters().unwrap().clone();
    hunters.hunters[0].cell = Cell { x: 2, y: 0 };
    hunters.hunters[0].motion.sensing_radius = 2;
    let input = MobileScenario::new(base.scenario().clone(), base.population().clone(), space)
        .with_hunters(hunters)
        .with_refuges(RefugeConfig::new(vec![RefugeSeed {
            id: RefugeId(7), cell: Cell { x: 1, y: 0 },
        }]));
    let mut world = CourseWorld::new_mobile(input).unwrap();
    world.step();
    let mobile = world.mobile_snapshot().unwrap();
    let hunt = world.hunt_snapshot().unwrap();
    // Protection uses the destination cell at the capture boundary.
    assert_eq!(hunt.ledger.captures, 1);
    assert!(mobile.base.grazers.is_empty());
    assert!(world.refuge_snapshot().unwrap().grazers.is_empty());
    assert_eq!(hunt.hunters[0].cell, Cell { x: 2, y: 0 });
    assert_eq!(hunt.hunters[0].target.unwrap().observed_cell, Cell { x: 0, y: 0 });
    assert_eq!(hunt.hunters[0].reserve_units, 5);
    assert_eq!(mobile.base.patches[0].biomass_units, 5);
    assert_eq!((mobile.ledger.travel_units, hunt.ledger.attack_units,
        hunt.ledger.transferred_units), (2, 1, 3));
    let stores = u64::from(hunt.hunters[0].reserve_units)
        + u64::from(mobile.base.patches[0].biomass_units);
    assert_eq!(stores + mobile.base.ledger.maintenance_units
        + hunt.ledger.maintenance_units + mobile.ledger.travel_units
        + hunt.ledger.travel_units + hunt.ledger.attack_units
        + mobile.population.ledger.dissipated_units,
        15 + mobile.base.ledger.added_units);
}
