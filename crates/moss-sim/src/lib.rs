//! Moss's authoritative state and explicitly requested simulation ticks.
//! This crate has no renderer, wall clock, or browser APIs. Baseline maintenance
//! is the first biological rule; movement, feeding, and death remain future work.
//!
//! Start with [`Energy`], then the [`lessons::spend_energy`] system.
//! [`install`] owns schedule wiring. Modules group implementation by responsibility;
//! these re-exports keep the host and tests independent of the internal file layout.

mod components;
mod config;
mod energy;
mod fixture;
mod journal;
pub mod lessons;
mod simulation;

pub use components::{Creature, EcologicalRole, FoodPatch, FoodTarget, Position, SimId, Species};
pub use config::{WorldConfig, WorldConfigError};
pub use energy::{Energy, MovementRules, SpeciesEnergyRules};
pub use journal::{EventKind, Journal, JournalEntry};
pub use simulation::{RunRecord, SimClock, SimTick, TICK_SECONDS, install, reset, tick};
