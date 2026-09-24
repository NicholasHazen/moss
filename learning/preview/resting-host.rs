//! Scalar adapter for the same checked CourseWorld; no simulation in JavaScript.
//! Tables0–14 preserve the mobile interface; tables15–18 add optional rest evidence.
use course::{
    Activity, CourseWorld, EventKind, MobileEventKind, MobileScenario, MobileSnapshot,
    PopulationEventKind, RestEventKind, RestPolicy, RestSnapshot, Stage,
};
use std::cell::RefCell;

fn fixture(preset: u32, minimum: u32, recovery: u32) -> Option<MobileScenario> {
    if !matches!(minimum, 2 | 3) || !matches!(recovery, 2 | 6) {
        return None;
    }
    let policy = RestPolicy {
        minimum_rest_ticks: minimum,
        recovery_points_per_tick: recovery,
        ..Default::default()
    };
    match preset {
        0 => Some(MobileScenario::resting_journey().with_rest(policy)),
        1 => Some(MobileScenario::meadow().with_rest(policy)),
        _ => None,
    }
}
struct BrowserResting {
    simulation: CourseWorld,
    state: MobileSnapshot,
    rest: RestSnapshot,
}
impl BrowserResting {
    fn new() -> Self {
        let simulation = CourseWorld::new_mobile(fixture(0, 2, 2).expect("authored fixture"))
            .expect("valid authored fixture");
        let state = simulation.mobile_snapshot().expect("mobile mode");
        let rest = simulation.rest_snapshot().expect("rest mode");
        Self {
            simulation,
            state,
            rest,
        }
    }
    fn observe(&mut self) {
        self.rest = self
            .simulation
            .rest_snapshot()
            .expect("rest remains enabled");
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
        assert_eq!(
            (self.state.run, self.state.tick),
            (self.rest.run, self.rest.tick)
        );
        assert_eq!(self.state.grazers.len(), self.rest.grazers.len());
        assert!(
            self.state
                .grazers
                .iter()
                .zip(&self.rest.grazers)
                .all(|(mobile, rest)| mobile.id == rest.id)
        );
    }
}
thread_local! {static HOST: RefCell<BrowserResting> = RefCell::new(BrowserResting::new());}

// SAFETY: Unique scalar-only C exports; no caller-owned memory crosses the ABI.
#[unsafe(no_mangle)]
pub extern "C" fn moss_resting_reset(preset: u32, minimum: u32, recovery: u32) -> u32 {
    let Some(inputs) = fixture(preset, minimum, recovery) else {
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
pub extern "C" fn moss_resting_step() {
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        host.simulation.step();
        host.observe();
    });
}

fn value(host: &BrowserResting, table: u32, row: usize, column: usize) -> Option<u64> {
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
        15 if row == 0 => {
            let p = &host.rest.policy;
            [
                u64::from(p.maximum_fatigue_points),
                u64::from(p.effort_points_per_cell),
                u64::from(p.enter_at_points),
                u64::from(p.exit_at_points),
                u64::from(p.minimum_rest_ticks),
                u64::from(p.recovery_points_per_tick),
                p.history_limit as u64,
            ]
            .get(column)
            .copied()
        }
        16 if row == 0 => {
            let r = &host.rest;
            [
                r.ledger.effort_points,
                r.ledger.recovered_points,
                r.ledger.rest_actions,
                r.ledger.entered_rest,
                r.ledger.woke,
                r.totals.effort_points,
                r.totals.recovered_points,
                r.totals.rest_actions,
                r.totals.entered_rest,
                r.totals.woke,
                r.history.events.len() as u64,
                r.history.complete_after_tick,
                r.history.collected_through_tick,
                r.history.evicted_events,
                r.history.limit as u64,
            ]
            .get(column)
            .copied()
        }
        17 => host.rest.grazers.get(row).and_then(|g| {
            let (activity, remaining) = match g.activity {
                Activity::Foraging => (0, 0),
                Activity::Resting { remaining_ticks } => (1, remaining_ticks),
            };
            [
                u64::from(g.id.0),
                u64::from(g.fatigue_points),
                activity,
                u64::from(remaining),
                u64::from(g.last_rest_tick.is_some()),
                g.last_rest_tick.unwrap_or(0),
                u64::from(g.last_transition_tick.is_some()),
                g.last_transition_tick.unwrap_or(0),
            ]
            .get(column)
            .copied()
        }),
        18 => host.rest.history.events.get(row).and_then(|event| {
            let mut fields = [0_u64; 8];
            fields[0] = event.run;
            fields[1] = event.tick;
            match event.kind {
                RestEventKind::EnteredRest {
                    grazer,
                    fatigue_points,
                    committed_ticks,
                } => {
                    fields[3] = u64::from(grazer.0);
                    fields[4] = u64::from(fatigue_points);
                    fields[5] = u64::from(committed_ticks);
                }
                RestEventKind::Woke {
                    grazer,
                    fatigue_points,
                } => {
                    fields[2] = 1;
                    fields[3] = u64::from(grazer.0);
                    fields[4] = u64::from(fatigue_points);
                }
                RestEventKind::TravelEffort {
                    grazer,
                    cells,
                    points,
                } => {
                    fields[2] = 2;
                    fields[3] = u64::from(grazer.0);
                    fields[4] = u64::from(cells);
                    fields[5] = u64::from(points);
                }
                RestEventKind::Rested {
                    grazer,
                    recovered_points,
                    fatigue_after,
                    remaining_ticks,
                } => {
                    fields[2] = 3;
                    fields[3] = u64::from(grazer.0);
                    fields[4] = u64::from(recovered_points);
                    fields[5] = u64::from(fatigue_after);
                    fields[6] = u64::from(remaining_ticks);
                }
            }
            fields.get(column).copied()
        }),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_resting_read(table: u32, row: u32, column: u32) -> u64 {
    HOST.with(|host| {
        value(&host.borrow(), table, row as usize, column as usize).unwrap_or(u64::MAX)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_resting_valid(table: u32, row: u32, column: u32) -> u32 {
    HOST.with(|host| {
        u32::from(value(&host.borrow(), table, row as usize, column as usize).is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_journey_retains_observation_while_resting_and_refreshes_on_wake() {
        assert_eq!(moss_resting_reset(0, 2, 2), 1);
        for (tick, x, reserve, fatigue, activity, remaining, observed) in [
            (1, 1, 18, 2, 0, 0, 1),
            (2, 2, 16, 4, 0, 0, 2),
            (3, 3, 14, 6, 0, 0, 3),
            (4, 3, 13, 4, 1, 1, 3),
            (5, 3, 12, 2, 1, 0, 3),
            (6, 4, 10, 4, 0, 0, 6),
            (7, 5, 10, 6, 0, 0, 7),
            (8, 5, 9, 4, 1, 1, 7),
            (9, 5, 8, 2, 1, 0, 7),
            (10, 5, 9, 2, 0, 0, 10),
        ] {
            moss_resting_step();
            assert_eq!(moss_resting_read(0, 0, 1), tick);
            assert_eq!(moss_resting_read(10, 0, 1), x);
            assert_eq!(moss_resting_read(2, 0, 1), reserve);
            assert_eq!(moss_resting_read(17, 0, 1), fatigue);
            assert_eq!(moss_resting_read(17, 0, 2), activity);
            assert_eq!(moss_resting_read(17, 0, 3), remaining);
            assert_eq!(moss_resting_read(10, 0, 12), observed);
        }
        assert_eq!(
            (5..10)
                .map(|c| moss_resting_read(16, 0, c))
                .collect::<Vec<_>>(),
            vec![10, 8, 4, 2, 2]
        );
    }

    #[test]
    fn fast_recovery_does_not_cancel_the_second_committed_action() {
        assert_eq!(moss_resting_reset(0, 2, 6), 1);
        for _ in 0..4 {
            moss_resting_step();
        }
        assert_eq!(moss_resting_read(17, 0, 1), 0);
        assert_eq!(moss_resting_read(17, 0, 2), 1);
        assert_eq!(moss_resting_read(17, 0, 3), 1);
        assert_eq!(moss_resting_read(16, 0, 1), 6);
        moss_resting_step();
        assert_eq!(moss_resting_read(17, 0, 2), 1);
        assert_eq!(moss_resting_read(17, 0, 3), 0);
        assert_eq!(moss_resting_read(16, 0, 1), 0);
        assert_eq!(moss_resting_read(16, 0, 2), 1);
        assert_eq!(moss_resting_read(10, 0, 1), 3);
        moss_resting_step();
        assert_eq!(moss_resting_read(17, 0, 2), 0);
        assert_eq!(moss_resting_read(10, 0, 1), 4);
    }

    #[test]
    fn invalid_reset_and_repeated_reads_preserve_both_owned_reports() {
        assert_eq!(moss_resting_reset(0, 2, 2), 1);
        for _ in 0..4 {
            moss_resting_step();
        }
        let before = HOST.with(|host| (host.borrow().state.clone(), host.borrow().rest.clone()));
        for (preset, minimum, recovery) in [(2, 2, 2), (0, 1, 2), (0, 2, 3)] {
            assert_eq!(moss_resting_reset(preset, minimum, recovery), 0);
        }
        for _ in 0..5 {
            assert_eq!(moss_resting_read(17, 0, 3), 1);
            assert_eq!(moss_resting_valid(17, 99, 0), 0);
            assert_eq!(moss_resting_read(17, 99, 0), u64::MAX);
            assert_eq!(
                HOST.with(|host| (host.borrow().state.clone(), host.borrow().rest.clone())),
                before
            );
        }
        assert_eq!(moss_resting_reset(1, 3, 6), 1);
        assert_eq!(moss_resting_read(0, 0, 0), before.0.run + 1);
        assert_eq!(moss_resting_read(0, 0, 1), 0);
        assert_eq!(moss_resting_read(15, 0, 4), 3);
        assert_eq!(moss_resting_read(15, 0, 5), 6);
        assert_eq!(moss_resting_read(16, 0, 7), 0);
    }

    #[test]
    fn old_tables_and_rest_readings_join_every_living_id_after_turnover() {
        assert_eq!(moss_resting_reset(1, 2, 2), 1);
        for _ in 0..120 {
            moss_resting_step();
            let count = moss_resting_read(0, 0, 3) as u32;
            for row in 0..count {
                assert_eq!(moss_resting_read(2, row, 0), moss_resting_read(10, row, 0));
                assert_eq!(moss_resting_read(2, row, 0), moss_resting_read(17, row, 0));
            }
        }
        assert!(moss_resting_read(16, 0, 7) > 0);
    }
}
