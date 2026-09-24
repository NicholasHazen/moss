//! Bounded rest evidence has its own coverage; it never changes mobile-A events.

use crate::SimId;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestEventKind {
    EnteredRest {
        grazer: SimId,
        fatigue_points: u32,
        committed_ticks: u32,
    },
    Woke {
        grazer: SimId,
        fatigue_points: u32,
    },
    TravelEffort {
        grazer: SimId,
        cells: u32,
        points: u32,
    },
    Rested {
        grazer: SimId,
        recovered_points: u32,
        fatigue_after: u32,
        remaining_ticks: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestEvent {
    pub run: u64,
    pub tick: u64,
    pub kind: RestEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestHistorySnapshot {
    pub events: Vec<RestEvent>,
    pub limit: usize,
    pub evicted_events: u64,
    pub complete_after_tick: u64,
    pub collected_through_tick: u64,
}

impl RestHistorySnapshot {
    pub fn rest_actions_between(&self, first: u64, last: u64) -> Option<u64> {
        if first == 0
            || first > last
            || first <= self.complete_after_tick
            || last > self.collected_through_tick
        {
            return None;
        }
        Some(
            self.events
                .iter()
                .filter(|event| {
                    (first..=last).contains(&event.tick)
                        && matches!(event.kind, RestEventKind::Rested { .. })
                })
                .count() as u64,
        )
    }
}

pub(crate) struct RestHistory {
    events: VecDeque<RestEvent>,
    limit: usize,
    evicted_events: u64,
    complete_after_tick: u64,
    pub collected_through_tick: u64,
}
impl RestHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            events: VecDeque::new(),
            limit,
            evicted_events: 0,
            complete_after_tick: 0,
            collected_through_tick: 0,
        }
    }
    pub fn can_record(&self) -> bool {
        self.events.len() < self.limit || self.evicted_events.checked_add(1).is_some()
    }
    pub fn record(&mut self, event: RestEvent) {
        assert!(self.can_record(), "rest event eviction counter exhausted");
        self.events.push_back(event);
        if self.events.len() > self.limit {
            let evicted = self.events.pop_front().expect("event just appended");
            self.complete_after_tick = self.complete_after_tick.max(evicted.tick);
            self.evicted_events += 1;
        }
    }
    pub fn snapshot(&self) -> RestHistorySnapshot {
        RestHistorySnapshot {
            events: self.events.iter().copied().collect(),
            limit: self.limit,
            evicted_events: self.evicted_events,
            complete_after_tick: self.complete_after_tick,
            collected_through_tick: self.collected_through_tick,
        }
    }
}
