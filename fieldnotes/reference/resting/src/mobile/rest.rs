//! Optional fatigue: a separate store, committed actions, and owned evidence.

mod history;
mod systems;
pub use history::{RestEvent, RestEventKind, RestHistorySnapshot};
pub(crate) use systems::{decide, execute};

use super::MobileActor;
use crate::SimId;
use crate::model::{Clock, Identity};
use crate::simulation::Roster;
use bevy_ecs::prelude::*;
use history::RestHistory;

pub const REST_VERSION: &str = "moss-course-rest-b-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestPolicy {
    pub maximum_fatigue_points: u32,
    pub effort_points_per_cell: u32,
    pub enter_at_points: u32,
    pub exit_at_points: u32,
    pub minimum_rest_ticks: u32,
    pub recovery_points_per_tick: u32,
    pub history_limit: usize,
}

impl Default for RestPolicy {
    fn default() -> Self {
        Self {
            maximum_fatigue_points: 10,
            effort_points_per_cell: 2,
            enter_at_points: 6,
            exit_at_points: 2,
            minimum_rest_ticks: 2,
            recovery_points_per_tick: 2,
            history_limit: 128,
        }
    }
}

impl RestPolicy {
    pub(super) fn valid(self) -> bool {
        self.exit_at_points < self.enter_at_points
            && self.enter_at_points <= self.maximum_fatigue_points
            && self.effort_points_per_cell > 0
            && self.effort_points_per_cell <= self.maximum_fatigue_points
            && self.minimum_rest_ticks > 0
            && self.recovery_points_per_tick > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Foraging,
    Resting { remaining_ticks: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RestActor {
    pub fatigue_points: u32,
    pub activity: Activity,
    pub last_rest_tick: Option<u64>,
    pub last_transition_tick: Option<u64>,
}

impl Default for RestActor {
    fn default() -> Self {
        Self {
            fatigue_points: 0,
            activity: Activity::Foraging,
            last_rest_tick: None,
            last_transition_tick: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RestCounters {
    pub effort_points: u64,
    pub recovered_points: u64,
    pub rest_actions: u64,
    pub entered_rest: u64,
    pub woke: u64,
}

impl RestCounters {
    fn adding(self, delta: Self) -> Self {
        Self {
            effort_points: self
                .effort_points
                .checked_add(delta.effort_points)
                .expect("effort counter exhausted"),
            recovered_points: self
                .recovered_points
                .checked_add(delta.recovered_points)
                .expect("recovery counter exhausted"),
            rest_actions: self
                .rest_actions
                .checked_add(delta.rest_actions)
                .expect("rest-action counter exhausted"),
            entered_rest: self
                .entered_rest
                .checked_add(delta.entered_rest)
                .expect("rest-entry counter exhausted"),
            woke: self
                .woke
                .checked_add(delta.woke)
                .expect("wake counter exhausted"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestGrazerReading {
    pub id: SimId,
    pub fatigue_points: u32,
    pub activity: Activity,
    pub last_rest_tick: Option<u64>,
    pub last_transition_tick: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestSnapshot {
    pub version: &'static str,
    pub run: u64,
    pub tick: u64,
    pub policy: RestPolicy,
    pub grazers: Vec<RestGrazerReading>,
    pub ledger: RestCounters,
    pub totals: RestCounters,
    pub history: RestHistorySnapshot,
}

#[derive(Resource)]
pub(crate) struct RestState {
    pub policy: RestPolicy,
    pub ledger: RestCounters,
    pub totals: RestCounters,
    pub history: RestHistory,
}

impl RestState {
    pub fn new(policy: RestPolicy) -> Self {
        Self {
            policy,
            ledger: RestCounters::default(),
            totals: RestCounters::default(),
            history: RestHistory::new(policy.history_limit),
        }
    }

    /// Preflight every fallible accounting operation before an actor changes.
    pub fn planned(&self, delta: RestCounters) -> (RestCounters, RestCounters) {
        let ledger = self.ledger.adding(delta);
        let totals = self.totals.adding(delta);
        assert!(
            self.history.can_record(),
            "rest event eviction counter exhausted"
        );
        (ledger, totals)
    }

    pub fn commit(&mut self, counters: (RestCounters, RestCounters), event: RestEvent) {
        self.history.record(event);
        self.ledger = counters.0;
        self.totals = counters.1;
    }
}

pub(crate) fn snapshot(world: &World) -> Option<RestSnapshot> {
    let state = world.get_resource::<RestState>()?;
    let clock = world.resource::<Clock>();
    let mut grazers: Vec<_> = world
        .resource::<Roster>()
        .grazers
        .iter()
        .map(|entity| {
            let id = world
                .get::<Identity>(*entity)
                .expect("rostered identity exists")
                .0;
            let rest = world
                .get::<MobileActor>(*entity)
                .expect("mobile actor exists")
                .rest
                .expect("rest enabled for every actor");
            RestGrazerReading {
                id,
                fatigue_points: rest.fatigue_points,
                activity: rest.activity,
                last_rest_tick: rest.last_rest_tick,
                last_transition_tick: rest.last_transition_tick,
            }
        })
        .collect();
    grazers.sort_by_key(|grazer| grazer.id);
    Some(RestSnapshot {
        version: REST_VERSION,
        run: clock.run,
        tick: clock.tick,
        policy: state.policy,
        grazers,
        ledger: state.ledger,
        totals: state.totals,
        history: state.history.snapshot(),
    })
}
