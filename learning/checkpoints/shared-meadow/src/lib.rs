//! A proposed course ecology, isolated from the learner-owned live Moss world.
//!
//! One ordered Bevy world combines upkeep, bounded daylight supply, competing
//! meals, and starvation. There is no travel, predation, birth, or inheritance.

mod feeding;
mod history;
mod lifecycle;
mod model;
mod simulation;
mod supply;

pub use history::{Event, EventKind, HistorySnapshot};
pub use model::{
    Daylight, GrazerReading, GrazerSeed, Life, PatchReading, PatchSeed, Scenario, ScenarioError,
    SimId, Snapshot, TickLedger,
};
pub use simulation::CourseWorld;
