use moss_course_ecosystem::{
    Cell, CourseWorld, EventKind, HuntConfig, HunterSeed, MobileError, MobileScenario, Motion,
    RefugeConfig, RefugeId, RefugeSeed, RestPolicy, Scenario, SimId,
};

fn refuge(x: u32) -> RefugeConfig {
    RefugeConfig::new(vec![RefugeSeed {
        id: RefugeId(1),
        cell: Cell { x, y: 0 },
    }])
}

fn crossing() -> MobileScenario {
    let original = MobileScenario::short_journey();
    let mut scenario = original.scenario().clone();
    scenario.grazers[0].reserve_units = 6;
    scenario.patches[0].id = SimId(1000);
    scenario.patches[0].biomass_units = 5;
    scenario.patches[0].capacity_units = 5;
    let mut space = original.space().clone();
    space.bounds.width_cells = 2;
    space.bounds.height_cells = 1;
    space.grazers[0].motion.sensing_radius = 1;
    space.patches[0].id = SimId(1000);
    space.patches[0].cell = Cell { x: 1, y: 0 };
    MobileScenario::new(scenario, original.population().clone(), space).with_hunters(
        HuntConfig::new(vec![HunterSeed {
            id: SimId(100),
            reserve_units: 4,
            capacity_units: 10,
            maintenance_units_per_tick: 1,
            attack_units: 1,
            attack_effort_points: 2,
            cell: Cell { x: 1, y: 0 },
            motion: Motion {
                sensing_radius: 1,
                max_cells_per_tick: 0,
                travel_units_per_cell: 1,
            },
        }]),
    )
}

fn reports(world: &CourseWorld) -> String {
    format!(
        "{:?}{:?}{:?}{:?}{:?}",
        world.snapshot(),
        world.population_snapshot(),
        world.mobile_snapshot(),
        world.rest_snapshot(),
        (world.hunt_snapshot(), world.refuge_snapshot())
    )
}

fn stores(world: &CourseWorld) -> u64 {
    let s = world.snapshot();
    s.grazers
        .iter()
        .map(|g| u64::from(g.reserve_units))
        .sum::<u64>()
        + s.patches
            .iter()
            .map(|p| u64::from(p.biomass_units))
            .sum::<u64>()
        + world.hunt_snapshot().map_or(0, |h| {
            h.hunters.iter().map(|h| u64::from(h.reserve_units)).sum()
        })
}

#[test]
fn protected_prey_are_absent_from_the_local_observation_and_target() {
    let mut world =
        CourseWorld::new_mobile(MobileScenario::hunting_contention().with_refuges(refuge(0)))
            .unwrap();
    let initial = world.refuge_snapshot().unwrap();
    assert_eq!(initial.grazers.len(), 2);
    assert!(
        initial
            .grazers
            .iter()
            .all(|g| g.refuge == Some(RefugeId(1)) && g.assessed_tick == 0)
    );
    world.step();
    let h = world.hunt_snapshot().unwrap();
    assert_eq!(h.hunters.len(), 2);
    assert!(
        h.hunters
            .iter()
            .all(|h| h.observed_tick == Some(1) && h.visible_prey.is_empty() && h.target.is_none())
    );
    assert_eq!(
        (
            h.totals.captures,
            h.totals.attack_units,
            h.totals.effort_points
        ),
        (0, 0, 0)
    );
    assert_eq!(world.population_snapshot().unwrap().totals.births, 1);
}

#[test]
fn arrival_after_observation_rejects_a_funded_same_cell_capture() {
    let mut exposed = CourseWorld::new_mobile(crossing()).unwrap();
    let mut protected = CourseWorld::new_mobile(crossing().with_refuges(refuge(1))).unwrap();
    exposed.step();
    protected.step();
    let old = exposed.hunt_snapshot().unwrap();
    let h = protected.hunt_snapshot().unwrap();
    let m = protected.mobile_snapshot().unwrap();
    assert_eq!(old.totals.captures, 1);
    assert_eq!(old.hunters[0].reserve_units, 6);
    assert!(exposed.snapshot().grazers.is_empty());
    assert_eq!(h.hunters[0].visible_prey, old.hunters[0].visible_prey);
    assert_eq!(h.hunters[0].visible_prey[0].cell, Cell { x: 0, y: 0 });
    assert_eq!(h.hunters[0].target.unwrap().prey, SimId(1));
    assert_eq!(h.hunters[0].cell, m.grazers[0].cell);
    assert_eq!(m.grazers[0].cell, Cell { x: 1, y: 0 });
    assert_eq!(
        (
            h.hunters[0].reserve_units,
            h.ledger.attack_units,
            h.ledger.transferred_units
        ),
        (3, 0, 0)
    );
    assert_eq!(h.totals.captures, 0);
    assert_eq!(
        (
            m.base.grazers[0].reserve_units,
            m.base.patches[0].biomass_units
        ),
        (6, 3)
    );
    assert_eq!(
        protected.refuge_snapshot().unwrap().grazers[0].refuge,
        Some(RefugeId(1))
    );
    assert_eq!(
        stores(&protected)
            + m.base.ledger.maintenance_units
            + m.ledger.travel_units
            + h.ledger.maintenance_units,
        15
    );
}

#[test]
fn departing_protection_waits_for_the_next_fresh_local_observation() {
    let mut world = CourseWorld::new_mobile(crossing().with_refuges(refuge(0))).unwrap();
    world.step();
    let first = world.hunt_snapshot().unwrap();
    let r = world.refuge_snapshot().unwrap();
    assert_eq!(r.grazers[0].cell, Cell { x: 1, y: 0 });
    assert_eq!(r.grazers[0].refuge, None);
    assert_eq!(r.grazers[0].assessed_tick, 1);
    assert_eq!(first.hunters[0].observed_tick, Some(1));
    assert!(first.hunters[0].visible_prey.is_empty());
    assert!(first.hunters[0].target.is_none());
    assert_eq!(first.totals.captures, 0);
    world.step();
    let second = world.hunt_snapshot().unwrap();
    assert_eq!(second.hunters[0].observed_tick, Some(2));
    assert_eq!(second.hunters[0].target.unwrap().observed_tick, 2);
    assert_eq!(second.hunters[0].visible_prey[0].prey, SimId(1));
    assert_eq!(
        (second.totals.captures, second.totals.transferred_units),
        (1, 5)
    );
    assert!(world.refuge_snapshot().unwrap().grazers.is_empty());
}

#[test]
fn a_newborn_has_membership_at_birth_but_no_early_action_or_prey_eligibility() {
    let mut world =
        CourseWorld::new_mobile(MobileScenario::hunting_contention().with_refuges(refuge(0)))
            .unwrap();
    world.step();
    let first = world.refuge_snapshot().unwrap();
    assert_eq!(
        first.grazers.iter().map(|g| g.id).collect::<Vec<_>>(),
        vec![SimId(1), SimId(2), SimId(1001)]
    );
    assert!(
        first
            .grazers
            .iter()
            .all(|g| g.refuge == Some(RefugeId(1)) && g.assessed_tick == 1)
    );
    let s = world.snapshot();
    assert_eq!(s.grazers[2].reserve_units, 4);
    assert!(!s.history.events.iter().any(|e| matches!(
        e.kind,
        EventKind::Maintenance {
            grazer: SimId(1001),
            ..
        } | EventKind::Meal {
            grazer: SimId(1001),
            ..
        }
    )));
    world.step();
    assert_eq!(world.snapshot().grazers[2].reserve_units, 5);
    assert!(world.snapshot().history.events.iter().any(|e| e.tick == 2
        && matches!(
            e.kind,
            EventKind::Meal {
                grazer: SimId(1001),
                units: 2,
                ..
            }
        )));
    assert!(
        world
            .hunt_snapshot()
            .unwrap()
            .hunters
            .iter()
            .all(|h| h.visible_prey.is_empty())
    );
    assert_eq!(world.hunt_snapshot().unwrap().totals.captures, 0);
}

#[test]
fn invalid_refuge_resets_preserve_all_reports_and_the_saved_mode() {
    let spec = crossing().with_refuges(refuge(1));
    let mut world = CourseWorld::new_mobile(spec.clone()).unwrap();
    world.step();
    let before = reports(&world);
    let cases = [
        (
            vec![RefugeSeed {
                id: RefugeId(0),
                cell: Cell { x: 0, y: 0 },
            }],
            MobileError::ZeroRefugeId,
        ),
        (
            vec![RefugeSeed {
                id: RefugeId(1),
                cell: Cell { x: 2, y: 0 },
            }],
            MobileError::RefugeOutOfBounds(RefugeId(1)),
        ),
        (
            vec![
                RefugeSeed {
                    id: RefugeId(1),
                    cell: Cell { x: 0, y: 0 },
                },
                RefugeSeed {
                    id: RefugeId(1),
                    cell: Cell { x: 1, y: 0 },
                },
            ],
            MobileError::DuplicateRefugeId(RefugeId(1)),
        ),
        (
            vec![
                RefugeSeed {
                    id: RefugeId(1),
                    cell: Cell { x: 0, y: 0 },
                },
                RefugeSeed {
                    id: RefugeId(2),
                    cell: Cell { x: 0, y: 0 },
                },
            ],
            MobileError::SharedRefugeCell(Cell { x: 0, y: 0 }),
        ),
    ];
    for (sites, expected) in cases {
        let invalid = crossing().with_refuges(RefugeConfig::new(sites));
        assert_eq!(world.reset_with_mobile(invalid.clone()), Err(expected));
        assert_eq!(reports(&world), before);
        assert!(matches!(CourseWorld::new_mobile(invalid), Err(error) if error == expected));
    }
    world.reset();
    let reset = world.refuge_snapshot().unwrap();
    assert_eq!((reset.run, reset.tick), (2, 0));
    assert_eq!(reset.config, spec.refuges().unwrap().clone());
    world.reset_with_mobile(crossing()).unwrap();
    assert!(world.refuge_snapshot().is_none());
    assert_eq!(world.snapshot().run, 3);
}

#[test]
fn reports_are_owned_and_repeated_reads_neither_advance_nor_protect() {
    let spec = crossing().with_refuges(refuge(1));
    let mut observed = CourseWorld::new_mobile(spec.clone()).unwrap();
    let mut quiet = CourseWorld::new_mobile(spec).unwrap();
    let mut saved = observed.refuge_snapshot().unwrap();
    saved.grazers[0].refuge = Some(RefugeId(99));
    saved.config.sites.clear();
    for _ in 0..3 {
        let before = reports(&observed);
        for _ in 0..100 {
            assert_eq!(reports(&observed), before);
        }
        observed.step();
        quiet.step();
        assert_eq!(reports(&observed), reports(&quiet));
    }
    assert_eq!((saved.run, saved.tick), (1, 0));
    observed.reset();
    assert_eq!(saved.grazers[0].refuge, Some(RefugeId(99)));
    assert_eq!(observed.refuge_snapshot().unwrap().run, 2);
}

#[test]
fn enabled_empty_is_distinct_from_disabled_without_changing_existing_outcomes() {
    let spec = MobileScenario::hunting_contention();
    let mut off = CourseWorld::new_mobile(spec.clone()).unwrap();
    let mut empty = CourseWorld::new_mobile(spec.with_refuges(RefugeConfig::new(vec![]))).unwrap();
    assert!(off.refuge_snapshot().is_none());
    let r = empty.refuge_snapshot().unwrap();
    assert!(r.config.sites.is_empty());
    assert!(r.grazers.iter().all(|g| g.refuge.is_none()));
    for _ in 0..6 {
        off.step();
        empty.step();
        assert_eq!(off.snapshot(), empty.snapshot());
        assert_eq!(off.population_snapshot(), empty.population_snapshot());
        assert_eq!(off.rest_snapshot(), empty.rest_snapshot());
        assert_eq!(off.hunt_snapshot(), empty.hunt_snapshot());
        let a = off.mobile_snapshot().unwrap();
        let b = empty.mobile_snapshot().unwrap();
        assert_eq!(
            (a.grazers, a.ledger, a.totals, a.history),
            (b.grazers, b.ledger, b.totals, b.history)
        );
    }
    let mut basic = CourseWorld::new(Scenario::limited_supply()).unwrap();
    assert!(basic.refuge_snapshot().is_none());
    basic
        .reset_with_mobile(crossing().with_refuges(refuge(1)))
        .unwrap();
    assert!(basic.refuge_snapshot().is_some());
    basic.reset_with(Scenario::limited_supply()).unwrap();
    assert!(basic.refuge_snapshot().is_none());
}

#[test]
fn static_site_ids_do_not_consume_child_ids_or_change_non_hunting_costs() {
    let spec = MobileScenario::hunting_contention();
    let ordinary = MobileScenario::new(
        spec.scenario().clone(),
        spec.population().clone(),
        spec.space().clone(),
    )
    .with_rest(RestPolicy::default());
    let mut off = CourseWorld::new_mobile(ordinary.clone()).unwrap();
    let mut protected =
        CourseWorld::new_mobile(ordinary.with_refuges(RefugeConfig::new(vec![RefugeSeed {
            id: RefugeId(u32::MAX),
            cell: Cell { x: 0, y: 0 },
        }])))
        .unwrap();
    for _ in 0..8 {
        off.step();
        protected.step();
        assert_eq!(off.snapshot(), protected.snapshot());
        assert_eq!(off.population_snapshot(), protected.population_snapshot());
        assert_eq!(off.rest_snapshot(), protected.rest_snapshot());
    }
    assert!(protected.population_snapshot().unwrap().totals.births > 0);
    assert!(
        protected
            .snapshot()
            .history
            .events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Maintenance { .. }))
    );
}

#[test]
fn protection_does_not_prevent_starvation_or_create_missing_food() {
    let base = crossing();
    let mut scenario = base.scenario().clone();
    scenario.grazers[0].reserve_units = 2;
    scenario.patches[0].biomass_units = 0;
    let spec = MobileScenario::new(scenario, base.population().clone(), base.space().clone())
        .with_hunters(base.hunters().unwrap().clone())
        .with_refuges(refuge(0));
    let mut world = CourseWorld::new_mobile(spec).unwrap();
    world.step();
    assert_eq!(world.snapshot().grazers[0].reserve_units, 1);
    assert_eq!(
        world.refuge_snapshot().unwrap().grazers[0].refuge,
        Some(RefugeId(1))
    );
    world.step();
    assert!(world.refuge_snapshot().unwrap().grazers.is_empty());
    assert_eq!(world.population_snapshot().unwrap().totals.starvations, 1);
    assert_eq!(world.hunt_snapshot().unwrap().totals.captures, 0);
    assert_eq!(world.snapshot().ledger.eaten_units, 0);
}

#[test]
fn current_protection_does_not_turn_unretained_capture_history_into_zero() {
    let spec = crossing().with_refuges(refuge(1));
    let mut retained = CourseWorld::new_mobile(spec.clone()).unwrap();
    let mut config = spec.hunters().unwrap().clone();
    config.history_limit = 0;
    let mut absent = CourseWorld::new_mobile(spec.with_hunters(config)).unwrap();
    retained.step();
    absent.step();
    assert_eq!(retained.refuge_snapshot(), absent.refuge_snapshot());
    assert_eq!(
        retained
            .hunt_snapshot()
            .unwrap()
            .history
            .captures_between(1, 1),
        Some(0)
    );
    let h = absent.hunt_snapshot().unwrap();
    assert_eq!(h.totals.captures, 0);
    assert!(h.history.events.is_empty());
    assert!(h.history.evicted_events > 0);
    assert_eq!(h.history.captures_between(1, 1), None);
    assert_eq!(
        retained
            .hunt_snapshot()
            .unwrap()
            .history
            .captures_between(1, 2),
        None
    );
}

#[test]
fn authored_site_and_actor_order_does_not_change_complete_reports() {
    let mut spec = MobileScenario::hunting_contention();
    let sites = RefugeConfig::new(vec![
        RefugeSeed {
            id: RefugeId(7),
            cell: Cell { x: 1, y: 0 },
        },
        RefugeSeed {
            id: RefugeId(1),
            cell: Cell { x: 0, y: 0 },
        },
    ]);
    let mut scenario = spec.scenario().clone();
    scenario.grazers.reverse();
    scenario.patches.reverse();
    let mut population = spec.population().clone();
    population.founders.reverse();
    let mut space = spec.space().clone();
    space.grazers.reverse();
    space.patches.reverse();
    let mut hunters = spec.hunters().unwrap().clone();
    hunters.hunters.reverse();
    let mut reverse_sites = sites.clone();
    reverse_sites.sites.reverse();
    let reversed = MobileScenario::new(scenario, population, space)
        .with_rest(*spec.rest().unwrap())
        .with_hunters(hunters)
        .with_refuges(reverse_sites);
    spec = spec.with_refuges(sites);
    let mut a = CourseWorld::new_mobile(spec).unwrap();
    let mut b = CourseWorld::new_mobile(reversed).unwrap();
    for _ in 0..8 {
        assert_eq!(reports(&a), reports(&b));
        a.step();
        b.step();
    }
}

#[test]
fn matched_placements_change_literal_population_outcomes_without_resource_gifts() {
    for (cell, expected) in [
        (None, (0, 4, 3, 60, 53, 15, 62, 13, 36)),
        (
            Some(Cell { x: 3, y: 2 }),
            (1, 5, 1, 168, 171, 17, 38, 8, 55),
        ),
        (Some(Cell { x: 4, y: 2 }), (0, 4, 3, 55, 51, 15, 62, 10, 36)),
    ] {
        let mut spec = MobileScenario::meadow()
            .with_rest(RestPolicy::default())
            .with_hunters(HuntConfig::meadow());
        if let Some(cell) = cell {
            spec = spec.with_refuges(RefugeConfig::new(vec![RefugeSeed {
                id: RefugeId(1),
                cell,
            }]));
        }
        let mut world = CourseWorld::new_mobile(spec).unwrap();
        assert_eq!(stores(&world), 128);
        let (mut growth, mut upkeep) = (0, 0);
        for _ in 0..120 {
            let before = stores(&world);
            world.step();
            let m = world.mobile_snapshot().unwrap();
            let h = world.hunt_snapshot().unwrap();
            growth += m.base.ledger.added_units;
            upkeep += m.base.ledger.maintenance_units;
            assert_eq!(
                stores(&world)
                    + m.base.ledger.maintenance_units
                    + m.ledger.travel_units
                    + m.population.ledger.dissipated_units
                    + h.ledger.maintenance_units
                    + h.ledger.travel_units
                    + h.ledger.attack_units,
                before + m.base.ledger.added_units
            );
            assert_eq!(
                m.population.living as u64 + m.population.totals.starvations + h.totals.captures,
                4 + m.population.totals.births
            );
        }
        let m = world.mobile_snapshot().unwrap();
        let h = world.hunt_snapshot().unwrap();
        assert_eq!(
            (
                m.population.living,
                m.population.totals.starvations,
                h.totals.captures,
                growth,
                upkeep,
                m.totals.travel_units,
                h.totals.maintenance_units,
                h.totals.travel_units,
                stores(&world)
            ),
            expected
        );
        assert_eq!(
            (
                m.population.totals.births,
                m.population.totals.capacity_blocked_pairs,
                m.population.totals.capacity_blocked_ticks,
                h.totals.hunter_starvations
            ),
            (3, 0, 0, 2)
        );
        assert_eq!(h.history.captures_between(1, 120), Some(expected.2));
        if expected.0 == 1 {
            assert_eq!(
                (m.base.grazers[0].id, m.base.grazers[0].reserve_units),
                (SimId(2), 24)
            );
            assert_eq!(
                (
                    m.base.history.complete_after_tick,
                    m.base.history.evicted_events
                ),
                (68, 209)
            );
            assert_eq!(m.base.history.starvations_between(1, 120), None);
        } else {
            assert_eq!(m.base.history.starvations_between(1, 120), Some(4));
        }
    }
}
