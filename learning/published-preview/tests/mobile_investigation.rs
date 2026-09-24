use moss_course_ecosystem::{
    Cell, CourseWorld, Daylight, FounderTrait, GrazerPlacement, GrazerSeed,
    GridBounds, MobileScenario, Motion, PatchPlacement, PatchSeed,
    PopulationConfig, Scenario, SimId, SpatialConfig, UpkeepTrait,
};

#[test]
fn stale_local_claim_cannot_take_a_second_meal_or_skip_to_unseen_food() {
    let motion = Motion {
        sensing_radius: 1,
        max_cells_per_tick: 1,
        travel_units_per_cell: 1,
    };
    let scenario = Scenario {
        daylight: Daylight::new(1, 0).unwrap(),
        grazers: [1, 2].into_iter().map(|id| GrazerSeed {
            id: SimId(id), reserve_units: 4, capacity_units: 8,
            maintenance_units_per_tick: 1, meal_units_per_tick: 2,
            feeding_site: None,
        }).collect(),
        patches: [(100, 2), (101, 5)].into_iter().map(|(id, units)| PatchSeed {
            id: SimId(id), biomass_units: units, capacity_units: 8,
            growth_units_per_lit_tick: 0,
        }).collect(),
        history_limit: 128,
    };
    let mut population = PopulationConfig::new([1, 2].into_iter().map(|id| FounderTrait {
        id: SimId(id), upkeep: UpkeepTrait::new(1).unwrap(),
    }).collect());
    population.max_living = 2;
    let space = SpatialConfig {
        bounds: GridBounds { width_cells: 3, height_cells: 1 },
        grazers: [1, 2].into_iter().map(|id| GrazerPlacement {
            id: SimId(id), cell: Cell { x: 0, y: 0 }, motion,
        }).collect(),
        patches: vec![
            PatchPlacement { id: SimId(100), cell: Cell { x: 1, y: 0 } },
            PatchPlacement { id: SimId(101), cell: Cell { x: 2, y: 0 } },
        ],
        newborn_motion: motion,
    };
    let mut world = CourseWorld::new_mobile(MobileScenario::new(scenario, population, space)).unwrap();
    world.step();
    let first = world.mobile_snapshot().unwrap();
    assert_eq!(first.tick, 1);
    assert_eq!(first.base.grazers.iter().map(|g| g.reserve_units).collect::<Vec<_>>(), vec![4, 2]);
    assert_eq!(first.base.patches.iter().map(|p| p.biomass_units).collect::<Vec<_>>(), vec![0, 5]);
    assert_eq!((first.ledger.travel_cells, first.ledger.travel_units), (2, 2));
    assert_eq!(first.base.ledger.eaten_units, 2);
    for grazer in &first.grazers {
        assert_eq!(grazer.cell, Cell { x: 1, y: 0 });
        let target = grazer.target.unwrap();
        assert_eq!((target.patch, target.observed_tick), (SimId(100), 1));
        assert_eq!(grazer.visible_patches.len(), 1);
        assert_eq!(grazer.visible_patches[0].patch, SimId(100));
        assert_eq!(grazer.visible_patches[0].biomass_units, 2);
    }
    world.step();
    let second = world.mobile_snapshot().unwrap();
    assert_eq!(second.tick, 2);
    assert_eq!(second.base.grazers.iter().map(|g| g.reserve_units).collect::<Vec<_>>(), vec![4, 2]);
    assert_eq!(second.base.patches.iter().map(|p| p.biomass_units).collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(second.base.ledger.eaten_units, 4);
    assert_eq!(second.population.totals.births, 0);
    for grazer in &second.grazers {
        assert_eq!(grazer.cell, Cell { x: 2, y: 0 });
        let target = grazer.target.unwrap();
        assert_eq!((target.patch, target.observed_tick), (SimId(101), 2));
    }
}
