use moss_course_ecosystem::{CourseWorld, MobileScenario};

fn main() {
    let mut short = CourseWorld::new_mobile(MobileScenario::short_journey()).unwrap();
    println!(
        "short: reserve5, upkeep1, speed1/cost1, sight3, capacity8/meal2; patch100 at(2,0), biomass3, no growth"
    );
    for _ in 0..3 {
        short.step();
        let state = short.mobile_snapshot().unwrap();
        println!(
            "tick{} cell={:?} reserve{} biomass{} travelled{} paid{} meal{}",
            state.tick,
            state.grazers[0].cell,
            state.base.grazers[0].reserve_units,
            state.base.patches[0].biomass_units,
            state.ledger.travel_cells,
            state.ledger.travel_units,
            state.base.ledger.eaten_units
        );
    }
    let initial = MobileScenario::meadow();
    println!("meadow horizon120; complete authored inputs={initial:?}");
    let mut world = CourseWorld::new_mobile(initial).unwrap();
    let (mut growth, mut upkeep, mut transferred, mut cost) = (0, 0, 0, 0);
    for _ in 0..120 {
        world.step();
        let state = world.mobile_snapshot().unwrap();
        growth += state.base.ledger.added_units;
        upkeep += state.base.ledger.maintenance_units;
        transferred += state.population.ledger.transferred_units;
        cost += state.population.ledger.dissipated_units;
        if [1, 2, 4, 8, 12, 40, 80, 120].contains(&state.tick) {
            let positions: Vec<_> = state
                .grazers
                .iter()
                .map(|actor| (actor.id.0, actor.cell.x, actor.cell.y))
                .collect();
            let reserves: u64 = state
                .base
                .grazers
                .iter()
                .map(|actor| u64::from(actor.reserve_units))
                .sum();
            let biomass: u64 = state
                .base
                .patches
                .iter()
                .map(|patch| u64::from(patch.biomass_units))
                .sum();
            println!(
                "tick{} living{} juvenile{} births{} deaths{} travel_cells{} travel_units{} target_changes{} reserves{} biomass{} positions={positions:?}",
                state.tick,
                state.population.living,
                state.population.juveniles,
                state.population.totals.births,
                state.population.totals.starvations,
                state.totals.travel_cells,
                state.totals.travel_units,
                state.totals.target_changes,
                reserves,
                biomass
            );
        }
    }
    let state = world.mobile_snapshot().unwrap();
    println!(
        "actual growth{growth} upkeep{upkeep} birth_transfer{transferred} birth_cost{cost} cap_pairs{} cap_ticks{}",
        state.population.totals.capacity_blocked_pairs,
        state.population.totals.capacity_blocked_ticks
    );
    println!(
        "coverage base({},{}] lost{}; population({},{}] lost{}; mobile({},{}] lost{}; full-range travel={:?}",
        state.base.history.complete_after_tick,
        state.base.history.collected_through_tick,
        state.base.history.evicted_events,
        state.population.history.complete_after_tick,
        state.population.history.collected_through_tick,
        state.population.history.evicted_events,
        state.history.complete_after_tick,
        state.history.collected_through_tick,
        state.history.evicted_events,
        state.history.travelled_cells_between(1, 120)
    );
}
