//! A deterministic 40-tick comparison, not sampled evidence of biological evolution.

use moss_course_ecosystem::{
    CourseWorld, Daylight, FounderTrait, GrazerSeed, PatchSeed, PopulationConfig, Scenario, SimId,
    UpkeepTrait,
};

const HORIZON: u64 = 40;
const COHORTS: [[u8; 2]; 4] = [[1, 1], [1, 2], [2, 2], [1, 3]];

fn comparison(traits: [u8; 2], pulsed: bool) -> (Scenario, PopulationConfig) {
    let founders: Vec<_> = traits
        .into_iter()
        .enumerate()
        .map(|(index, value)| FounderTrait {
            id: SimId(index as u32 + 1),
            upkeep: UpkeepTrait::new(value).unwrap(),
        })
        .collect();
    let scenario = Scenario {
        daylight: Daylight::new(4, if pulsed { 2 } else { 4 }).unwrap(),
        grazers: founders
            .iter()
            .map(|founder| GrazerSeed {
                id: founder.id,
                reserve_units: 5,
                capacity_units: 8,
                maintenance_units_per_tick: founder.upkeep.units(),
                meal_units_per_tick: 2,
                feeding_site: Some(SimId(100)),
            })
            .collect(),
        patches: vec![PatchSeed {
            id: SimId(100),
            biomass_units: 0,
            capacity_units: 20,
            growth_units_per_lit_tick: if pulsed { 12 } else { 6 },
        }],
        history_limit: 128,
    };
    (scenario, PopulationConfig::new(founders))
}

fn main() {
    println!(
        "Deterministic authored comparison; horizon={HORIZON} ticks; founder upkeep cohorts={COHORTS:?}."
    );
    println!(
        "Each pair uses IDs1/2, reserve5, capacity8, meal2, fixed contact100; patch100 starts0, capacity20."
    );
    println!(
        "Each 4-tick supply budget is24 requested units: steady6 on4 lit ticks; pulsed12 on2 lit ticks then2 dark."
    );
    println!(
        "Policy: contribution2 + cost1 per parent, positive remainder, maturation2, cooldown3, child capacity8/meal2, maxliving8."
    );
    println!(
        "Mutation cycle[0], alternating lower/upper-ID donor on accepted births, upkeep trait1..3; both history limits128."
    );
    println!(
        "All rows begin a fresh run1; no random draws, movement, rest, or predation. Accepted growth can differ because capacity clips input."
    );
    println!(
        "cohort env living juvenile adult births deaths traitcounts[1,2,3] growth upkeep transfer birthcost cap_pairs cap_ticks reserves biomass"
    );
    for traits in COHORTS {
        for pulsed in [false, true] {
            let (scenario, config) = comparison(traits, pulsed);
            let mut course = CourseWorld::new_population(scenario, config).unwrap();
            let (mut growth, mut upkeep, mut transferred, mut cost) = (0, 0, 0, 0);
            for _ in 0..HORIZON {
                course.step();
                let state = course.snapshot();
                let population = course.population_snapshot().unwrap();
                growth += state.ledger.added_units;
                upkeep += state.ledger.maintenance_units;
                transferred += population.ledger.transferred_units;
                cost += population.ledger.dissipated_units;
            }
            let state = course.snapshot();
            let population = course.population_snapshot().unwrap();
            let mut trait_counts = [0; 3];
            for individual in &population.individuals {
                trait_counts[usize::from(individual.upkeep.value() - 1)] += 1;
            }
            let reserves: u64 = state
                .grazers
                .iter()
                .map(|grazer| u64::from(grazer.reserve_units))
                .sum();
            let environment = if pulsed { "pulsed" } else { "steady" };
            println!(
                "{traits:?} {environment} {} {} {} {} {} {trait_counts:?} {growth} {upkeep} {transferred} {cost} {} {} {reserves} {}",
                population.living,
                population.juveniles,
                population.adults,
                population.totals.births,
                population.totals.starvations,
                population.totals.capacity_blocked_pairs,
                population.totals.capacity_blocked_ticks,
                state.patches[0].biomass_units
            );
            println!(
                "  complete event ticks: base({},{}], lost{}; population({},{}], lost{}; births1..40={:?}; starvations1..40={:?}",
                state.history.complete_after_tick,
                state.history.collected_through_tick,
                state.history.evicted_events,
                population.history.complete_after_tick,
                population.history.collected_through_tick,
                population.history.evicted_events,
                population.history.births_between(1, HORIZON),
                state.history.starvations_between(1, HORIZON)
            );
        }
    }
}
