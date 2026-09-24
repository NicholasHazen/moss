//! A proposed course ecology, isolated from the learner-owned live Moss world.
//!
//! One ordered Bevy world combines upkeep, bounded daylight supply, competing
//! meals, and starvation. An opt-in population policy adds funded births,
//! maturation, and inherited upkeep. Travel, rest, and predation remain later arcs.

mod feeding;
mod history;
mod lifecycle;
mod model;
mod population;
mod reproduction;
mod simulation;
mod supply;

pub use history::{Event, EventKind, HistorySnapshot};
pub use model::{
    Daylight, GrazerReading, GrazerSeed, Life, PatchReading, PatchSeed, Scenario, ScenarioError,
    SimId, Snapshot, TickLedger,
};
pub use population::{
    BirthOrigin, BirthRecord, FounderTrait, INHERITANCE_ALGORITHM, POPULATION_VERSION,
    PopulationConfig, PopulationError, PopulationEvent, PopulationEventKind,
    PopulationHistorySnapshot, PopulationLedger, PopulationReading, PopulationSnapshot,
    PopulationTotals, Stage, UpkeepTrait,
};
pub use simulation::CourseWorld;
