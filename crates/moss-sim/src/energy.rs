//! Metabolic reserves and the species rates used by maintenance.

use bevy_ecs::prelude::{Component, Resource};

use crate::Species;

/// Incremental travel cost for the first movement experiment, above maintenance.
/// This small shared setting can later initialize individual cost components.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovementRules {
    pub units_per_cell: u32,
}

impl Default for MovementRules {
    fn default() -> Self {
        Self { units_per_cell: 2 }
    }
}

/// Metabolic energy units. Maintenance spends the configured species rate per tick,
/// bounded at zero. The default rate is one for both animal species.
/// No replenishment or death rule is installed yet.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Energy {
    pub reserve: u32,
    pub capacity: u32,
}

/// Animal maintenance costs in energy units per executed simulation tick.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeciesEnergyRules {
    pub hare_maintenance_units_per_tick: u32,
    pub fox_maintenance_units_per_tick: u32,
}

impl Default for SpeciesEnergyRules {
    fn default() -> Self {
        Self {
            hare_maintenance_units_per_tick: 1,
            fox_maintenance_units_per_tick: 1,
        }
    }
}

impl SpeciesEnergyRules {
    pub fn maintenance_units_per_tick(&self, species: Species) -> Option<u32> {
        match species {
            Species::Hare => Some(self.hare_maintenance_units_per_tick),
            Species::Fox => Some(self.fox_maintenance_units_per_tick),
            Species::Grass => None,
        }
    }
}
