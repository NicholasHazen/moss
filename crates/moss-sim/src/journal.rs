//! Bounded, chronological records of actual simulation outcomes.

use std::collections::VecDeque;

use bevy_ecs::prelude::Resource;

use crate::SimId;

const JOURNAL_CAPACITY: usize = 32;

/// Actual initialization outcomes; authored placement is deliberately not a birth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    RunStarted { number: u64 },
    FixturePlaced { name: &'static str },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalEntry {
    pub tick: u64,
    pub participants: Vec<SimId>,
    pub kind: EventKind,
}

/// Recent events for the current run only. Reset discards the previous run's history.
#[derive(Resource, Default, Debug)]
pub struct Journal {
    entries: VecDeque<JournalEntry>,
    evicted: u64,
}

impl Journal {
    /// Retained entries in chronological order (oldest first).
    pub fn entries(&self) -> &VecDeque<JournalEntry> {
        &self.entries
    }

    pub fn capacity(&self) -> usize {
        JOURNAL_CAPACITY
    }

    /// Events dropped from the front in this run. A nonzero count means incomplete history.
    pub fn evicted(&self) -> u64 {
        self.evicted
    }

    pub fn oldest_tick(&self) -> Option<u64> {
        self.entries.front().map(|entry| entry.tick)
    }

    pub(crate) fn record(&mut self, entry: JournalEntry) {
        if self.entries.len() == JOURNAL_CAPACITY {
            self.entries.pop_front();
            self.evicted = self.evicted.checked_add(1).expect("journal count overflow");
        }
        self.entries.push_back(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_eviction_reports_incomplete_coverage() {
        let mut journal = Journal::default();
        assert_eq!(journal.oldest_tick(), None);
        for tick in 0..35 {
            journal.record(JournalEntry {
                tick,
                participants: vec![SimId(tick + 1)],
                kind: EventKind::FixturePlaced {
                    name: "test fixture",
                },
            });
        }
        assert_eq!(journal.entries().len(), journal.capacity());
        assert_eq!(journal.evicted(), 3);
        assert_eq!(journal.oldest_tick(), Some(3));
        assert_eq!(journal.entries().back().unwrap().tick, 34);
    }
}
