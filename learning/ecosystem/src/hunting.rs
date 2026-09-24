//! Opt-in mortal hunters; all roles inhabit the same authoritative body world.

pub(crate) mod activity;
pub(crate) mod capture;
mod config;
mod history;
pub(crate) mod movement;
pub(crate) mod perception;
mod scenarios;

use crate::mobile::{MobileActor, Position};
use crate::model::{Body, Clock, Identity, Terminal};
use crate::simulation::Roster;
use crate::{Activity, Cell, MobileScenario, Motion, SimId};
use bevy_ecs::prelude::*;
pub use config::{HuntConfig, HunterSeed};
use history::HuntHistory;
pub use history::{HuntEvent, HuntEventKind, HuntHistorySnapshot};

pub const HUNT_VERSION: &str = "moss-course-hunting-c-v1";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreyObservation {
    pub prey: SimId,
    pub cell: Cell,
    pub reserve_units: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreyTarget {
    pub prey: SimId,
    pub observed_cell: Cell,
    pub observed_tick: u64,
}
#[derive(Component)]
pub(crate) struct Hunter {
    pub attack_units: u32,
    pub attack_effort_points: u32,
    pub target: Option<PreyTarget>,
    pub observed_tick: Option<u64>,
    pub observed_cell: Option<Cell>,
    pub visible_prey: Vec<PreyObservation>,
    pub last_capture_tick: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HunterReading {
    pub id: SimId,
    pub reserve_units: u32,
    pub capacity_units: u32,
    pub maintenance_units_per_tick: u32,
    pub attack_units: u32,
    pub attack_effort_points: u32,
    pub first_eligible_tick: u64,
    pub cell: Cell,
    pub motion: Motion,
    pub target: Option<PreyTarget>,
    pub observed_tick: Option<u64>,
    pub observed_cell: Option<Cell>,
    pub visible_prey: Vec<PreyObservation>,
    pub fatigue_points: Option<u32>,
    pub activity: Activity,
    pub last_rest_tick: Option<u64>,
    pub last_transition_tick: Option<u64>,
    pub last_travel_tick: Option<u64>,
    pub last_capture_tick: Option<u64>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HuntCounters {
    pub maintenance_units: u64,
    pub travel_cells: u64,
    pub travel_units: u64,
    pub attack_units: u64,
    pub transferred_units: u64,
    pub captures: u64,
    pub hunter_starvations: u64,
    pub target_changes: u64,
    pub effort_points: u64,
    pub recovered_points: u64,
    pub rest_actions: u64,
    pub entered_rest: u64,
    pub woke: u64,
}
impl HuntCounters {
    fn adding(self, delta: Self) -> Self {
        Self {
            maintenance_units: self
                .maintenance_units
                .checked_add(delta.maintenance_units)
                .expect("hunter upkeep counter exhausted"),
            travel_cells: self
                .travel_cells
                .checked_add(delta.travel_cells)
                .expect("hunter distance counter exhausted"),
            travel_units: self
                .travel_units
                .checked_add(delta.travel_units)
                .expect("hunter travel counter exhausted"),
            attack_units: self
                .attack_units
                .checked_add(delta.attack_units)
                .expect("attack cost counter exhausted"),
            transferred_units: self
                .transferred_units
                .checked_add(delta.transferred_units)
                .expect("prey transfer counter exhausted"),
            captures: self
                .captures
                .checked_add(delta.captures)
                .expect("capture counter exhausted"),
            hunter_starvations: self
                .hunter_starvations
                .checked_add(delta.hunter_starvations)
                .expect("hunter death counter exhausted"),
            target_changes: self
                .target_changes
                .checked_add(delta.target_changes)
                .expect("hunter target counter exhausted"),
            effort_points: self
                .effort_points
                .checked_add(delta.effort_points)
                .expect("hunter effort counter exhausted"),
            recovered_points: self
                .recovered_points
                .checked_add(delta.recovered_points)
                .expect("hunter recovery counter exhausted"),
            rest_actions: self
                .rest_actions
                .checked_add(delta.rest_actions)
                .expect("hunter rest counter exhausted"),
            entered_rest: self
                .entered_rest
                .checked_add(delta.entered_rest)
                .expect("hunter rest entry exhausted"),
            woke: self
                .woke
                .checked_add(delta.woke)
                .expect("hunter wake counter exhausted"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntSnapshot {
    pub version: &'static str,
    pub run: u64,
    pub tick: u64,
    pub config: HuntConfig,
    pub initial_hunters: usize,
    pub living_hunters: usize,
    pub hunters: Vec<HunterReading>,
    pub ledger: HuntCounters,
    pub totals: HuntCounters,
    pub history: HuntHistorySnapshot,
}
#[derive(Resource)]
pub(crate) struct HuntState {
    pub config: HuntConfig,
    pub ledger: HuntCounters,
    pub totals: HuntCounters,
    pub history: HuntHistory,
}
impl HuntState {
    pub fn planned(&self, delta: HuntCounters) -> (HuntCounters, HuntCounters) {
        let ledger = self.ledger.adding(delta);
        let totals = self.totals.adding(delta);
        assert!(
            self.history.can_record(),
            "hunt history exhausted before transaction"
        );
        (ledger, totals)
    }
    pub fn commit(&mut self, planned: (HuntCounters, HuntCounters), event: HuntEvent) {
        self.history.record(event);
        self.ledger = planned.0;
        self.totals = planned.1;
    }
}
pub(crate) fn install(world: &mut World, initial: &MobileScenario) {
    let Some(config) = initial.hunters() else {
        return;
    };
    let mut entities = Vec::new();
    for seed in &config.hunters {
        entities.push(
            world
                .spawn((
                    Identity(seed.id),
                    Body {
                        reserve_units: seed.reserve_units,
                        capacity_units: seed.capacity_units,
                        maintenance_units_per_tick: seed.maintenance_units_per_tick,
                        terminal: Terminal::Alive,
                        first_eligible_tick: 1,
                    },
                    Hunter {
                        attack_units: seed.attack_units,
                        attack_effort_points: seed.attack_effort_points,
                        target: None,
                        observed_tick: None,
                        observed_cell: None,
                        visible_prey: Vec::new(),
                        last_capture_tick: None,
                    },
                    Position(seed.cell),
                    MobileActor::new(seed.motion).with_rest(initial.rest().is_some()),
                ))
                .id(),
        );
    }
    world.resource_mut::<Roster>().hunters = entities;
    world.insert_resource(HuntState {
        config: config.clone(),
        ledger: Default::default(),
        totals: Default::default(),
        history: HuntHistory::new(config.history_limit),
    });
}
pub(crate) fn snapshot(world: &World) -> Option<HuntSnapshot> {
    let state = world.get_resource::<HuntState>()?;
    let clock = world.resource::<Clock>();
    let mut hunters: Vec<_> = world
        .resource::<Roster>()
        .hunters
        .iter()
        .map(|entity| {
            let id = world.get::<Identity>(*entity).expect("hunter identity").0;
            let body = world.get::<Body>(*entity).expect("hunter body");
            let hunter = world.get::<Hunter>(*entity).expect("hunter role");
            let actor = world.get::<MobileActor>(*entity).expect("hunter motion");
            HunterReading {
                id,
                reserve_units: body.reserve_units,
                capacity_units: body.capacity_units,
                maintenance_units_per_tick: body.maintenance_units_per_tick,
                attack_units: hunter.attack_units,
                attack_effort_points: hunter.attack_effort_points,
                first_eligible_tick: body.first_eligible_tick,
                cell: world.get::<Position>(*entity).expect("hunter cell").0,
                motion: actor.motion,
                target: hunter.target,
                observed_tick: hunter.observed_tick,
                observed_cell: hunter.observed_cell,
                visible_prey: hunter.visible_prey.clone(),
                fatigue_points: actor.rest.map(|r| r.fatigue_points),
                activity: actor.rest.map_or(Activity::Foraging, |r| r.activity),
                last_rest_tick: actor.rest.and_then(|r| r.last_rest_tick),
                last_transition_tick: actor.rest.and_then(|r| r.last_transition_tick),
                last_travel_tick: actor.last_travel_tick,
                last_capture_tick: hunter.last_capture_tick,
            }
        })
        .collect();
    hunters.sort_by_key(|h| h.id);
    Some(HuntSnapshot {
        version: HUNT_VERSION,
        run: clock.run,
        tick: clock.tick,
        config: state.config.clone(),
        initial_hunters: state.config.hunters.len(),
        living_hunters: hunters.len(),
        hunters,
        ledger: state.ledger,
        totals: state.totals,
        history: state.history.snapshot(),
    })
}
pub(crate) fn begin(state: Option<ResMut<HuntState>>) {
    if let Some(mut state) = state {
        state.ledger = Default::default();
    }
}
pub(crate) fn complete(clock: Res<Clock>, state: Option<ResMut<HuntState>>) {
    if let Some(mut state) = state {
        state.history.collected_through_tick = clock.tick;
    }
}
