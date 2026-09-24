//! Scalar adapter for the same checked CourseWorld; no simulation in JavaScript.
//! Tables0–8 retain the population projection; tables9–14 add owned space and history.
use course::{
    CourseWorld, EventKind, MobileEventKind, MobileScenario, MobileSnapshot, PopulationEventKind,
    Stage,
};
use std::cell::RefCell;

fn fixture(preset: u32) -> Option<MobileScenario> {
    match preset {
        0 => Some(MobileScenario::short_journey()),
        1 => Some(MobileScenario::meadow()),
        _ => None,
    }
}
struct BrowserMobile {
    simulation: CourseWorld,
    state: MobileSnapshot,
}
impl BrowserMobile {
    fn new() -> Self {
        let simulation = CourseWorld::new_mobile(fixture(0).expect("authored fixture"))
            .expect("valid authored fixture");
        let state = simulation.mobile_snapshot().expect("mobile mode");
        Self { simulation, state }
    }
    fn observe(&mut self) {
        self.state = self
            .simulation
            .mobile_snapshot()
            .expect("mobile mode remains enabled");
        assert_eq!(
            (self.state.run, self.state.tick),
            (self.state.base.run, self.state.base.tick)
        );
        assert_eq!(
            (self.state.run, self.state.tick),
            (self.state.population.run, self.state.population.tick)
        );
    }
}
thread_local! {static HOST: RefCell<BrowserMobile> = RefCell::new(BrowserMobile::new());}

// SAFETY: Unique scalar-only C exports; no caller-owned memory crosses the ABI.
#[unsafe(no_mangle)]
pub extern "C" fn moss_mobile_reset(preset: u32) -> u32 {
    let Some(inputs) = fixture(preset) else {
        return 0;
    };
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        if host.simulation.reset_with_mobile(inputs).is_err() {
            return 0;
        }
        host.observe();
        1
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn moss_mobile_step() {
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        host.simulation.step();
        host.observe();
    });
}

fn value(host: &BrowserMobile, table: u32, row: usize, column: usize) -> Option<u64> {
    let s = &host.state.base;
    let p = &host.state.population;
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
        9 if row == 0 => {
            let m = &host.state;
            let space = m.initial.space();
            [
                u64::from(space.bounds.width_cells),
                u64::from(space.bounds.height_cells),
                m.history.events.len() as u64,
                m.history.limit as u64,
                m.history.complete_after_tick,
                m.history.collected_through_tick,
                m.history.evicted_events,
                m.ledger.travel_cells,
                m.ledger.travel_units,
                m.ledger.target_changes,
                m.totals.travel_cells,
                m.totals.travel_units,
                m.totals.target_changes,
                u64::from(space.newborn_motion.sensing_radius),
                u64::from(space.newborn_motion.max_cells_per_tick),
                u64::from(space.newborn_motion.travel_units_per_cell),
                m.grazers
                    .iter()
                    .map(|g| g.visible_patches.len() as u64)
                    .sum(),
            ]
            .get(column)
            .copied()
        }
        10 => host.state.grazers.get(row).and_then(|g| {
            [
                u64::from(g.id.0),
                u64::from(g.cell.x),
                u64::from(g.cell.y),
                u64::from(g.motion.sensing_radius),
                u64::from(g.motion.max_cells_per_tick),
                u64::from(g.motion.travel_units_per_cell),
                u64::from(g.target.is_some()),
                g.target.map_or(0, |t| u64::from(t.patch.0)),
                g.target.map_or(0, |t| u64::from(t.observed_cell.x)),
                g.target.map_or(0, |t| u64::from(t.observed_cell.y)),
                g.target.map_or(0, |t| t.observed_tick),
                u64::from(g.observed_tick.is_some()),
                g.observed_tick.unwrap_or(0),
                g.visible_patches.len() as u64,
                u64::from(g.last_travel_tick.is_some()),
                g.last_travel_tick.unwrap_or(0),
                u64::from(g.observed_cell.is_some()),
                g.observed_cell.map_or(0, |cell| u64::from(cell.x)),
                g.observed_cell.map_or(0, |cell| u64::from(cell.y)),
            ]
            .get(column)
            .copied()
        }),
        11 => host.state.patches.get(row).and_then(|p| {
            [u64::from(p.id.0), u64::from(p.cell.x), u64::from(p.cell.y)]
                .get(column)
                .copied()
        }),
        12 => host
            .state
            .grazers
            .iter()
            .flat_map(|g| g.visible_patches.iter().map(move |p| (g.id, p)))
            .nth(row)
            .and_then(|(id, p)| {
                [
                    u64::from(id.0),
                    u64::from(p.patch.0),
                    u64::from(p.cell.x),
                    u64::from(p.cell.y),
                    u64::from(p.biomass_units),
                ]
                .get(column)
                .copied()
            }),
        13 => host.state.history.events.get(row).and_then(|event| {
            let mut fields = [0_u64; 12];
            fields[0] = event.run;
            fields[1] = event.tick;
            match event.kind {
                MobileEventKind::TargetChanged { grazer, from, to } => {
                    fields[3] = u64::from(grazer.0);
                    fields[4] = u64::from(from.is_some());
                    fields[5] = from.map_or(0, |id| u64::from(id.0));
                    fields[6] = u64::from(to.is_some());
                    fields[7] = to.map_or(0, |id| u64::from(id.0));
                }
                MobileEventKind::Travelled {
                    grazer,
                    from,
                    to,
                    cells,
                    spent_units,
                } => {
                    fields[2] = 1;
                    fields[3] = u64::from(grazer.0);
                    fields[4] = u64::from(from.x);
                    fields[5] = u64::from(from.y);
                    fields[6] = u64::from(to.x);
                    fields[7] = u64::from(to.y);
                    fields[8] = u64::from(cells);
                    fields[9] = u64::from(spent_units);
                }
                MobileEventKind::BornAt { child, cell } => {
                    fields[2] = 2;
                    fields[3] = u64::from(child.0);
                    fields[4] = u64::from(cell.x);
                    fields[5] = u64::from(cell.y);
                }
            }
            fields.get(column).copied()
        }),
        14 => host.state.initial.space().grazers.get(row).and_then(|g| {
            [
                u64::from(g.id.0),
                u64::from(g.cell.x),
                u64::from(g.cell.y),
                u64::from(g.motion.sensing_radius),
                u64::from(g.motion.max_cells_per_tick),
                u64::from(g.motion.travel_units_per_cell),
            ]
            .get(column)
            .copied()
        }),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_mobile_read(table: u32, row: u32, column: u32) -> u64 {
    HOST.with(|host| {
        value(&host.borrow(), table, row as usize, column as usize).unwrap_or(u64::MAX)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_mobile_valid(table: u32, row: u32, column: u32) -> u32 {
    HOST.with(|host| {
        u32::from(value(&host.borrow(), table, row as usize, column as usize).is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_origin_precedes_paid_travel_and_meal_requires_arrival() {
        moss_mobile_step();
        assert_eq!(moss_mobile_read(10, 0, 1), 1); // current x
        assert_eq!(moss_mobile_read(10, 0, 17), 0); // observed x
        assert_eq!(moss_mobile_read(10, 0, 12), 1); // observation tick
        assert_eq!(moss_mobile_read(2, 0, 1), 3); // reserve
        assert_eq!(moss_mobile_read(0, 0, 17), 0); // no meal en route
        assert_eq!(moss_mobile_read(9, 0, 8), 1); // paid travel
        moss_mobile_step();
        assert_eq!(moss_mobile_read(10, 0, 1), 2);
        assert_eq!(moss_mobile_read(0, 0, 17), 2);
        assert_eq!(moss_mobile_read(3, 0, 1), 1);
        assert_eq!(moss_mobile_read(12, 0, 4), 3); // before competing consumption
    }
    #[test]
    fn invalid_selection_and_observation_leave_the_world_unchanged() {
        moss_mobile_step();
        let before = HOST.with(|host| host.borrow().state.clone());
        assert_eq!(moss_mobile_reset(99), 0);
        assert_eq!(moss_mobile_valid(10, 99, 0), 0);
        assert_eq!(moss_mobile_valid(9, 1, 0), 0);
        assert_eq!(moss_mobile_read(10, 99, 0), u64::MAX);
        assert_eq!(HOST.with(|host| host.borrow().state.clone()), before);
        assert_eq!(moss_mobile_reset(1), 1);
        assert_eq!(moss_mobile_read(0, 0, 0), before.run + 1);
        assert_eq!(moss_mobile_read(0, 0, 1), 0);
        assert_eq!(moss_mobile_read(9, 0, 0), 11);
    }
    #[test]
    fn long_run_keeps_cumulative_travel_but_reports_evicted_history() {
        assert_eq!(moss_mobile_reset(1), 1);
        for _ in 0..120 {
            moss_mobile_step();
        }
        assert_eq!(moss_mobile_read(0, 0, 7), 4);
        assert_eq!(moss_mobile_read(0, 0, 11), 32);
        assert_eq!(moss_mobile_read(9, 0, 10), 173);
        assert!(moss_mobile_read(9, 0, 6) > 0);
        assert!(moss_mobile_read(9, 0, 4) > 0);
        assert_eq!(moss_mobile_read(9, 0, 5), 120);
    }
}
