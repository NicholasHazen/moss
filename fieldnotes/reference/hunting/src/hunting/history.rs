//! Hunter outcomes have their own bounded history and conservative tick coverage.

use crate::{Cell, SimId};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuntEventKind {
    Maintenance {
        hunter: SimId,
        units: u32,
    },
    TargetChanged {
        hunter: SimId,
        from: Option<SimId>,
        to: Option<SimId>,
    },
    Travelled {
        hunter: SimId,
        from: Cell,
        to: Cell,
        cells: u32,
        spent_units: u32,
        effort_points: u32,
    },
    Captured {
        hunter: SimId,
        prey: SimId,
        cell: Cell,
        transferred_units: u32,
        attack_units: u32,
        effort_points: u32,
    },
    HunterStarved {
        hunter: SimId,
    },
    EnteredRest {
        hunter: SimId,
        fatigue_points: u32,
        committed_ticks: u32,
    },
    Woke {
        hunter: SimId,
        fatigue_points: u32,
    },
    Rested {
        hunter: SimId,
        recovered_points: u32,
        fatigue_after: u32,
        remaining_ticks: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HuntEvent {
    pub run: u64,
    pub tick: u64,
    pub kind: HuntEventKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntHistorySnapshot {
    pub events: Vec<HuntEvent>,
    pub limit: usize,
    pub evicted_events: u64,
    pub complete_after_tick: u64,
    pub collected_through_tick: u64,
}
impl HuntHistorySnapshot {
    pub fn captures_between(&self, first: u64, last: u64) -> Option<u64> {
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
                .filter(|e| {
                    (first..=last).contains(&e.tick)
                        && matches!(e.kind, HuntEventKind::Captured { .. })
                })
                .count() as u64,
        )
    }
}
pub(crate) struct HuntHistory {
    events: VecDeque<HuntEvent>,
    limit: usize,
    evicted_events: u64,
    complete_after_tick: u64,
    pub collected_through_tick: u64,
}
impl HuntHistory {
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
    pub fn record(&mut self, event: HuntEvent) {
        assert!(self.can_record(), "hunt event eviction counter exhausted");
        self.events.push_back(event);
        if self.events.len() > self.limit {
            let old = self.events.pop_front().expect("just appended");
            self.complete_after_tick = self.complete_after_tick.max(old.tick);
            self.evicted_events += 1;
        }
    }
    pub fn snapshot(&self) -> HuntHistorySnapshot {
        HuntHistorySnapshot {
            events: self.events.iter().copied().collect(),
            limit: self.limit,
            evicted_events: self.evicted_events,
            complete_after_tick: self.complete_after_tick,
            collected_through_tick: self.collected_through_tick,
        }
    }
    #[cfg(test)]
    pub fn exhaust_for_test(&mut self) {
        self.events.clear();
        self.limit = 0;
        self.evicted_events = u64::MAX;
    }
}
