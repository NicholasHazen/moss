//! A proposed course ecology, isolated from the learner-owned live Moss world.
//!
//! One ordered Bevy world combines upkeep, bounded daylight supply, competing
//! meals, and starvation. An opt-in population policy adds funded births,
//! maturation, and inherited upkeep. Mobile mode adds local choice and paid travel;
//! optional fatigue and committed rest interrupt those actions. Hunters share those
//! body rules and resolve preflighted whole-reserve captures before grazer meals.

mod feeding;
mod history;
mod hunting;
pub use hunting::{
    HUNT_VERSION, HuntConfig, HuntCounters, HuntEvent, HuntEventKind, HuntHistorySnapshot,
    HuntSnapshot, HunterReading, HunterSeed, PreyObservation, PreyTarget,
};
mod lifecycle;
mod mobile;
mod model;
mod population;
mod refuge;
mod reproduction;
pub use refuge::{
    REFUGE_VERSION, RefugeConfig, RefugeGrazerReading, RefugeId, RefugeSeed, RefugeSnapshot,
};
mod simulation;
mod supply;

pub use history::{Event, EventKind, HistorySnapshot};
pub use mobile::{
    Activity, Cell, GrazerPlacement, GridBounds, MOBILE_VERSION, MobileError, MobileEvent,
    MobileEventKind, MobileGrazerReading, MobileHistorySnapshot, MobileLedger, MobileScenario,
    MobileSnapshot, MobileTotals, Motion, PatchObservation, PatchPlacement, PatchTarget,
    REST_VERSION, RestCounters, RestEvent, RestEventKind, RestGrazerReading, RestHistorySnapshot,
    RestPolicy, RestSnapshot, SpatialConfig,
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
