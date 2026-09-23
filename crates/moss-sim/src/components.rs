//! Identity, location, and ecological classification of simulation entities.

use bevy_ecs::prelude::Component;

/// A stable application identity within a run. History uses `(run number, SimId)`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimId(pub u64);

/// Authoritative integer cell coordinates, with `(0, 0)` at the lower left.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Accepted destination, identified within this run rather than by an ECS handle.
/// The diagnostic fixture authors Fern's target; autonomous choice is a later lesson.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoodTarget(pub SimId);

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

/// Available biomass units. This bootstrap patch does not grow or get consumed.
/// One entity represents a local stand of plants, not an individual blade or CA.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoodPatch {
    pub name: &'static str,
    pub biomass: u32,
}
