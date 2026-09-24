//! Scalar presentation adapter for the checked continuing course project.
//! Fixtures are authored inputs; all life, resource and inheritance transitions
//! execute in CourseWorld. Observation never advances the world.
use std::cell::RefCell;

use course::{
    CourseWorld, Daylight, EventKind, FounderTrait, GrazerSeed, PatchSeed, PopulationConfig,
    PopulationEventKind, PopulationSnapshot, Scenario, SimId, Snapshot, Stage, UpkeepTrait,
};

fn fixture(environment: u32, cohort: u32, mutation: u32) -> Option<(Scenario, PopulationConfig)> {
    if environment > 1 || mutation > 1 {
        return None;
    }
    let rates = match cohort {
        0 => [1, 1],
        1 => [1, 2],
        2 => [2, 2],
        3 => [1, 3],
        _ => return None,
    };
    let grazers = rates
        .iter()
        .enumerate()
        .map(|(index, rate)| GrazerSeed {
            id: SimId(index as u32 + 1),
            reserve_units: 5,
            capacity_units: 8,
            maintenance_units_per_tick: u32::from(*rate),
            meal_units_per_tick: 2,
            feeding_site: Some(SimId(100)),
        })
        .collect();
    let scenario = Scenario {
        daylight: Daylight::new(4, if environment == 0 { 4 } else { 2 }).ok()?,
        grazers,
        patches: vec![PatchSeed {
            id: SimId(100),
            biomass_units: 0,
            capacity_units: 20,
            growth_units_per_lit_tick: if environment == 0 { 6 } else { 12 },
        }],
        history_limit: 128,
    };
    let mut config = PopulationConfig::new(
        rates
            .iter()
            .enumerate()
            .map(|(index, rate)| FounderTrait {
                id: SimId(index as u32 + 1),
                upkeep: UpkeepTrait::new(*rate).expect("valid authored rate"),
            })
            .collect(),
    );
    config.mutation_cycle = if mutation == 0 {
        vec![0]
    } else {
        vec![-1, 0, 1, 0]
    };
    Some((scenario, config))
}

struct BrowserPopulation {
    simulation: CourseWorld,
    state: Snapshot,
    population: PopulationSnapshot,
}
impl BrowserPopulation {
    fn new() -> Self {
        let (scenario, config) = fixture(0, 0, 0).expect("valid fixture");
        let simulation =
            CourseWorld::new_population(scenario, config).expect("valid population fixture");
        let state = simulation.snapshot();
        let population = simulation
            .population_snapshot()
            .expect("population mode is enabled");
        Self {
            simulation,
            state,
            population,
        }
    }
    fn observe(&mut self) {
        self.state = self.simulation.snapshot();
        self.population = self
            .simulation
            .population_snapshot()
            .expect("population mode remains enabled");
        assert_eq!(
            (self.state.run, self.state.tick),
            (self.population.run, self.population.tick)
        );
    }
}
thread_local! { static HOST: RefCell<BrowserPopulation> = RefCell::new(BrowserPopulation::new()); }

// SAFETY: Unique scalar-only C exports, with no caller-owned memory or mutable
// global references crossing the ABI. Availability is separate from u64 values.
#[unsafe(no_mangle)]
pub extern "C" fn moss_population_reset(environment: u32, cohort: u32, mutation: u32) -> u32 {
    let Some((scenario, config)) = fixture(environment, cohort, mutation) else {
        return 0;
    };
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        if host
            .simulation
            .reset_with_population(scenario, config)
            .is_err()
        {
            return 0;
        }
        host.observe();
        1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_population_step() {
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        host.simulation.step();
        host.observe();
    });
}

fn value(host: &BrowserPopulation, table: u32, row: usize, column: usize) -> Option<u64> {
    let s = &host.state;
    let p = &host.population;
    let c = &p.config;
    match table {
        0 if row == 0 => [
            s.run,
            s.tick,
            match s.lit {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            },
            s.grazers.len() as u64,
            s.patches.len() as u64,
            s.history.events.len() as u64,
            p.history.events.len() as u64,
            p.living as u64,
            p.juveniles as u64,
            p.adults as u64,
            p.totals.initial_founders,
            p.totals.births,
            p.totals.starvations,
            p.totals.capacity_blocked_ticks,
            p.totals.capacity_blocked_pairs,
            s.ledger.added_units,
            s.ledger.maintenance_units,
            s.ledger.eaten_units,
            s.ledger.starvations,
            p.ledger.births,
            p.ledger.transferred_units,
            p.ledger.dissipated_units,
            p.ledger.capacity_blocked_pairs,
            s.history.complete_after_tick,
            s.history.collected_through_tick,
            s.history.evicted_events,
            p.history.complete_after_tick,
            p.history.collected_through_tick,
            p.history.evicted_events,
            u64::from(p.next_child_id.is_some()),
            p.next_child_id.map_or(0, |id| u64::from(id.0)),
            p.next_birth_ordinal,
            s.daylight.cycle_ticks(),
            s.daylight.lit_ticks(),
            p.initial_scenario.grazers.len() as u64,
            p.initial_scenario.patches.len() as u64,
        ]
        .get(column)
        .copied(),
        1 if row == 0 => [
            c.maturation_ticks,
            c.cooldown_ticks,
            u64::from(c.contribution_units_per_parent),
            u64::from(c.birth_cost_units_per_parent),
            u64::from(c.child_capacity_units),
            u64::from(c.child_meal_units_per_tick),
            c.max_living as u64,
            c.mutation_cycle.len() as u64,
            s.history.limit as u64,
            p.history.limit as u64,
        ]
        .get(column)
        .copied(),
        2 => s
            .grazers
            .get(row)
            .zip(p.individuals.get(row))
            .and_then(|(g, i)| {
                assert_eq!(g.id, i.id, "the two owned reports must join by stable ID");
                let o = i.origin;
                [
                    u64::from(g.id.0),
                    u64::from(g.reserve_units),
                    u64::from(g.capacity_units),
                    u64::from(g.maintenance_units_per_tick),
                    u64::from(g.meal_units_per_tick),
                    u64::from(g.feeding_site.is_some()),
                    g.feeding_site.map_or(0, |id| u64::from(id.0)),
                    u64::from(i.stage == Stage::Adult),
                    i.first_eligible_tick,
                    u64::from(i.matures_at_tick.is_some()),
                    i.matures_at_tick.unwrap_or(0),
                    i.next_birth_tick,
                    u64::from(i.upkeep.value()),
                    u64::from(o.is_some()),
                    o.map_or(0, |o| o.run),
                    o.map_or(0, |o| o.birth_tick),
                    o.map_or(0, |o| o.birth_ordinal),
                    o.map_or(0, |o| u64::from(o.parents[0].0)),
                    o.map_or(0, |o| u64::from(o.parents[1].0)),
                    o.map_or(0, |o| u64::from(o.donor.0)),
                    o.map_or(0, |o| u64::from(o.inherited.value())),
                    o.map_or(0, |o| i64::from(o.proposed_delta) as u64),
                    o.map_or(0, |o| i64::from(o.applied_delta) as u64),
                    o.map_or(0, |o| u64::from(o.child_trait.value())),
                ]
                .get(column)
                .copied()
            }),
        3 => s.patches.get(row).and_then(|patch| {
            [
                u64::from(patch.id.0),
                u64::from(patch.biomass_units),
                u64::from(patch.capacity_units),
                u64::from(patch.growth_units_per_lit_tick),
            ]
            .get(column)
            .copied()
        }),
        4 => s.history.events.get(row).and_then(|event| {
            let (kind, actor, target, units) = match event.kind {
                EventKind::Growth { patch, units } => (0, patch.0, 0, units),
                EventKind::Maintenance { grazer, units } => (1, grazer.0, 0, units),
                EventKind::Meal {
                    grazer,
                    patch,
                    units,
                } => (2, grazer.0, patch.0, units),
                EventKind::Starved { grazer } => (3, grazer.0, 0, 0),
            };
            [
                event.run,
                event.tick,
                kind,
                u64::from(actor),
                u64::from(target),
                u64::from(units),
            ]
            .get(column)
            .copied()
        }),
        5 => p.history.events.get(row).and_then(|event| {
            let mut fields = [0_u64; 17];
            fields[0] = event.run;
            fields[1] = event.tick;
            match event.kind {
                PopulationEventKind::Born(b) => {
                    fields[3] = u64::from(b.child.0);
                    fields[4] = u64::from(b.origin.child_trait.value());
                    fields[5] = b.origin.birth_ordinal;
                    fields[6] = u64::from(b.origin.parents[0].0);
                    fields[7] = u64::from(b.origin.parents[1].0);
                    fields[8] = u64::from(b.origin.donor.0);
                    fields[9] = u64::from(b.origin.inherited.value());
                    fields[10] = i64::from(b.origin.proposed_delta) as u64;
                    fields[11] = i64::from(b.origin.applied_delta) as u64;
                    fields[12] = u64::from(b.contribution_units_per_parent);
                    fields[13] = u64::from(b.cost_units_per_parent);
                    fields[14] = u64::from(b.initial_reserve_units);
                    fields[15] = b.first_eligible_tick;
                    fields[16] = b.matures_at_tick;
                }
                PopulationEventKind::Matured { individual, upkeep } => {
                    fields[2] = 1;
                    fields[3] = u64::from(individual.0);
                    fields[4] = u64::from(upkeep.value());
                }
            }
            fields.get(column).copied()
        }),
        6 if column == 0 => c
            .mutation_cycle
            .get(row)
            .map(|delta| i64::from(*delta) as u64),
        7 => p.initial_scenario.grazers.get(row).and_then(|g| {
            [
                u64::from(g.id.0),
                u64::from(g.reserve_units),
                u64::from(g.capacity_units),
                u64::from(g.maintenance_units_per_tick),
                u64::from(g.meal_units_per_tick),
                u64::from(g.feeding_site.is_some()),
                g.feeding_site.map_or(0, |id| u64::from(id.0)),
            ]
            .get(column)
            .copied()
        }),
        8 => p.initial_scenario.patches.get(row).and_then(|p| {
            [
                u64::from(p.id.0),
                u64::from(p.biomass_units),
                u64::from(p.capacity_units),
                u64::from(p.growth_units_per_lit_tick),
            ]
            .get(column)
            .copied()
        }),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_population_read(table: u32, row: u32, column: u32) -> u64 {
    HOST.with(|host| {
        value(&host.borrow(), table, row as usize, column as usize).unwrap_or(u64::MAX)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_population_valid(table: u32, row: u32, column: u32) -> u32 {
    HOST.with(|host| {
        u32::from(value(&host.borrow(), table, row as usize, column as usize).is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newborn_projection_exposes_presence_before_first_action() {
        assert_eq!(moss_population_read(0, 0, 7), 2);
        moss_population_step();
        assert_eq!(moss_population_read(0, 0, 7), 3);
        assert_eq!(moss_population_read(2, 2, 0), 101);
        assert_eq!(moss_population_read(2, 2, 1), 4);
        assert_eq!(moss_population_read(2, 2, 7), 0);
        assert_eq!(moss_population_read(2, 2, 8), 2);
        assert_eq!(moss_population_read(2, 2, 10), 3);
        assert_eq!(moss_population_read(0, 0, 20), 4);
        assert_eq!(moss_population_read(0, 0, 21), 2);
        moss_population_step();
        assert_eq!(moss_population_read(2, 2, 1), 5);
        moss_population_step();
        assert_eq!(moss_population_read(2, 2, 7), 1);
    }

    #[test]
    fn read_and_invalid_reset_preserve_run_and_state() {
        moss_population_step();
        let run = moss_population_read(0, 0, 0);
        let tick = moss_population_read(0, 0, 1);
        assert_eq!(moss_population_reset(2, 0, 0), 0);
        assert_eq!(moss_population_read(0, 0, 0), run);
        assert_eq!(moss_population_read(0, 0, 1), tick);
        assert_eq!(moss_population_reset(1, 3, 0), 1);
        assert_eq!(moss_population_read(0, 0, 0), run + 1);
        assert_eq!(moss_population_read(0, 0, 1), 0);
        assert_eq!(moss_population_read(1, 0, 7), 1);
    }

    #[test]
    fn signed_mutation_and_missing_rows_are_explicit() {
        assert_eq!(moss_population_reset(0, 0, 1), 1);
        assert_eq!(moss_population_read(6, 0, 0) as i64, -1);
        assert_eq!(moss_population_valid(6, 0, 0), 1);
        assert_eq!(moss_population_valid(2, 99, 0), 0);
        assert_eq!(moss_population_valid(0, 1, 0), 0);
        assert_eq!(moss_population_read(2, 99, 0), u64::MAX);
    }
}
