//! Moss's authoritative state and explicitly requested simulation ticks.
//! This crate has no renderer, wall clock, or browser APIs. Baseline maintenance
//! is the first biological rule; movement, feeding, and death remain future work.
//!
//! Start with [`Energy`], then the [`lessons::spend_energy`] system.
//! [`install`] owns schedule wiring. `fixture.rs` owns the authored starting scene.

mod fixture;
pub mod lessons;

use std::{collections::VecDeque, error::Error, fmt};

use bevy_ecs::{
    prelude::*,
    schedule::{ExecutorKind, ScheduleLabel},
};

/// Simulated seconds represented by one complete tick (four ticks per second).
pub const TICK_SECONDS: f32 = 0.25;
const JOURNAL_CAPACITY: usize = 32;

/// A stable application identity within a run. History uses `(run number, SimId)`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimId(pub u64);

/// Authoritative integer cell coordinates, with `(0, 0)` at the lower left.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// An animal individual. `name` is a nickname, never a species or rule selector.
/// Species and ecological role are separate components on the same entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Creature {
    pub name: &'static str,
}

/// Authored species groups for inspection and later population summaries.
/// These are simple game species, not calibrated models of real animals/plants.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Species {
    Hare,
    Fox,
    Grass,
}

impl Species {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hare => "Hare",
            Self::Fox => "Fox",
            Self::Grass => "Grass",
        }
    }
}

/// A food-web role, independent of species and nickname; no policy runs yet.
/// "Archetype" is reserved for Bevy's set-of-components meaning.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EcologicalRole {
    Grazer,
    Hunter,
    Producer,
}

impl EcologicalRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::Grazer => "grazer",
            Self::Hunter => "hunter",
            Self::Producer => "producer",
        }
    }
}

/// Metabolic energy units. Maintenance spends one per tick, bounded at zero.
/// No replenishment or death rule is installed yet.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Energy {
    pub reserve: u32,
    pub capacity: u32,
}

/// Available biomass units. This bootstrap patch does not grow or get consumed.
/// One entity represents a local stand of plants, not an individual blade or CA.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoodPatch {
    pub name: &'static str,
    pub biomass: u32,
}

/// Dimensions in cells. Changing dimensions begins a new authored run.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfig {
    width: u32,
    height: u32,
}

impl WorldConfig {
    /// Keep the authored fixture legible and the bootstrap world deliberately small.
    pub fn new(width: u32, height: u32) -> Result<Self, WorldConfigError> {
        if !(8..=256).contains(&width) || !(8..=256).contains(&height) {
            return Err(WorldConfigError { width, height });
        }
        Ok(Self { width, height })
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 32,
            height: 20,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfigError {
    width: u32,
    height: u32,
}

impl fmt::Display for WorldConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "world dimensions must each be 8..=256 cells; received {} × {}",
            self.width, self.height
        )
    }
}

impl Error for WorldConfigError {}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimClock {
    /// Number of fully completed simulation ticks in this run.
    pub tick: u64,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunRecord {
    pub number: u64,
}

/// Actual initialization outcomes; authored placement is deliberately not a birth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    RunStarted { number: u64 },
    FixturePlaced { name: &'static str },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalEntry {
    pub tick: u64,
    pub participants: Vec<SimId>,
    pub kind: EventKind,
}

/// Recent events for the current run only. Reset discards the previous run's history.
#[derive(Resource, Default, Debug)]
pub struct Journal {
    entries: VecDeque<JournalEntry>,
    evicted: u64,
}

impl Journal {
    /// Retained entries in chronological order (oldest first).
    pub fn entries(&self) -> &VecDeque<JournalEntry> {
        &self.entries
    }

    pub fn capacity(&self) -> usize {
        JOURNAL_CAPACITY
    }

    /// Events dropped from the front in this run. A nonzero count means incomplete history.
    pub fn evicted(&self) -> u64 {
        self.evicted
    }

    pub fn oldest_tick(&self) -> Option<u64> {
        self.entries.front().map(|entry| entry.tick)
    }

    fn record(&mut self, entry: JournalEntry) {
        if self.entries.len() == JOURNAL_CAPACITY {
            self.entries.pop_front();
            self.evicted = self.evicted.checked_add(1).expect("journal count overflow");
        }
        self.entries.push_back(entry);
    }
}

/// The browser and headless tests request the very same schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimTick;

/// Install the simulation in the host's ECS world and start its first run.
pub fn install(world: &mut World, config: WorldConfig) {
    world.insert_resource(config);
    world.init_resource::<RunRecord>();

    let mut schedule = Schedule::new(SimTick);
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    // `chain()` makes tuple order explicit: spend energy, then complete the tick.
    // The remaining lesson stubs are intentionally NOT scheduled.
    schedule.add_systems((lessons::spend_energy, complete_tick).chain());
    world.add_schedule(schedule);
    reset(world);
}

/// Replace only entities with simulation IDs; preserve camera/UI entities and resources.
pub fn reset(world: &mut World) {
    let simulation_entities: Vec<Entity> = world
        .query_filtered::<Entity, With<SimId>>()
        .iter(world)
        .collect();
    for entity in simulation_entities {
        world.despawn(entity);
    }

    let run_number = {
        let mut run = world.resource_mut::<RunRecord>();
        run.number = run.number.checked_add(1).expect("run number overflow");
        run.number
    };
    world.insert_resource(SimClock::default());
    world.insert_resource(Journal::default());
    world.resource_mut::<Journal>().record(JournalEntry {
        tick: 0,
        participants: Vec::new(),
        kind: EventKind::RunStarted { number: run_number },
    });
    fixture::place(world);
}

fn record_placement(world: &mut World, id: SimId, name: &'static str) {
    world.resource_mut::<Journal>().record(JournalEntry {
        tick: 0,
        participants: vec![id],
        kind: EventKind::FixturePlaced { name },
    });
}

/// Execute exactly one complete tick, independent of rendering or elapsed real time.
pub fn tick(world: &mut World) {
    world.run_schedule(SimTick);
}

fn complete_tick(mut clock: ResMut<SimClock>) {
    clock.tick = clock.tick.checked_add(1).expect("simulation tick overflow");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_eviction_reports_incomplete_coverage() {
        let mut journal = Journal::default();
        assert_eq!(journal.oldest_tick(), None);
        for tick in 0..35 {
            journal.record(JournalEntry {
                tick,
                participants: vec![SimId(tick + 1)],
                kind: EventKind::FixturePlaced {
                    name: "test fixture",
                },
            });
        }
        assert_eq!(journal.entries().len(), journal.capacity());
        assert_eq!(journal.evicted(), 3);
        assert_eq!(journal.oldest_tick(), Some(3));
        assert_eq!(journal.entries().back().unwrap().tick, 34);
    }
}
