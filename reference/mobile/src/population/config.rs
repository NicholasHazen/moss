//! Opt-in population inputs; ordinary first-arc scenarios stay unchanged.

use std::collections::BTreeMap;
use std::fmt;

use crate::{Scenario, ScenarioError, SimId};

pub const POPULATION_VERSION: &str = "moss-course-population-v1";
pub const INHERITANCE_ALGORITHM: &str = "upkeep-copy-cycle-v1";

/// A toy inherited trait whose value is the actual upkeep units per eligible tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UpkeepTrait(u8);

impl UpkeepTrait {
    pub fn new(value: u8) -> Result<Self, PopulationError> {
        if !(1..=3).contains(&value) {
            return Err(PopulationError::InvalidTrait(value));
        }
        Ok(Self(value))
    }
    pub fn value(self) -> u8 {
        self.0
    }
    pub fn units(self) -> u32 {
        u32::from(self.0)
    }

    pub(crate) fn mutate(self, delta: i8) -> (Self, i8) {
        let changed = (i16::from(self.0) + i16::from(delta)).clamp(1, 3) as u8;
        (Self(changed), changed as i8 - self.0 as i8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FounderTrait {
    pub id: SimId,
    pub upkeep: UpkeepTrait,
}

/// Raw authored configuration, validated atomically by the population constructor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopulationConfig {
    pub founders: Vec<FounderTrait>,
    pub maturation_ticks: u64,
    pub cooldown_ticks: u64,
    pub contribution_units_per_parent: u32,
    pub birth_cost_units_per_parent: u32,
    pub child_capacity_units: u32,
    pub child_meal_units_per_tick: u32,
    pub max_living: usize,
    pub mutation_cycle: Vec<i8>,
    pub history_limit: usize,
}

impl PopulationConfig {
    /// Default authored policy: contribution 2, cost 1, delay 2, cooldown 3.
    /// Founders must agree with the supplied scenario's upkeep rates.
    pub fn new(founders: Vec<FounderTrait>) -> Self {
        Self {
            founders,
            maturation_ticks: 2,
            cooldown_ticks: 3,
            contribution_units_per_parent: 2,
            birth_cost_units_per_parent: 1,
            child_capacity_units: 8,
            child_meal_units_per_tick: 2,
            max_living: 8,
            mutation_cycle: vec![0],
            history_limit: 128,
        }
    }

    pub(crate) fn validate(&self, scenario: &Scenario) -> Result<SimId, PopulationError> {
        scenario.validate().map_err(PopulationError::Scenario)?;
        if self.maturation_ticks == 0 {
            return Err(PopulationError::InvalidMaturation);
        }
        if self.cooldown_ticks == 0 {
            return Err(PopulationError::InvalidCooldown);
        }
        if self.contribution_units_per_parent == 0 {
            return Err(PopulationError::InvalidContribution);
        }
        let child_reserve = self
            .contribution_units_per_parent
            .checked_mul(2)
            .ok_or(PopulationError::PaymentOverflow)?;
        self.contribution_units_per_parent
            .checked_add(self.birth_cost_units_per_parent)
            .and_then(|debit| debit.checked_add(1))
            .ok_or(PopulationError::PaymentOverflow)?;
        if self.child_capacity_units < child_reserve {
            return Err(PopulationError::ChildCapacityTooSmall);
        }
        if self.max_living == 0 {
            return Err(PopulationError::InvalidLivingLimit);
        }
        if scenario.grazers.len() > self.max_living {
            return Err(PopulationError::OverLivingLimit);
        }
        if self.mutation_cycle.is_empty()
            || self
                .mutation_cycle
                .iter()
                .any(|delta| !(-1..=1).contains(delta))
        {
            return Err(PopulationError::InvalidMutationCycle);
        }
        let seeds: BTreeMap<_, _> = scenario
            .grazers
            .iter()
            .map(|grazer| (grazer.id, grazer))
            .collect();
        let mut traits = BTreeMap::new();
        for founder in &self.founders {
            if traits.insert(founder.id, founder.upkeep).is_some() {
                return Err(PopulationError::DuplicateFounder(founder.id));
            }
            let seed = seeds
                .get(&founder.id)
                .ok_or(PopulationError::UnexpectedFounder(founder.id))?;
            if seed.maintenance_units_per_tick != founder.upkeep.units() {
                return Err(PopulationError::UpkeepMismatch(founder.id));
            }
        }
        for id in seeds.keys() {
            if !traits.contains_key(id) {
                return Err(PopulationError::MissingFounder(*id));
            }
        }
        let highest = scenario
            .grazers
            .iter()
            .map(|grazer| grazer.id.0)
            .chain(scenario.patches.iter().map(|patch| patch.id.0))
            .max()
            .unwrap_or(0);
        highest
            .checked_add(1)
            .map(SimId)
            .ok_or(PopulationError::NoChildIdentity)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopulationError {
    Scenario(ScenarioError),
    InvalidTrait(u8),
    InvalidMaturation,
    InvalidCooldown,
    InvalidContribution,
    PaymentOverflow,
    ChildCapacityTooSmall,
    InvalidLivingLimit,
    OverLivingLimit,
    InvalidMutationCycle,
    DuplicateFounder(SimId),
    UnexpectedFounder(SimId),
    MissingFounder(SimId),
    UpkeepMismatch(SimId),
    NoChildIdentity,
}

impl fmt::Display for PopulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid population configuration: {self:?}")
    }
}
impl std::error::Error for PopulationError {}
