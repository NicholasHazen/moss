//! Authored hunters are finite, globally identified, and never part of the birth cap.

use crate::{Cell, MobileError, MobileScenario, Motion, SimId};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HunterSeed {
    pub id: SimId,
    pub reserve_units: u32,
    pub capacity_units: u32,
    pub maintenance_units_per_tick: u32,
    pub attack_units: u32,
    pub attack_effort_points: u32,
    pub cell: Cell,
    pub motion: Motion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntConfig {
    pub hunters: Vec<HunterSeed>,
    pub history_limit: usize,
}
impl HuntConfig {
    pub fn new(hunters: Vec<HunterSeed>) -> Self {
        Self {
            hunters,
            history_limit: 128,
        }
    }
    pub fn meadow() -> Self {
        Self::new(
            [(100, 5), (101, 8)]
                .into_iter()
                .map(|(id, x)| HunterSeed {
                    id: SimId(id),
                    reserve_units: 16,
                    capacity_units: 24,
                    maintenance_units_per_tick: 1,
                    attack_units: 1,
                    attack_effort_points: 2,
                    cell: Cell { x, y: 2 },
                    motion: Motion {
                        sensing_radius: 5,
                        max_cells_per_tick: 1,
                        travel_units_per_cell: 1,
                    },
                })
                .collect(),
        )
    }
    pub(crate) fn validate(
        &self,
        initial: &MobileScenario,
        first_child: SimId,
    ) -> Result<SimId, MobileError> {
        let mut ids: BTreeSet<_> = initial
            .scenario()
            .grazers
            .iter()
            .map(|g| g.id)
            .chain(initial.scenario().patches.iter().map(|p| p.id))
            .collect();
        let mut highest = first_child.0 - 1;
        for hunter in &self.hunters {
            if !ids.insert(hunter.id) {
                return Err(MobileError::DuplicateHunterId(hunter.id));
            }
            if hunter.reserve_units > hunter.capacity_units {
                return Err(MobileError::HunterOverCapacity(hunter.id));
            }
            if !initial.space().bounds.contains(hunter.cell) {
                return Err(MobileError::OutOfBounds(hunter.id));
            }
            if hunter.motion.travel_units_per_cell == 0 {
                return Err(MobileError::ZeroTravelCost(hunter.id));
            }
            if hunter.attack_units == 0
                || hunter.attack_effort_points == 0
                || initial
                    .rest()
                    .is_some_and(|rest| hunter.attack_effort_points > rest.maximum_fatigue_points)
            {
                return Err(MobileError::InvalidAttack(hunter.id));
            }
            highest = highest.max(hunter.id.0);
        }
        highest
            .checked_add(1)
            .map(SimId)
            .ok_or(MobileError::HunterExhaustsIdentity)
    }
}
