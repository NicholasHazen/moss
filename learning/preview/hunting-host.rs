//! Scalar adapter for the same checked CourseWorld; no simulation in JavaScript.
//! Tables0–18 preserve the grazer-only mobile/rest interface.
//! Supplemental hunting ABI (all scalar values are u64; valid distinguishes absence):
//! 19/13 summary: initial hunters, living hunters, visible-prey rows, event count,
//!   history limit, complete-after, collected-through, evicted, initial grazer
//!   reserves, initial patch biomass, initial hunter reserves, initial ALL stores,
//!   current hunter reserves.
//! 20/13 latest counters; 21/13 lifetime counters: upkeep, travel cells, travel
//!   units, attack units, transferred units, captures, hunter starvations, target
//!   changes, effort, recovered, rest actions, entered rest, woke.
//! 22/35 hunters: id, reserve, capacity, upkeep rate, attack cost, attack effort,
//!   first eligible tick, x, y, sight, speed, travel cost; target present, prey,
//!   observed prey x/y/tick; observation present/tick, origin present/x/y,
//!   visible-prey count; fatigue present/value, activity(0 forage/1 rest), remaining;
//!   last-rest present/tick, last-transition present/tick, last-travel present/tick,
//!   last-capture present/tick.
//! 23/5 prey observations: hunter, prey, observed x, y, reserve.
//! 24/11 initial hunters: id, reserve, capacity, upkeep, attack cost, attack effort,
//!   x, y, sight, speed, travel cost.
//! 25/12 events: run, tick, kind, hunter, followed by payload. Kinds and payloads:
//!   0 upkeep(units); 1 target(from present/id, to present/id);
//!   2 travel(from x/y, to x/y, cells, cost, effort);
//!   3 capture(prey, x, y, transfer, cost, effort); 4 starvation(no payload);
//!   5 entered rest(fatigue, commitment); 6 woke(fatigue);
//!   7 rested(recovered, fatigue after, remaining). Unused columns are zero.
use course::{
    Activity, CourseWorld, EventKind, HuntConfig, HuntCounters, HuntEventKind, HuntSnapshot,
    MobileEventKind, MobileScenario, MobileSnapshot, PopulationEventKind, RestEventKind,
    RestPolicy, RestSnapshot, Stage,
};
use std::cell::RefCell;

fn fixture(preset: u32, minimum: u32, hunters: u32) -> Option<MobileScenario> {
    if !matches!(minimum, 2 | 3) || hunters > 1 {
        return None;
    }
    let policy = RestPolicy {
        minimum_rest_ticks: minimum,
        ..Default::default()
    };
    let mut spec = match preset {
        0 => MobileScenario::hunting_contention(),
        1 => MobileScenario::meadow().with_hunters(HuntConfig::meadow()),
        _ => return None,
    }
    .with_rest(policy);
    if hunters == 0 {
        spec = spec.with_hunters(HuntConfig::new(Vec::new()));
    }
    Some(spec)
}
struct BrowserHunting {
    simulation: CourseWorld,
    state: MobileSnapshot,
    rest: RestSnapshot,
    hunt: HuntSnapshot,
}
impl BrowserHunting {
    fn new() -> Self {
        let simulation = CourseWorld::new_mobile(fixture(0, 2, 1).expect("authored fixture"))
            .expect("valid authored fixture");
        let state = simulation.mobile_snapshot().expect("mobile mode");
        let rest = simulation.rest_snapshot().expect("rest mode");
        let hunt = simulation.hunt_snapshot().expect("hunting mode");
        Self {
            simulation,
            state,
            rest,
            hunt,
        }
    }
    fn observe(&mut self) {
        self.hunt = self
            .simulation
            .hunt_snapshot()
            .expect("hunting remains enabled");
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
        assert_eq!(
            (self.state.run, self.state.tick),
            (self.hunt.run, self.hunt.tick)
        );
        assert_eq!(self.hunt.living_hunters, self.hunt.hunters.len());
        assert!(self.hunt.hunters.iter().all(|hunter| {
            self.state
                .grazers
                .iter()
                .all(|grazer| hunter.id != grazer.id)
        }));
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
thread_local! {static HOST: RefCell<BrowserHunting> = RefCell::new(BrowserHunting::new());}

// SAFETY: Unique scalar-only C exports; no caller-owned memory crosses the ABI.
#[unsafe(no_mangle)]
pub extern "C" fn moss_hunting_reset(preset: u32, minimum: u32, hunters: u32) -> u32 {
    let Some(inputs) = fixture(preset, minimum, hunters) else {
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
pub extern "C" fn moss_hunting_step() {
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        host.simulation.step();
        host.observe();
    });
}

fn counters(c: &HuntCounters) -> [u64; 13] {
    [
        c.maintenance_units,
        c.travel_cells,
        c.travel_units,
        c.attack_units,
        c.transferred_units,
        c.captures,
        c.hunter_starvations,
        c.target_changes,
        c.effort_points,
        c.recovered_points,
        c.rest_actions,
        c.entered_rest,
        c.woke,
    ]
}

fn value(host: &BrowserHunting, table: u32, row: usize, column: usize) -> Option<u64> {
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
        19 if row == 0 => {
            let h = &host.hunt;
            let grazer_reserves: u64 = host
                .state
                .initial
                .scenario()
                .grazers
                .iter()
                .map(|g| u64::from(g.reserve_units))
                .sum();
            let patch_biomass: u64 = host
                .state
                .initial
                .scenario()
                .patches
                .iter()
                .map(|p| u64::from(p.biomass_units))
                .sum();
            let hunter_reserves: u64 = h
                .config
                .hunters
                .iter()
                .map(|hunter| u64::from(hunter.reserve_units))
                .sum();
            [
                h.initial_hunters as u64,
                h.living_hunters as u64,
                h.hunters
                    .iter()
                    .map(|hunter| hunter.visible_prey.len() as u64)
                    .sum(),
                h.history.events.len() as u64,
                h.history.limit as u64,
                h.history.complete_after_tick,
                h.history.collected_through_tick,
                h.history.evicted_events,
                grazer_reserves,
                patch_biomass,
                hunter_reserves,
                grazer_reserves + patch_biomass + hunter_reserves,
                h.hunters
                    .iter()
                    .map(|hunter| u64::from(hunter.reserve_units))
                    .sum(),
            ]
            .get(column)
            .copied()
        }
        20 if row == 0 => counters(&host.hunt.ledger).get(column).copied(),
        21 if row == 0 => counters(&host.hunt.totals).get(column).copied(),
        22 => host.hunt.hunters.get(row).and_then(|h| {
            let (activity, remaining) = match h.activity {
                Activity::Foraging => (0, 0),
                Activity::Resting { remaining_ticks } => (1, remaining_ticks),
            };
            [
                u64::from(h.id.0),
                u64::from(h.reserve_units),
                u64::from(h.capacity_units),
                u64::from(h.maintenance_units_per_tick),
                u64::from(h.attack_units),
                u64::from(h.attack_effort_points),
                h.first_eligible_tick,
                u64::from(h.cell.x),
                u64::from(h.cell.y),
                u64::from(h.motion.sensing_radius),
                u64::from(h.motion.max_cells_per_tick),
                u64::from(h.motion.travel_units_per_cell),
                u64::from(h.target.is_some()),
                h.target.map_or(0, |t| u64::from(t.prey.0)),
                h.target.map_or(0, |t| u64::from(t.observed_cell.x)),
                h.target.map_or(0, |t| u64::from(t.observed_cell.y)),
                h.target.map_or(0, |t| t.observed_tick),
                u64::from(h.observed_tick.is_some()),
                h.observed_tick.unwrap_or(0),
                u64::from(h.observed_cell.is_some()),
                h.observed_cell.map_or(0, |cell| u64::from(cell.x)),
                h.observed_cell.map_or(0, |cell| u64::from(cell.y)),
                h.visible_prey.len() as u64,
                u64::from(h.fatigue_points.is_some()),
                h.fatigue_points.map_or(0, u64::from),
                activity,
                u64::from(remaining),
                u64::from(h.last_rest_tick.is_some()),
                h.last_rest_tick.unwrap_or(0),
                u64::from(h.last_transition_tick.is_some()),
                h.last_transition_tick.unwrap_or(0),
                u64::from(h.last_travel_tick.is_some()),
                h.last_travel_tick.unwrap_or(0),
                u64::from(h.last_capture_tick.is_some()),
                h.last_capture_tick.unwrap_or(0),
            ]
            .get(column)
            .copied()
        }),
        23 => host
            .hunt
            .hunters
            .iter()
            .flat_map(|h| h.visible_prey.iter().map(move |p| (h.id, p)))
            .nth(row)
            .and_then(|(id, p)| {
                [
                    u64::from(id.0),
                    u64::from(p.prey.0),
                    u64::from(p.cell.x),
                    u64::from(p.cell.y),
                    u64::from(p.reserve_units),
                ]
                .get(column)
                .copied()
            }),
        24 => host.hunt.config.hunters.get(row).and_then(|h| {
            [
                u64::from(h.id.0),
                u64::from(h.reserve_units),
                u64::from(h.capacity_units),
                u64::from(h.maintenance_units_per_tick),
                u64::from(h.attack_units),
                u64::from(h.attack_effort_points),
                u64::from(h.cell.x),
                u64::from(h.cell.y),
                u64::from(h.motion.sensing_radius),
                u64::from(h.motion.max_cells_per_tick),
                u64::from(h.motion.travel_units_per_cell),
            ]
            .get(column)
            .copied()
        }),
        25 => host.hunt.history.events.get(row).and_then(|event| {
            let mut fields = [0_u64; 12];
            fields[0] = event.run;
            fields[1] = event.tick;
            match event.kind {
                HuntEventKind::Maintenance { hunter, units } => {
                    fields[3] = u64::from(hunter.0);
                    fields[4] = u64::from(units);
                }
                HuntEventKind::TargetChanged { hunter, from, to } => {
                    fields[2] = 1;
                    fields[3] = u64::from(hunter.0);
                    fields[4] = u64::from(from.is_some());
                    fields[5] = from.map_or(0, |id| u64::from(id.0));
                    fields[6] = u64::from(to.is_some());
                    fields[7] = to.map_or(0, |id| u64::from(id.0));
                }
                HuntEventKind::Travelled {
                    hunter,
                    from,
                    to,
                    cells,
                    spent_units,
                    effort_points,
                } => {
                    fields[2] = 2;
                    fields[3] = u64::from(hunter.0);
                    fields[4] = u64::from(from.x);
                    fields[5] = u64::from(from.y);
                    fields[6] = u64::from(to.x);
                    fields[7] = u64::from(to.y);
                    fields[8] = u64::from(cells);
                    fields[9] = u64::from(spent_units);
                    fields[10] = u64::from(effort_points);
                }
                HuntEventKind::Captured {
                    hunter,
                    prey,
                    cell,
                    transferred_units,
                    attack_units,
                    effort_points,
                } => {
                    fields[2] = 3;
                    fields[3] = u64::from(hunter.0);
                    fields[4] = u64::from(prey.0);
                    fields[5] = u64::from(cell.x);
                    fields[6] = u64::from(cell.y);
                    fields[7] = u64::from(transferred_units);
                    fields[8] = u64::from(attack_units);
                    fields[9] = u64::from(effort_points);
                }
                HuntEventKind::HunterStarved { hunter } => {
                    fields[2] = 4;
                    fields[3] = u64::from(hunter.0);
                }
                HuntEventKind::EnteredRest {
                    hunter,
                    fatigue_points,
                    committed_ticks,
                } => {
                    fields[2] = 5;
                    fields[3] = u64::from(hunter.0);
                    fields[4] = u64::from(fatigue_points);
                    fields[5] = u64::from(committed_ticks);
                }
                HuntEventKind::Woke {
                    hunter,
                    fatigue_points,
                } => {
                    fields[2] = 6;
                    fields[3] = u64::from(hunter.0);
                    fields[4] = u64::from(fatigue_points);
                }
                HuntEventKind::Rested {
                    hunter,
                    recovered_points,
                    fatigue_after,
                    remaining_ticks,
                } => {
                    fields[2] = 7;
                    fields[3] = u64::from(hunter.0);
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
pub extern "C" fn moss_hunting_read(table: u32, row: u32, column: u32) -> u64 {
    HOST.with(|host| {
        value(&host.borrow(), table, row as usize, column as usize).unwrap_or(u64::MAX)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_hunting_valid(table: u32, row: u32, column: u32) -> u32 {
    HOST.with(|host| {
        u32::from(value(&host.borrow(), table, row as usize, column as usize).is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contention_projects_one_capture_and_preserves_the_losers_old_local_reading() {
        assert_eq!(moss_hunting_reset(0, 2, 1), 1);
        assert_eq!(moss_hunting_read(19, 0, 11), 28);
        moss_hunting_step();
        assert_eq!(moss_hunting_read(0, 0, 3), 1);
        assert_eq!(moss_hunting_read(2, 0, 0), 2);
        assert_eq!(moss_hunting_read(2, 0, 1), 6);
        assert_eq!(moss_hunting_read(3, 0, 1), 8);
        assert_eq!(moss_hunting_read(22, 0, 1), 6);
        assert_eq!(moss_hunting_read(22, 1, 1), 3);
        assert_eq!(moss_hunting_read(22, 0, 24), 2);
        assert_eq!(moss_hunting_read(22, 1, 24), 0);
        assert_eq!(moss_hunting_read(22, 0, 33), 1);
        assert_eq!(moss_hunting_read(22, 1, 33), 0);
        assert_eq!(moss_hunting_read(22, 1, 13), 1);
        assert_eq!(moss_hunting_read(23, 2, 0), 101);
        assert_eq!(moss_hunting_read(23, 2, 1), 1);
        assert_eq!(moss_hunting_read(23, 2, 4), 4);
        assert_eq!(moss_hunting_read(20, 0, 3), 1);
        assert_eq!(moss_hunting_read(20, 0, 4), 4);
        assert_eq!(moss_hunting_read(20, 0, 5), 1);
        assert_eq!(moss_hunting_read(0, 0, 11), 0);
        assert_eq!(moss_hunting_read(0, 0, 12), 0);
    }

    #[test]
    fn reads_and_rejected_selectors_preserve_all_three_owned_reports() {
        assert_eq!(moss_hunting_reset(0, 2, 1), 1);
        moss_hunting_step();
        let before = HOST.with(|host| {
            let host = host.borrow();
            (host.state.clone(), host.rest.clone(), host.hunt.clone())
        });
        for (preset, minimum, hunters) in [(2, 2, 1), (0, 1, 1), (0, 2, 2)] {
            assert_eq!(moss_hunting_reset(preset, minimum, hunters), 0);
        }
        for _ in 0..5 {
            assert_eq!(moss_hunting_read(23, 2, 1), 1);
            assert_eq!(moss_hunting_valid(22, 99, 0), 0);
            assert_eq!(moss_hunting_read(22, 99, 0), u64::MAX);
        }
        HOST.with(|host| {
            let host = host.borrow();
            assert_eq!(
                (&host.state, &host.rest, &host.hunt),
                (&before.0, &before.1, &before.2)
            );
        });
        assert_eq!(moss_hunting_reset(1, 3, 0), 1);
        assert_eq!(moss_hunting_read(0, 0, 0), before.0.run + 1);
        assert_eq!(moss_hunting_read(0, 0, 1), 0);
        assert_eq!(moss_hunting_read(19, 0, 0), 0);
        assert_eq!(moss_hunting_read(19, 0, 11), 96);
        assert_eq!(moss_hunting_read(21, 0, 5), 0);
        assert_eq!(moss_hunting_valid(22, 0, 0), 0);
    }

    #[test]
    fn scalar_energy_and_cause_specific_census_include_both_animal_roles() {
        for minimum in [2, 3] {
            assert_eq!(moss_hunting_reset(1, minimum, 1), 1);
            let stores = || {
                HOST.with(|host| {
                    let host = host.borrow();
                    host.state
                        .base
                        .grazers
                        .iter()
                        .map(|g| u64::from(g.reserve_units))
                        .sum::<u64>()
                        + host
                            .state
                            .base
                            .patches
                            .iter()
                            .map(|p| u64::from(p.biomass_units))
                            .sum::<u64>()
                        + moss_hunting_read(19, 0, 12)
                })
            };
            let mut previous = stores();
            assert_eq!(previous, 128);
            for _ in 0..120 {
                moss_hunting_step();
                let now = stores();
                let paid = moss_hunting_read(0, 0, 16)
                    + moss_hunting_read(9, 0, 8)
                    + moss_hunting_read(0, 0, 21)
                    + moss_hunting_read(20, 0, 0)
                    + moss_hunting_read(20, 0, 2)
                    + moss_hunting_read(20, 0, 3);
                assert_eq!(now + paid, previous + moss_hunting_read(0, 0, 15));
                assert_eq!(
                    moss_hunting_read(0, 0, 7)
                        + moss_hunting_read(0, 0, 12)
                        + moss_hunting_read(21, 0, 5),
                    moss_hunting_read(0, 0, 10) + moss_hunting_read(0, 0, 11)
                );
                assert_eq!(
                    moss_hunting_read(19, 0, 1) + moss_hunting_read(21, 0, 6),
                    moss_hunting_read(19, 0, 0)
                );
                previous = now;
            }
            assert_eq!(moss_hunting_read(19, 0, 1), 0);
            assert_eq!(
                moss_hunting_read(21, 0, 5),
                if minimum == 2 { 3 } else { 4 }
            );
        }
    }

    #[test]
    fn an_explicit_empty_hunter_configuration_has_no_added_stores_or_hidden_effects() {
        assert_eq!(moss_hunting_reset(1, 2, 0), 1);
        for _ in 0..120 {
            moss_hunting_step();
        }
        assert_eq!(moss_hunting_read(19, 0, 0), 0);
        assert_eq!(moss_hunting_read(19, 0, 10), 0);
        assert_eq!(moss_hunting_read(19, 0, 11), 96);
        for column in 0..13 {
            assert_eq!(moss_hunting_read(21, 0, column), 0);
        }
        assert_eq!(moss_hunting_read(19, 0, 6), 120);
        assert_eq!(moss_hunting_read(19, 0, 7), 0);
        assert_eq!(moss_hunting_read(0, 0, 11), 7);
        assert_eq!(moss_hunting_read(0, 0, 12), 11);
    }
}
