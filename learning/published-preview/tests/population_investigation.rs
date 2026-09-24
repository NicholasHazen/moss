use moss_course_ecosystem::{
    CourseWorld, FounderTrait, PopulationConfig, PopulationError, Scenario,
    SimId, UpkeepTrait,
};

#[test]
fn rejected_mutation_cycle_preserves_world_and_future_reset() {
    let scenario = Scenario::generous_supply();
    let config = PopulationConfig::new(vec![
        FounderTrait { id: SimId(1), upkeep: UpkeepTrait::new(2).unwrap() },
        FounderTrait { id: SimId(2), upkeep: UpkeepTrait::new(1).unwrap() },
    ]);
    let mut world = CourseWorld::new_population(scenario.clone(), config.clone()).unwrap();
    world.step();
    let before = world.snapshot();
    let population_before = world.population_snapshot().unwrap();
    let mut invalid = config.clone();
    invalid.mutation_cycle = vec![2];
    assert_eq!(
        world.reset_with_population(scenario, invalid),
        Err(PopulationError::InvalidMutationCycle),
    );
    assert_eq!(world.snapshot(), before);
    assert_eq!(world.population_snapshot().unwrap(), population_before);
    world.reset();
    assert_eq!(world.snapshot().run, before.run + 1);
    assert_eq!(world.snapshot().tick, 0);
    assert_eq!(world.population_snapshot().unwrap().config, config);
}
