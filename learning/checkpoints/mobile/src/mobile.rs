//! Opt-in space, observations, travel, and owned projections of the same world.

mod config;
mod history;
pub(crate) mod movement;
pub(crate) mod perception;
mod scenarios;

use crate::model::{Clock, Grazer, Identity, Life, Patch};
use crate::simulation::Roster;
use crate::{PopulationSnapshot, SimId, Snapshot};
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

pub use config::{
    Cell, GrazerPlacement, GridBounds, MobileError, MobileScenario, Motion, PatchPlacement,
    SpatialConfig,
};
use history::MobileHistory;
pub use history::{MobileEvent, MobileEventKind, MobileHistorySnapshot};

pub const MOBILE_VERSION: &str = "moss-course-mobile-a-v1";

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Position(pub Cell);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatchObservation {
    pub patch: SimId,
    pub cell: Cell,
    pub biomass_units: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatchTarget {
    pub patch: SimId,
    pub observed_cell: Cell,
    pub observed_tick: u64,
}

#[derive(Component, Clone)]
pub(crate) struct MobileActor {
    pub motion: Motion,
    pub target: Option<PatchTarget>,
    pub observed_tick: Option<u64>,
    pub observed_cell: Option<Cell>,
    pub visible_patches: Vec<PatchObservation>,
    pub last_travel_tick: Option<u64>,
}

impl MobileActor {
    pub fn new(motion: Motion) -> Self {
        Self {
            motion,
            target: None,
            observed_tick: None,
            observed_cell: None,
            visible_patches: Vec::new(),
            last_travel_tick: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileGrazerReading {
    pub id: SimId,
    pub cell: Cell,
    pub motion: Motion,
    pub target: Option<PatchTarget>,
    pub observed_tick: Option<u64>,
    pub observed_cell: Option<Cell>,
    pub visible_patches: Vec<PatchObservation>,
    pub last_travel_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MobileLedger {
    pub travel_cells: u64,
    pub travel_units: u64,
    pub target_changes: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MobileTotals {
    pub travel_cells: u64,
    pub travel_units: u64,
    pub target_changes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileSnapshot {
    pub version: &'static str,
    pub run: u64,
    pub tick: u64,
    pub initial: MobileScenario,
    pub base: Snapshot,
    pub population: PopulationSnapshot,
    pub grazers: Vec<MobileGrazerReading>,
    pub patches: Vec<PatchPlacement>,
    pub ledger: MobileLedger,
    pub totals: MobileTotals,
    pub history: MobileHistorySnapshot,
}

#[derive(Resource)]
pub(crate) struct MobileState {
    pub newborn_motion: Motion,
    pub ledger: MobileLedger,
    pub totals: MobileTotals,
    pub history: MobileHistory,
}

pub(crate) fn install(world: &mut World, initial: &MobileScenario) {
    let grazers: BTreeMap<_, _> = initial
        .space()
        .grazers
        .iter()
        .map(|placement| (placement.id, *placement))
        .collect();
    let patches: BTreeMap<_, _> = initial
        .space()
        .patches
        .iter()
        .map(|placement| (placement.id, placement.cell))
        .collect();
    let roster = world.resource::<Roster>();
    let grazer_entities = roster.grazers.clone();
    let patch_entities = roster.patches.clone();
    for entity in grazer_entities {
        let id = world
            .get::<Identity>(entity)
            .expect("grazer identity exists")
            .0;
        let placement = grazers[&id];
        world
            .entity_mut(entity)
            .insert((Position(placement.cell), MobileActor::new(placement.motion)));
    }
    for entity in patch_entities {
        let id = world
            .get::<Identity>(entity)
            .expect("patch identity exists")
            .0;
        world.entity_mut(entity).insert(Position(patches[&id]));
    }
    world.insert_resource(MobileState {
        newborn_motion: initial.space().newborn_motion,
        ledger: MobileLedger::default(),
        totals: MobileTotals::default(),
        history: MobileHistory::new(initial.history_limit()),
    });
}

pub(crate) fn snapshot(
    world: &World,
    initial: &MobileScenario,
    base: Snapshot,
    population: PopulationSnapshot,
) -> MobileSnapshot {
    let roster = world.resource::<Roster>();
    let mut grazers: Vec<_> = roster
        .grazers
        .iter()
        .map(|entity| {
            let id = world.get::<Identity>(*entity).expect("identity exists").0;
            let cell = world
                .get::<Position>(*entity)
                .expect("mobile grazer is placed")
                .0;
            let actor = world
                .get::<MobileActor>(*entity)
                .expect("mobile actor exists");
            MobileGrazerReading {
                id,
                cell,
                motion: actor.motion,
                target: actor.target,
                observed_tick: actor.observed_tick,
                observed_cell: actor.observed_cell,
                visible_patches: actor.visible_patches.clone(),
                last_travel_tick: actor.last_travel_tick,
            }
        })
        .collect();
    grazers.sort_by_key(|grazer| grazer.id);
    let mut patches: Vec<_> = roster
        .patches
        .iter()
        .map(|entity| PatchPlacement {
            id: world
                .get::<Identity>(*entity)
                .expect("patch identity exists")
                .0,
            cell: world
                .get::<Position>(*entity)
                .expect("mobile patch is placed")
                .0,
        })
        .collect();
    patches.sort_by_key(|patch| patch.id);
    let state = world.resource::<MobileState>();
    MobileSnapshot {
        version: MOBILE_VERSION,
        run: base.run,
        tick: base.tick,
        initial: initial.clone(),
        base,
        population,
        grazers,
        patches,
        ledger: state.ledger,
        totals: state.totals,
        history: state.history.snapshot(),
    }
}

pub(crate) fn begin_mobile_tick(mut state: ResMut<MobileState>) {
    state.ledger = MobileLedger::default();
}

/// Derive a contact token from current space; feeding rechecks the actual contact.
pub(crate) fn derive_contacts(
    clock: Res<Clock>,
    patches: Query<(&Identity, &Position), With<Patch>>,
    mut grazers: Query<(&Position, &MobileActor, &mut Grazer)>,
) {
    let cells: BTreeMap<_, _> = patches
        .iter()
        .map(|(id, position)| (id.0, position.0))
        .collect();
    for (position, actor, mut grazer) in &mut grazers {
        grazer.feeding_site =
            if grazer.life == Life::Alive && clock.tick >= grazer.first_eligible_tick {
                actor
                    .target
                    .filter(|target| {
                        target.observed_tick == clock.tick
                            && cells.get(&target.patch) == Some(&position.0)
                    })
                    .map(|target| target.patch)
            } else {
                None
            };
    }
}

pub(crate) fn complete_mobile_tick(clock: Res<Clock>, mut state: ResMut<MobileState>) {
    state.history.collected_through_tick = clock.tick;
}
