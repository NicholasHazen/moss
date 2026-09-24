//! Bounded owned outcomes with conservative whole-tick coverage.

use std::collections::VecDeque;

use bevy_ecs::prelude::*;

use crate::model::{Clock, SimId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    Growth {
        patch: SimId,
        units: u32,
    },
    Maintenance {
        grazer: SimId,
        units: u32,
    },
    Meal {
        grazer: SimId,
        patch: SimId,
        units: u32,
    },
    Starved {
        grazer: SimId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub run: u64,
    pub tick: u64,
    pub kind: EventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistorySnapshot {
    pub events: Vec<Event>,
    pub limit: usize,
    pub evicted_events: u64,
    /// Complete history is available only for ticks strictly above this bound.
    /// A partially retained tick is deliberately excluded.
    pub complete_after_tick: u64,
    pub collected_through_tick: u64,
}

impl HistorySnapshot {
    /// None means invalid, not-yet-collected, or incomplete history; Some(0)
    /// means a fully covered interval contains no starvation event.
    pub fn starvations_between(&self, first_tick: u64, last_tick: u64) -> Option<usize> {
        if first_tick == 0
            || first_tick > last_tick
            || first_tick <= self.complete_after_tick
            || last_tick > self.collected_through_tick
        {
            return None;
        }
        Some(
            self.events
                .iter()
                .filter(|event| {
                    (first_tick..=last_tick).contains(&event.tick)
                        && matches!(event.kind, EventKind::Starved { .. })
                })
                .count(),
        )
    }
}

#[derive(Resource)]
pub(crate) struct History {
    events: VecDeque<Event>,
    limit: usize,
    evicted_events: u64,
    complete_after_tick: u64,
    collected_through_tick: u64,
}

impl History {
    pub fn new(limit: usize) -> Self {
        Self {
            events: VecDeque::new(),
            limit,
            evicted_events: 0,
            complete_after_tick: 0,
            collected_through_tick: 0,
        }
    }

    pub fn record(&mut self, clock: &Clock, kind: EventKind) {
        self.events.push_back(Event {
            run: clock.run,
            tick: clock.tick,
            kind,
        });
        if self.events.len() > self.limit {
            let evicted = self.events.pop_front().expect("an event was just appended");
            self.complete_after_tick = self.complete_after_tick.max(evicted.tick);
            self.evicted_events = self
                .evicted_events
                .checked_add(1)
                .expect("event eviction counter exhausted");
        }
    }

    pub fn snapshot(&self) -> HistorySnapshot {
        HistorySnapshot {
            events: self.events.iter().copied().collect(),
            limit: self.limit,
            evicted_events: self.evicted_events,
            complete_after_tick: self.complete_after_tick,
            collected_through_tick: self.collected_through_tick,
        }
    }
}

pub(crate) fn complete_tick(clock: Res<Clock>, mut history: ResMut<History>) {
    history.collected_through_tick = clock.tick;
}
