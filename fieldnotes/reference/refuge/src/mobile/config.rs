//! Explicit mobile inputs; later extensions need not change constructor callers.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::rest::RestPolicy;
use crate::{PopulationConfig, PopulationError, Scenario, SimId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
}

impl Cell {
    pub fn distance(self, other: Self) -> u64 {
        u64::from(self.x.abs_diff(other.x)) + u64::from(self.y.abs_diff(other.y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridBounds {
    pub width_cells: u32,
    pub height_cells: u32,
}

impl GridBounds {
    pub fn contains(self, cell: Cell) -> bool {
        cell.x < self.width_cells && cell.y < self.height_cells
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Motion {
    pub sensing_radius: u32,
    pub max_cells_per_tick: u32,
    pub travel_units_per_cell: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrazerPlacement {
    pub id: SimId,
    pub cell: Cell,
    pub motion: Motion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatchPlacement {
    pub id: SimId,
    pub cell: Cell,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpatialConfig {
    pub bounds: GridBounds,
    pub grazers: Vec<GrazerPlacement>,
    pub patches: Vec<PatchPlacement>,
    pub newborn_motion: Motion,
}

/// Construct raw authored inputs here; CourseWorld validates before installation.
/// Private fields permit later optional policies without changing struct literals.
#[derive(Clone, PartialEq, Eq)]
pub struct MobileScenario {
    scenario: Scenario,
    population: PopulationConfig,
    space: SpatialConfig,
    history_limit: usize,
    rest: Option<RestPolicy>,
    hunters: Option<crate::HuntConfig>,
    refuges: Option<crate::RefugeConfig>,
}

impl fmt::Debug for MobileScenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("MobileScenario");
        debug
            .field("scenario", &self.scenario)
            .field("population", &self.population)
            .field("space", &self.space)
            .field("history_limit", &self.history_limit)
            .field("rest", &self.rest)
            .field("hunters", &self.hunters);
        // Keep existing off-mode report traces unchanged.
        if let Some(refuges) = &self.refuges {
            debug.field("refuges", refuges);
        }
        debug.finish()
    }
}

impl MobileScenario {
    pub fn new(scenario: Scenario, population: PopulationConfig, space: SpatialConfig) -> Self {
        Self {
            scenario,
            population,
            space,
            history_limit: 128,
            rest: None,
            hunters: None,
            refuges: None,
        }
    }

    pub fn with_refuges(mut self, config: crate::RefugeConfig) -> Self {
        self.refuges = Some(config);
        self
    }
    pub fn refuges(&self) -> Option<&crate::RefugeConfig> {
        self.refuges.as_ref()
    }

    pub fn with_hunters(mut self, config: crate::HuntConfig) -> Self {
        self.hunters = Some(config);
        self
    }
    pub fn hunters(&self) -> Option<&crate::HuntConfig> {
        self.hunters.as_ref()
    }

    pub fn with_rest(mut self, policy: RestPolicy) -> Self {
        self.rest = Some(policy);
        self
    }
    pub fn rest(&self) -> Option<&RestPolicy> {
        self.rest.as_ref()
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }
    pub fn population(&self) -> &PopulationConfig {
        &self.population
    }
    pub fn space(&self) -> &SpatialConfig {
        &self.space
    }
    pub fn history_limit(&self) -> usize {
        self.history_limit
    }
    pub fn with_history_limit(mut self, limit: usize) -> Self {
        self.history_limit = limit;
        self
    }

    pub(crate) fn validate_and_normalize(&mut self) -> Result<SimId, MobileError> {
        let next_id = self
            .population
            .validate(&self.scenario)
            .map_err(MobileError::Population)?;
        if self.rest.is_some_and(|policy| !policy.valid()) {
            return Err(MobileError::InvalidRestPolicy);
        }
        let bounds = self.space.bounds;
        if bounds.width_cells == 0 || bounds.height_cells == 0 {
            return Err(MobileError::InvalidBounds);
        }
        if self.space.newborn_motion.travel_units_per_cell == 0 {
            return Err(MobileError::ZeroNewbornTravelCost);
        }
        let grazers: BTreeMap<_, _> = self
            .scenario
            .grazers
            .iter()
            .map(|seed| (seed.id, seed))
            .collect();
        for seed in grazers.values() {
            if seed.feeding_site.is_some() {
                return Err(MobileError::AuthoredContact(seed.id));
            }
        }
        let mut seen = BTreeSet::new();
        for placement in &self.space.grazers {
            if !seen.insert(placement.id) {
                return Err(MobileError::DuplicateGrazerPlacement(placement.id));
            }
            if !grazers.contains_key(&placement.id) {
                return Err(MobileError::UnexpectedGrazerPlacement(placement.id));
            }
            if !bounds.contains(placement.cell) {
                return Err(MobileError::OutOfBounds(placement.id));
            }
            if placement.motion.travel_units_per_cell == 0 {
                return Err(MobileError::ZeroTravelCost(placement.id));
            }
        }
        for id in grazers.keys() {
            if !seen.contains(id) {
                return Err(MobileError::MissingGrazerPlacement(*id));
            }
        }
        let patches: BTreeSet<_> = self.scenario.patches.iter().map(|seed| seed.id).collect();
        seen.clear();
        let mut cells = BTreeSet::new();
        for placement in &self.space.patches {
            if !seen.insert(placement.id) {
                return Err(MobileError::DuplicatePatchPlacement(placement.id));
            }
            if !patches.contains(&placement.id) {
                return Err(MobileError::UnexpectedPatchPlacement(placement.id));
            }
            if !bounds.contains(placement.cell) {
                return Err(MobileError::OutOfBounds(placement.id));
            }
            if !cells.insert(placement.cell) {
                return Err(MobileError::SharedPatchCell(placement.cell));
            }
        }
        for id in patches {
            if !seen.contains(&id) {
                return Err(MobileError::MissingPatchPlacement(id));
            }
        }
        self.scenario.grazers.sort_by_key(|seed| seed.id);
        self.scenario.patches.sort_by_key(|seed| seed.id);
        self.population.founders.sort_by_key(|founder| founder.id);
        self.space.grazers.sort_by_key(|placement| placement.id);
        self.space.patches.sort_by_key(|placement| placement.id);
        let next_id = if let Some(config) = &self.hunters {
            config.validate(self, next_id)?
        } else {
            next_id
        };
        if let Some(config) = &mut self.hunters {
            config.hunters.sort_by_key(|hunter| hunter.id);
        }
        if let Some(config) = &mut self.refuges {
            config.validate(bounds)?;
            config.sites.sort_by_key(|site| site.id);
        }
        Ok(next_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobileError {
    Population(PopulationError),
    InvalidBounds,
    InvalidRestPolicy,
    ZeroRefugeId,
    DuplicateRefugeId(crate::RefugeId),
    RefugeOutOfBounds(crate::RefugeId),
    SharedRefugeCell(Cell),
    DuplicateHunterId(SimId),
    HunterOverCapacity(SimId),
    InvalidAttack(SimId),
    HunterExhaustsIdentity,
    AuthoredContact(SimId),
    MissingGrazerPlacement(SimId),
    DuplicateGrazerPlacement(SimId),
    UnexpectedGrazerPlacement(SimId),
    MissingPatchPlacement(SimId),
    DuplicatePatchPlacement(SimId),
    UnexpectedPatchPlacement(SimId),
    OutOfBounds(SimId),
    SharedPatchCell(Cell),
    ZeroTravelCost(SimId),
    ZeroNewbornTravelCost,
}

impl fmt::Display for MobileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid mobile configuration: {self:?}")
    }
}
impl std::error::Error for MobileError {}
