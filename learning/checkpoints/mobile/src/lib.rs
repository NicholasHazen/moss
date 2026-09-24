//! A proposed course ecology, isolated from the learner-owned live Moss world.
//!
//! One ordered Bevy world combines upkeep, bounded daylight supply, competing
//! meals, and starvation. An opt-in population policy adds funded births,
//! maturation, and inherited upkeep. Mobile mode adds local choice and paid travel;
//! rest and predation remain later arcs.

mod feeding;
mod history;
mod lifecycle;
mod mobile;
mod model;
mod population;
mod reproduction;
mod simulation;
mod supply;

pub use history::{Event, EventKind, HistorySnapshot};
pub use mobile::{
    Cell, GrazerPlacement, GridBounds, MOBILE_VERSION, MobileError, MobileEvent, MobileEventKind,
    MobileGrazerReading, MobileHistorySnapshot, MobileLedger, MobileScenario, MobileSnapshot,
    MobileTotals, Motion, PatchObservation, PatchPlacement, PatchTarget, SpatialConfig,
};
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
