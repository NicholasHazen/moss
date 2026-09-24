//! Population events supplement, rather than change, the first-arc event schema.

use std::collections::VecDeque;

use super::{BirthOrigin, UpkeepTrait};
use crate::SimId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BirthRecord {
    pub child: SimId,
    pub origin: BirthOrigin,
    pub contribution_units_per_parent: u32,
    pub cost_units_per_parent: u32,
    pub initial_reserve_units: u32,
    pub first_eligible_tick: u64,
    pub matures_at_tick: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopulationEventKind {
    Born(BirthRecord),
    Matured {
        individual: SimId,
        upkeep: UpkeepTrait,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopulationEvent {
    pub run: u64,
    pub tick: u64,
    pub kind: PopulationEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopulationHistorySnapshot {
    pub events: Vec<PopulationEvent>,
    pub limit: usize,
    pub evicted_events: u64,
    pub complete_after_tick: u64,
    pub collected_through_tick: u64,
}

impl PopulationHistorySnapshot {
    /// None is incomplete, invalid, or future coverage; Some(0) is known absence.
    pub fn births_between(&self, first_tick: u64, last_tick: u64) -> Option<usize> {
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
                        && matches!(event.kind, PopulationEventKind::Born(_))
                })
                .count(),
        )
    }
}

pub(crate) struct PopulationHistory {
    events: VecDeque<PopulationEvent>,
    limit: usize,
    evicted_events: u64,
    complete_after_tick: u64,
    pub collected_through_tick: u64,
}

impl PopulationHistory {
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

    pub fn record(&mut self, event: PopulationEvent) {
        assert!(
            self.can_record(),
            "population event eviction counter exhausted"
        );
        self.events.push_back(event);
        if self.events.len() > self.limit {
            let evicted = self
                .events
                .pop_front()
                .expect("population event was just appended");
            self.complete_after_tick = self.complete_after_tick.max(evicted.tick);
            self.evicted_events += 1;
        }
    }

    pub fn snapshot(&self) -> PopulationHistorySnapshot {
        PopulationHistorySnapshot {
            events: self.events.iter().copied().collect(),
            limit: self.limit,
            evicted_events: self.evicted_events,
            complete_after_tick: self.complete_after_tick,
            collected_through_tick: self.collected_through_tick,
        }
    }
}
