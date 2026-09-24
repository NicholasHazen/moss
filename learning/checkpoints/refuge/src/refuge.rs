//! Static protection changes prey eligibility, never movement or resource flows.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::prelude::*;

use crate::mobile::Position;
use crate::model::{Clock, Grazer, Identity};
use crate::simulation::Roster;
use crate::{Cell, GridBounds, MobileError, MobileScenario, SimId};

pub const REFUGE_VERSION: &str = "moss-course-refuge-v1";

/// Static-site IDs have their own namespace and never consume actor identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefugeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefugeSeed {
    pub id: RefugeId,
    pub cell: Cell,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefugeConfig {
    pub sites: Vec<RefugeSeed>,
}

impl RefugeConfig {
    /// An empty enabled configuration is valid; world construction validates it.
    pub fn new(sites: Vec<RefugeSeed>) -> Self {
        Self { sites }
    }

    pub(crate) fn validate(&self, bounds: GridBounds) -> Result<(), MobileError> {
        let mut ids = BTreeSet::new();
        let mut cells = BTreeSet::new();
        for site in &self.sites {
            if site.id.0 == 0 {
                return Err(MobileError::ZeroRefugeId);
            }
            if !ids.insert(site.id) {
                return Err(MobileError::DuplicateRefugeId(site.id));
            }
            if !bounds.contains(site.cell) {
                return Err(MobileError::RefugeOutOfBounds(site.id));
            }
            if !cells.insert(site.cell) {
                return Err(MobileError::SharedRefugeCell(site.cell));
            }
        }
        Ok(())
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RefugeMembership {
    pub refuge: Option<RefugeId>,
    pub assessed_tick: u64,
}

#[derive(Resource)]
pub(crate) struct RefugeState {
    config: RefugeConfig,
    sites_by_cell: BTreeMap<Cell, RefugeId>,
}

impl RefugeState {
    pub(crate) fn membership(&self, cell: Cell, tick: u64) -> RefugeMembership {
        RefugeMembership {
            refuge: self.sites_by_cell.get(&cell).copied(),
            assessed_tick: tick,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefugeGrazerReading {
    pub id: SimId,
    pub cell: Cell,
    pub refuge: Option<RefugeId>,
    pub assessed_tick: u64,
}

/// Current membership only: no claim about time spent inside or evicted events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefugeSnapshot {
    pub version: &'static str,
    pub run: u64,
    pub tick: u64,
    pub config: RefugeConfig,
    pub grazers: Vec<RefugeGrazerReading>,
}

pub(crate) fn install(world: &mut World, initial: &MobileScenario) {
    let Some(config) = initial.refuges() else {
        return;
    };
    let state = RefugeState {
        config: config.clone(),
        sites_by_cell: config.sites.iter().map(|s| (s.cell, s.id)).collect(),
    };
    let entities = world.resource::<Roster>().grazers.clone();
    for entity in entities {
        let cell = world
            .get::<Position>(entity)
            .expect("mobile grazer position")
            .0;
        world.entity_mut(entity).insert(state.membership(cell, 0));
    }
    world.insert_resource(state);
}

/// Run before local prey observation and again after all travel, before capture.
/// Decisions and snapshots do not execute this system or confer protection.
pub(crate) fn refresh_memberships(
    clock: Res<Clock>,
    state: Option<Res<RefugeState>>,
    mut grazers: Query<(&Position, &mut RefugeMembership), With<Grazer>>,
) {
    let Some(state) = state else {
        return;
    };
    for (position, mut membership) in &mut grazers {
        *membership = state.membership(position.0, clock.tick);
    }
}

pub(crate) fn snapshot(world: &World) -> Option<RefugeSnapshot> {
    let state = world.get_resource::<RefugeState>()?;
    let clock = world.resource::<Clock>();
    let mut grazers: Vec<_> = world
        .resource::<Roster>()
        .grazers
        .iter()
        .map(|&entity| {
            let membership = world
                .get::<RefugeMembership>(entity)
                .expect("refuge-mode membership");
            RefugeGrazerReading {
                id: world.get::<Identity>(entity).expect("grazer identity").0,
                cell: world.get::<Position>(entity).expect("grazer position").0,
                refuge: membership.refuge,
                assessed_tick: membership.assessed_tick,
            }
        })
        .collect();
    grazers.sort_by_key(|g| g.id);
    Some(RefugeSnapshot {
        version: REFUGE_VERSION,
        run: clock.run,
        tick: clock.tick,
        config: state.config.clone(),
        grazers,
    })
}
