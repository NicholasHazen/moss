//! Opt-in population state, owned observations, stage transitions, and cleanup.

mod config;
mod history;

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::prelude::*;

use crate::model::{Clock, Grazer, Identity, Life, TickLedger};
use crate::simulation::Roster;
use crate::{Scenario, SimId};

pub use config::{
    FounderTrait, INHERITANCE_ALGORITHM, POPULATION_VERSION, PopulationConfig, PopulationError,
    UpkeepTrait,
};
use history::PopulationHistory;
pub use history::{BirthRecord, PopulationEvent, PopulationEventKind, PopulationHistorySnapshot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Juvenile,
    Adult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BirthOrigin {
    pub run: u64,
    pub birth_tick: u64,
    pub birth_ordinal: u64,
    pub parents: [SimId; 2],
    pub donor: SimId,
    pub inherited: UpkeepTrait,
    pub proposed_delta: i8,
    pub applied_delta: i8,
    pub child_trait: UpkeepTrait,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Individual {
    pub stage: Stage,
    pub matures_at_tick: Option<u64>,
    pub next_birth_tick: u64,
    pub upkeep: UpkeepTrait,
    pub origin: Option<BirthOrigin>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopulationReading {
    pub id: SimId,
    pub stage: Stage,
    pub first_eligible_tick: u64,
    pub matures_at_tick: Option<u64>,
    pub next_birth_tick: u64,
    pub upkeep: UpkeepTrait,
    pub origin: Option<BirthOrigin>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopulationLedger {
    pub births: u64,
    pub transferred_units: u64,
    pub dissipated_units: u64,
    pub capacity_blocked_pairs: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopulationTotals {
    pub initial_founders: u64,
    pub births: u64,
    pub starvations: u64,
    pub capacity_blocked_ticks: u64,
    pub capacity_blocked_pairs: u64,
}

/// Supplemental v1 report; the first-arc Snapshot and EventKind are unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopulationSnapshot {
    pub version: &'static str,
    pub inheritance_algorithm: &'static str,
    pub run: u64,
    pub tick: u64,
    pub initial_scenario: Scenario,
    pub config: PopulationConfig,
    pub individuals: Vec<PopulationReading>,
    pub living: usize,
    pub juveniles: usize,
    pub adults: usize,
    pub totals: PopulationTotals,
    pub ledger: PopulationLedger,
    pub next_child_id: Option<SimId>,
    pub next_birth_ordinal: u64,
    pub history: PopulationHistorySnapshot,
}

#[derive(Resource)]
pub(crate) struct PopulationState {
    pub config: PopulationConfig,
    pub next_child_id: Option<SimId>,
    pub totals: PopulationTotals,
    pub ledger: PopulationLedger,
    pub history: PopulationHistory,
}

pub(crate) fn install(world: &mut World, config: PopulationConfig, next_child_id: SimId) {
    let founders: BTreeMap<_, _> = config
        .founders
        .iter()
        .map(|founder| (founder.id, founder.upkeep))
        .collect();
    let entities = world.resource::<Roster>().grazers.clone();
    for entity in &entities {
        let id = world
            .get::<Identity>(*entity)
            .expect("founder identity exists")
            .0;
        world.entity_mut(*entity).insert(Individual {
            stage: Stage::Adult,
            matures_at_tick: None,
            next_birth_tick: 1,
            upkeep: founders[&id],
            origin: None,
        });
    }
    let state = PopulationState {
        next_child_id: Some(next_child_id),
        totals: PopulationTotals {
            initial_founders: entities.len() as u64,
            births: 0,
            starvations: 0,
            capacity_blocked_ticks: 0,
            capacity_blocked_pairs: 0,
        },
        ledger: PopulationLedger::default(),
        history: PopulationHistory::new(config.history_limit),
        config,
    };
    world.insert_resource(state);
}

pub(crate) fn snapshot(world: &World, initial: &Scenario) -> Option<PopulationSnapshot> {
    let state = world.get_resource::<PopulationState>()?;
    let mut individuals: Vec<_> = world
        .resource::<Roster>()
        .grazers
        .iter()
        .map(|entity| {
            let id = world
                .get::<Identity>(*entity)
                .expect("rostered identity exists")
                .0;
            let grazer = world
                .get::<Grazer>(*entity)
                .expect("rostered grazer exists");
            let individual = world
                .get::<Individual>(*entity)
                .expect("population member has stage data");
            PopulationReading {
                id,
                stage: individual.stage,
                first_eligible_tick: grazer.first_eligible_tick,
                matures_at_tick: individual.matures_at_tick,
                next_birth_tick: individual.next_birth_tick,
                upkeep: individual.upkeep,
                origin: individual.origin,
            }
        })
        .collect();
    individuals.sort_by_key(|individual| individual.id);
    let juveniles = individuals
        .iter()
        .filter(|individual| individual.stage == Stage::Juvenile)
        .count();
    let clock = world.resource::<Clock>();
    Some(PopulationSnapshot {
        version: POPULATION_VERSION,
        inheritance_algorithm: INHERITANCE_ALGORITHM,
        run: clock.run,
        tick: clock.tick,
        initial_scenario: initial.clone(),
        config: state.config.clone(),
        living: individuals.len(),
        juveniles,
        adults: individuals.len() - juveniles,
        individuals,
        totals: state.totals,
        ledger: state.ledger,
        next_child_id: state.next_child_id,
        next_birth_ordinal: state.totals.births,
        history: state.history.snapshot(),
    })
}

pub(crate) fn begin_population_tick(mut state: ResMut<PopulationState>) {
    state.ledger = PopulationLedger::default();
}

pub(crate) fn mature(
    clock: Res<Clock>,
    mut individuals: Query<(Entity, &Identity, &Grazer, &mut Individual)>,
    mut state: ResMut<PopulationState>,
) {
    let mut order: Vec<_> = individuals
        .iter()
        .map(|(entity, id, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, grazer, mut individual) = individuals
            .get_mut(entity)
            .expect("listed individual exists");
        if grazer.life == Life::Alive
            && individual.stage == Stage::Juvenile
            && individual
                .matures_at_tick
                .is_some_and(|tick| clock.tick >= tick)
        {
            assert!(
                state.history.can_record(),
                "population event eviction counter exhausted"
            );
            individual.stage = Stage::Adult;
            state.history.record(PopulationEvent {
                run: clock.run,
                tick: clock.tick,
                kind: PopulationEventKind::Matured {
                    individual: id,
                    upkeep: individual.upkeep,
                },
            });
        }
    }
}

pub(crate) fn cleanup_dead(
    mut commands: Commands,
    grazers: Query<(Entity, &Grazer)>,
    mut roster: ResMut<Roster>,
) {
    let dead: BTreeSet<_> = grazers
        .iter()
        .filter(|(_, grazer)| grazer.life == Life::Dead)
        .map(|(entity, _)| entity)
        .collect();
    roster.grazers.retain(|entity| !dead.contains(entity));
    for entity in dead {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn complete_population_tick(
    clock: Res<Clock>,
    ledger: Res<TickLedger>,
    mut state: ResMut<PopulationState>,
) {
    state.totals.starvations = state
        .totals
        .starvations
        .checked_add(ledger.starvations)
        .expect("population starvation total exhausted");
    if state.ledger.capacity_blocked_pairs > 0 {
        state.totals.capacity_blocked_ticks = state
            .totals
            .capacity_blocked_ticks
            .checked_add(1)
            .expect("capacity-blocked tick counter exhausted");
    }
    state.totals.capacity_blocked_pairs = state
        .totals
        .capacity_blocked_pairs
        .checked_add(state.ledger.capacity_blocked_pairs)
        .expect("capacity-blocked pair counter exhausted");
    state.history.collected_through_tick = clock.tick;
}
