//! Actual target changes, travel, and newborn placement with bounded coverage.

use std::collections::VecDeque;

use super::Cell;
use crate::SimId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobileEventKind {
    TargetChanged {
        grazer: SimId,
        from: Option<SimId>,
        to: Option<SimId>,
    },
    Travelled {
        grazer: SimId,
        from: Cell,
        to: Cell,
        cells: u32,
        spent_units: u32,
    },
    BornAt {
        child: SimId,
        cell: Cell,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobileEvent {
    pub run: u64,
    pub tick: u64,
    pub kind: MobileEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileHistorySnapshot {
    pub events: Vec<MobileEvent>,
    pub limit: usize,
    pub evicted_events: u64,
    pub complete_after_tick: u64,
    pub collected_through_tick: u64,
}

impl MobileHistorySnapshot {
    /// None means invalid, future, or incomplete coverage; Some(0) means no travel.
    pub fn travelled_cells_between(&self, first: u64, last: u64) -> Option<u64> {
        if first == 0
            || first > last
            || first <= self.complete_after_tick
            || last > self.collected_through_tick
        {
            return None;
        }
        self.events
            .iter()
            .filter(|event| (first..=last).contains(&event.tick))
            .try_fold(0_u64, |sum, event| {
                let cells = match event.kind {
                    MobileEventKind::Travelled { cells, .. } => cells,
                    _ => 0,
                };
                sum.checked_add(u64::from(cells))
            })
    }
}

pub(crate) struct MobileHistory {
    events: VecDeque<MobileEvent>,
    limit: usize,
    evicted_events: u64,
    complete_after_tick: u64,
    pub collected_through_tick: u64,
}

impl MobileHistory {
    #[cfg(test)]
    pub(crate) fn exhaust_eviction_counter_for_test(&mut self) {
        self.events.clear();
        self.limit = 0;
        self.evicted_events = u64::MAX;
    }

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
    pub fn record(&mut self, event: MobileEvent) {
        assert!(self.can_record(), "mobile event eviction counter exhausted");
        self.events.push_back(event);
        if self.events.len() > self.limit {
            let evicted = self.events.pop_front().expect("an event was just appended");
            self.complete_after_tick = self.complete_after_tick.max(evicted.tick);
            self.evicted_events += 1;
        }
    }
    pub fn snapshot(&self) -> MobileHistorySnapshot {
        MobileHistorySnapshot {
            events: self.events.iter().copied().collect(),
            limit: self.limit,
            evicted_events: self.evicted_events,
            complete_after_tick: self.complete_after_tick,
            collected_through_tick: self.collected_through_tick,
        }
    }
}
