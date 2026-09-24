use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Key { run: u32, id: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Life { Alive, Dead }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cause { Starved }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Terminal { sequence: u64, tick: u64, subject: Key, cause: Cause }
#[derive(Debug)]
struct Animal { key: Key, reserve_units: u32, life: Life, actions: u64 }
struct Journal {
    run: u32,
    limit: usize,
    events: VecDeque<Terminal>,
    evicted: u64,
    complete_from: u64,
    collected_through: u64,
    next_sequence: u64,
}
impl Journal {
    fn new(run: u32, limit: usize) -> Self {
        assert!(limit > 0, "this journal requires positive retention");
        Self { run, limit, events: VecDeque::new(), evicted: 0,
            complete_from: 1, collected_through: 0, next_sequence: 1 }
    }
    fn record(&mut self, tick: u64, subject: Key, cause: Cause) {
        assert_eq!(subject.run, self.run);
        assert_eq!(tick, self.collected_through.checked_add(1).unwrap());
        let sequence = self.next_sequence;
        self.next_sequence = sequence.checked_add(1).expect("event sequence exhausted");
        if self.events.len() == self.limit {
            let removed = self.events.pop_front().unwrap();
            self.evicted = self.evicted.checked_add(1).expect("eviction count exhausted");
            self.complete_from = self.complete_from.max(
                removed.tick.checked_add(1).expect("coverage tick exhausted"));
        }
        self.events.push_back(Terminal { sequence, tick, subject, cause });
    }
    fn finish_tick(&mut self, tick: u64) {
        assert_eq!(tick, self.collected_through.checked_add(1).unwrap());
        self.collected_through = tick;
    }
    fn terminal_count(&self, subject: Key, from: u64, through: u64) -> Option<usize> {
        if subject.run != self.run || from > through || from < self.complete_from
            || through > self.collected_through { return None; }
        Some(self.events.iter().filter(|event| event.subject == subject
            && event.tick >= from && event.tick <= through).count())
    }
}

fn resolve_terminal(animal: &mut Animal, tick: u64, journal: &mut Journal) -> bool {
    if animal.life != Life::Alive || animal.reserve_units != 0 { return false; }
    animal.life = Life::Dead;
    journal.record(tick, animal.key, Cause::Starved);
    true
}

fn step(animals: &mut [Animal], journal: &mut Journal) {
    let tick = journal.collected_through.checked_add(1).expect("tick exhausted");
    for animal in animals.iter_mut().filter(|animal| animal.life == Life::Alive) {
        animal.reserve_units = animal.reserve_units.saturating_sub(1);
    }
    for animal in animals.iter_mut() { resolve_terminal(animal, tick, journal); }
    for animal in animals.iter_mut().filter(|animal| animal.life == Life::Alive) {
        animal.actions = animal.actions.checked_add(1).expect("action count exhausted");
    }
    journal.finish_tick(tick);
}
fn animal(run: u32, id: u32, reserve_units: u32) -> Animal {
    Animal { key: Key { run, id }, reserve_units, life: Life::Alive, actions: 0 }
}
fn main() {
    let mut journal = Journal::new(7, 2);
    let mut animals = vec![animal(7, 9, 1)];
    step(&mut animals, &mut journal);
    println!("life={:?} actions={} terminal={:?}", animals[0].life,
        animals[0].actions, journal.terminal_count(Key { run: 7, id: 9 }, 1, 1));
    animals.retain(|animal| animal.life == Life::Alive);
    println!("live={} retained={}", animals.len(), journal.events.len());
    step(&mut [animal(7, 10, 1)], &mut journal);
    step(&mut [animal(7, 11, 1)], &mut journal);
    println!("retained={} evicted={} complete={}..={}", journal.events.len(),
        journal.evicted, journal.complete_from, journal.collected_through);
    println!("old={:?} later={:?}", journal.terminal_count(Key { run: 7, id: 9 }, 1, 1),
        journal.terminal_count(Key { run: 7, id: 9 }, 2, 3));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_transition_happens_once_before_later_action() {
        let mut journal = Journal::new(7, 4);
        let mut animals = [animal(7, 9, 1)];
        step(&mut animals, &mut journal);
        step(&mut animals, &mut journal);
        assert_eq!((animals[0].life, animals[0].reserve_units, animals[0].actions),
            (Life::Dead, 0, 0));
        assert_eq!(journal.events.iter().copied().collect::<Vec<_>>(), vec![
            Terminal { sequence: 1, tick: 1, subject: Key { run: 7, id: 9 }, cause: Cause::Starved }]);
    }
    #[test]
    fn surviving_animal_acts_and_known_absence_is_zero() {
        let mut journal = Journal::new(7, 2);
        let mut animals = [animal(7, 9, 3)];
        step(&mut animals, &mut journal);
        assert_eq!((animals[0].reserve_units, animals[0].actions), (2, 1));
        assert_eq!(journal.terminal_count(animals[0].key, 1, 1), Some(0));
    }
    #[test]
    fn repeated_resolution_in_one_tick_does_not_duplicate_the_event() {
        let mut journal = Journal::new(7, 2);
        let mut subject = animal(7, 9, 0);
        assert!(resolve_terminal(&mut subject, 1, &mut journal));
        assert!(!resolve_terminal(&mut subject, 1, &mut journal));
        assert_eq!(journal.events.len(), 1);
    }
    #[test]
    fn retained_identity_survives_removal_and_differs_across_runs() {
        let mut journal = Journal::new(7, 2);
        let mut animals = vec![animal(7, 9, 1)];
        step(&mut animals, &mut journal);
        animals.retain(|animal| animal.life == Life::Alive);
        assert!(animals.is_empty());
        assert_eq!(journal.events[0].subject, Key { run: 7, id: 9 });
        let next_run = animal(8, 9, 4);
        assert_ne!(journal.events[0].subject, next_run.key);
        assert_eq!(journal.terminal_count(next_run.key, 1, 1), None);
    }
    #[test]
    fn eviction_preserves_order_and_exposes_lost_coverage() {
        let mut journal = Journal::new(7, 2);
        for id in [9, 10, 11] { step(&mut [animal(7, id, 1)], &mut journal); }
        assert_eq!(journal.events.iter().map(|e| (e.sequence, e.subject.id)).collect::<Vec<_>>(),
            vec![(2, 10), (3, 11)]);
        assert_eq!((journal.evicted, journal.complete_from, journal.collected_through), (1, 2, 3));
        assert_eq!(journal.terminal_count(Key { run: 7, id: 9 }, 1, 3), None);
        assert_eq!(journal.terminal_count(Key { run: 7, id: 9 }, 2, 3), Some(0));
    }
    #[test]
    fn eviction_within_one_tick_marks_the_entire_tick_incomplete() {
        let mut journal = Journal::new(7, 1);
        step(&mut [animal(7, 9, 1), animal(7, 10, 1)], &mut journal);
        assert_eq!(journal.events.len(), 1);
        assert_eq!(journal.events[0].subject.id, 10);
        assert_eq!(journal.complete_from, 2);
        assert_eq!(journal.terminal_count(Key { run: 7, id: 10 }, 1, 1), None);
        step(&mut [], &mut journal);
        assert_eq!(journal.terminal_count(Key { run: 7, id: 10 }, 2, 2), Some(0));
    }
    #[test]
    fn uncollected_and_invalid_ranges_are_unknown() {
        let mut journal = Journal::new(7, 2);
        let key = Key { run: 7, id: 9 };
        assert_eq!(journal.terminal_count(key, 1, 1), None);
        step(&mut [], &mut journal);
        assert_eq!(journal.terminal_count(key, 1, 1), Some(0));
        assert_eq!(journal.terminal_count(key, 1, 2), None);
        assert_eq!(journal.terminal_count(key, 2, 1), None);
    }
    #[test]
    fn forgetting_a_record_does_not_revive_terminal_eligibility() {
        let mut journal = Journal::new(7, 1);
        let mut original = animal(7, 9, 0);
        assert!(resolve_terminal(&mut original, 1, &mut journal));
        journal.finish_tick(1);
        step(&mut [animal(7, 10, 1)], &mut journal);
        assert_eq!(journal.events[0].subject.id, 10);
        assert!(!resolve_terminal(&mut original, 3, &mut journal));
        assert_eq!((journal.events.len(), journal.evicted), (1, 1));
    }

}
