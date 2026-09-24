//! Authored inputs, validated bounds, simulation data, and owned observations.

use std::collections::BTreeSet;
use std::fmt;

use bevy_ecs::prelude::*;

use crate::history::HistorySnapshot;

/// Stable application identity, unique across grazers and patches within a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Life {
    Alive,
    Dead,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Daylight {
    cycle_ticks: u64,
    lit_ticks: u64,
}

impl Daylight {
    /// Zero lit ticks and an entirely lit cycle are valid; a zero cycle is not.
    pub fn new(cycle_ticks: u64, lit_ticks: u64) -> Result<Self, ScenarioError> {
        if cycle_ticks == 0 || lit_ticks > cycle_ticks {
            return Err(ScenarioError::InvalidDaylight);
        }
        Ok(Self {
            cycle_ticks,
            lit_ticks,
        })
    }

    pub fn cycle_ticks(self) -> u64 {
        self.cycle_ticks
    }
    pub fn lit_ticks(self) -> u64 {
        self.lit_ticks
    }

    /// Tick zero has no executed environmental phase.
    pub fn is_lit(self, completed_tick: u64) -> Option<bool> {
        completed_tick
            .checked_sub(1)
            .map(|tick| tick % self.cycle_ticks < self.lit_ticks)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrazerSeed {
    pub id: SimId,
    pub reserve_units: u32,
    pub capacity_units: u32,
    pub maintenance_units_per_tick: u32,
    pub meal_units_per_tick: u32,
    /// Explicit contact with this patch. None or an absent patch permits no meal.
    pub feeding_site: Option<SimId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchSeed {
    pub id: SimId,
    pub biomass_units: u32,
    pub capacity_units: u32,
    pub growth_units_per_lit_tick: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub daylight: Daylight,
    pub grazers: Vec<GrazerSeed>,
    pub patches: Vec<PatchSeed>,
    /// Maximum retained events, including zero for intentionally absent history.
    pub history_limit: usize,
}

impl Scenario {
    /// Two fixed feeding contacts compete for two new units on each lit tick.
    pub fn limited_supply() -> Self {
        Self::comparison(2)
    }

    /// Same population and daylight; only lit-tick supply changes from two to six.
    pub fn generous_supply() -> Self {
        Self::comparison(6)
    }

    fn comparison(growth_units_per_lit_tick: u32) -> Self {
        Self {
            daylight: Daylight::new(4, 2).expect("authored four-tick day is valid"),
            grazers: vec![
                GrazerSeed {
                    id: SimId(1),
                    reserve_units: 3,
                    capacity_units: 8,
                    maintenance_units_per_tick: 2,
                    meal_units_per_tick: 2,
                    feeding_site: Some(SimId(100)),
                },
                GrazerSeed {
                    id: SimId(2),
                    reserve_units: 3,
                    capacity_units: 8,
                    maintenance_units_per_tick: 1,
                    meal_units_per_tick: 2,
                    feeding_site: Some(SimId(100)),
                },
            ],
            patches: vec![PatchSeed {
                id: SimId(100),
                biomass_units: 0,
                capacity_units: 8,
                growth_units_per_lit_tick,
            }],
            history_limit: 128,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ScenarioError> {
        let mut ids = BTreeSet::new();
        for grazer in &self.grazers {
            if !ids.insert(grazer.id) {
                return Err(ScenarioError::DuplicateId(grazer.id));
            }
            if grazer.reserve_units > grazer.capacity_units {
                return Err(ScenarioError::ReserveOverCapacity(grazer.id));
            }
        }
        for patch in &self.patches {
            if !ids.insert(patch.id) {
                return Err(ScenarioError::DuplicateId(patch.id));
            }
            if patch.biomass_units > patch.capacity_units {
                return Err(ScenarioError::BiomassOverCapacity(patch.id));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioError {
    InvalidDaylight,
    DuplicateId(SimId),
    ReserveOverCapacity(SimId),
    BiomassOverCapacity(SimId),
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid course scenario: {self:?}")
    }
}
impl std::error::Error for ScenarioError {}

#[derive(Component, Clone, Copy)]
pub(crate) struct Identity(pub SimId);

#[derive(Component)]
pub(crate) struct Grazer {
    pub reserve_units: u32,
    pub capacity_units: u32,
    pub maintenance_units_per_tick: u32,
    pub meal_units_per_tick: u32,
    pub feeding_site: Option<SimId>,
    pub life: Life,
    pub first_eligible_tick: u64,
}

#[derive(Component)]
pub(crate) struct Patch {
    pub biomass_units: u32,
    pub capacity_units: u32,
    pub growth_units_per_lit_tick: u32,
}

#[derive(Resource)]
pub(crate) struct Clock {
    pub run: u64,
    pub tick: u64,
}

/// Actual flows during the latest completed tick, not requested amounts.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickLedger {
    pub added_units: u64,
    pub maintenance_units: u64,
    pub eaten_units: u64,
    pub starvations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrazerReading {
    pub id: SimId,
    pub reserve_units: u32,
    pub capacity_units: u32,
    pub maintenance_units_per_tick: u32,
    pub meal_units_per_tick: u32,
    pub feeding_site: Option<SimId>,
    pub life: Life,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchReading {
    pub id: SimId,
    pub biomass_units: u32,
    pub capacity_units: u32,
    pub growth_units_per_lit_tick: u32,
}

/// Owned report. Mutating a report cannot mutate the ECS world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub run: u64,
    pub tick: u64,
    pub daylight: Daylight,
    pub lit: Option<bool>,
    /// Both populations are sorted by stable ID, independent of spawn order.
    pub grazers: Vec<GrazerReading>,
    pub patches: Vec<PatchReading>,
    pub ledger: TickLedger,
    pub history: HistorySnapshot,
}
