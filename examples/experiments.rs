use std::collections::VecDeque;
const ALGORITHM: &str = "moss-cohort-lcg64-v1";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spec { seed: u64, maintenance_units: u32, ticks: u32, history_limit: usize }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Animal { id: u32, reserve: u32, alive: bool, actions: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Grant { tick: u32, id: u32, units: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Death { tick: u32, id: u32 }
#[derive(Clone, Debug)]
struct Run {
    spec: Spec,
    animals: Vec<Animal>,
    pending: Vec<Grant>,
    deaths: VecDeque<Death>,
    evicted: usize,
    complete_from: u32,
    collected_through: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Mean { sum: u64, count: usize }
#[derive(Clone, Debug, PartialEq, Eq)]
struct Report {
    spec: Spec,
    algorithm: &'static str,
    outcomes: Vec<Animal>,
    pending: Vec<Grant>,
    initial: usize,
    survivors: usize,
    sorted_reserves: Vec<u32>,
    cohort_mean: Option<Mean>,
    survivor_mean: Option<Mean>,
    complete_death_count: Option<usize>,
    retained_deaths: Vec<Death>,
    complete_from: u32,
    collected_through: u32,
    evicted: usize,
}
fn mean(values: impl Iterator<Item = u32>) -> Option<Mean> {
    let (sum, count) = values.fold((0_u64, 0_usize), |(sum, count), value|
        (sum.checked_add(u64::from(value)).expect("sum exhausted"),
         count.checked_add(1).expect("count exhausted")));
    (count > 0).then_some(Mean { sum, count })
}
fn initial(spec: Spec) -> Run {
    assert!(spec.history_limit > 0);
    let mut state = spec.seed;
    let animals = (1..=4).map(|id| {
        state = state.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        Animal { id, reserve: 3 + ((state >> 32) as u32 % 6), alive: true, actions: 0 }
    }).collect();
    Run { spec, animals, pending: vec![], deaths: VecDeque::new(), evicted: 0,
        complete_from: 1, collected_through: 0 }
}
fn advance(run: &mut Run) {
    let tick = run.collected_through.checked_add(1).expect("tick exhausted");
    assert!(tick <= run.spec.ticks);
    for grant in run.pending.iter().filter(|grant| grant.tick == tick) {
        if let Some(animal) = run.animals.iter_mut().find(|animal| animal.id == grant.id && animal.alive) {
            animal.reserve = animal.reserve.checked_add(grant.units).expect("grant exceeds units");
        }
    }
    run.pending.retain(|grant| grant.tick > tick);
    for animal in run.animals.iter_mut().filter(|animal| animal.alive) {
        animal.reserve = animal.reserve.saturating_sub(run.spec.maintenance_units);
        if animal.reserve == 0 {
            animal.alive = false;
            if run.deaths.len() == run.spec.history_limit {
                let removed = run.deaths.pop_front().unwrap();
                run.complete_from = run.complete_from.max(removed.tick.checked_add(1).unwrap());
                run.evicted = run.evicted.checked_add(1).unwrap();
            }
            run.deaths.push_back(Death { tick, id: animal.id });
        } else {
            animal.actions = animal.actions.checked_add(1).unwrap();
        }
    }
    run.collected_through = tick;
}
fn execute(spec: Spec) -> Run {
    let mut run = initial(spec);
    for _ in 0..spec.ticks { advance(&mut run); }
    run
}
fn summarize(run: &Run) -> Report {
    let (_, outcomes, pending) = canonical(run);
    let mut sorted_reserves: Vec<_> = run.animals.iter().map(|animal| animal.reserve).collect();
    sorted_reserves.sort_unstable();
    let full_coverage = run.complete_from == 1 && run.collected_through == run.spec.ticks
        && run.collected_through > 0;
    Report { spec: run.spec, algorithm: ALGORITHM, outcomes, pending, initial: run.animals.len(), survivors: run.animals.iter().filter(|animal| animal.alive).count(),
        sorted_reserves, cohort_mean: mean(run.animals.iter().map(|animal| animal.reserve)),
        survivor_mean: mean(run.animals.iter().filter(|animal| animal.alive).map(|animal| animal.reserve)),
        complete_death_count: full_coverage.then_some(run.deaths.len()),
        retained_deaths: run.deaths.iter().copied().collect(), complete_from: run.complete_from,
        collected_through: run.collected_through, evicted: run.evicted }
}
fn canonical(run: &Run) -> (u32, Vec<Animal>, Vec<Grant>) {
    let mut animals = run.animals.clone(); animals.sort_by_key(|animal| animal.id);
    let mut pending = run.pending.clone(); pending.sort();
    (run.collected_through, animals, pending)
}
fn spec(seed: u64, maintenance_units: u32, history_limit: usize) -> Spec {
    Spec { seed, maintenance_units, ticks: 3, history_limit }
}
fn main() {
    println!("algorithm={ALGORITHM} ticks=3 seeds=1,2,3,4");
    for seed in 1..=4 {
        let baseline = summarize(&execute(spec(seed, 1, 2)));
        let treatment = summarize(&execute(spec(seed, 2, 2)));
        println!("seed={seed} alive={}/{} -> {}/{} delta={}", baseline.survivors,
            baseline.initial, treatment.survivors, treatment.initial,
            treatment.survivors as i64 - baseline.survivors as i64);
    }
    let run = execute(spec(1, 2, 2));
    let report = summarize(&run);
    println!("seed=1 treatment reserves={:?} cohort={:?} survivors={:?}",
        report.sorted_reserves, report.cohort_mean, report.survivor_mean);
    println!("journal death count={:?} retained={} evicted={} complete={}..={}",
        report.complete_death_count, report.retained_deaths.len(), report.evicted,
        report.complete_from, report.collected_through);
    println!("pending after horizon={}", canonical(&run).2.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reporting_keeps_dead_members_in_the_initial_cohort_denominator() {
        let run = execute(spec(1, 2, 8));
        let report = summarize(&run);
        assert_eq!((report.initial, report.survivors), (4, 1));
        assert_eq!(report.sorted_reserves, vec![0, 0, 0, 1]);
        assert_eq!(report.cohort_mean, Some(Mean { sum: 1, count: 4 }));
        assert_eq!(report.survivor_mean, Some(Mean { sum: 1, count: 1 }));
        assert_eq!(report.complete_death_count, Some(3));
    }
    #[test]
    fn matched_cost_intervention_starts_from_identical_animals_for_every_seed() {
        for seed in 1..=4 {
            assert_eq!(canonical(&initial(spec(seed, 1, 8))), canonical(&initial(spec(seed, 2, 8))));
        }
        let differences: Vec<_> = (1..=4).map(|seed| {
            let a = summarize(&execute(spec(seed, 1, 8)));
            let b = summarize(&execute(spec(seed, 2, 8)));
            b.survivors as i32 - a.survivors as i32
        }).collect();
        assert_eq!(differences, vec![-3, -2, -2, -2]);
    }
    #[test]
    fn report_owns_values_and_retained_records_after_the_run_changes() {
        let mut run = execute(spec(1, 2, 8));
        let report = summarize(&run);
        run.animals.clear(); run.deaths.clear();
        assert_eq!(report.initial, 4);
        assert_eq!(report.sorted_reserves, vec![0, 0, 0, 1]);
        assert_eq!(report.retained_deaths.len(), 3);
    }
    #[test]
    fn eviction_makes_complete_death_count_unknown_without_changing_biology() {
        let short = execute(spec(1, 2, 1));
        let long = execute(spec(1, 2, 8));
        assert_eq!(canonical(&short), canonical(&long));
        let report = summarize(&short);
        assert_eq!(report.complete_death_count, None);
        assert_eq!((report.retained_deaths.len(), report.evicted), (1, 2));
        assert_eq!(summarize(&long).complete_death_count, Some(3));
    }
    #[test]
    fn fully_observed_zero_deaths_differs_from_no_collection() {
        let observed = summarize(&execute(spec(1, 0, 2)));
        assert_eq!(observed.complete_death_count, Some(0));
        assert_eq!(observed.survivors, 4);
        assert_eq!(summarize(&initial(spec(1, 0, 2))).complete_death_count, None);
    }
    #[test]
    fn no_survivors_has_no_survivor_mean_and_does_not_erase_the_cohort() {
        let report = summarize(&execute(spec(1, 20, 8)));
        assert_eq!((report.initial, report.survivors), (4, 0));
        assert_eq!(report.survivor_mean, None);
        assert_eq!(report.cohort_mean, Some(Mean { sum: 0, count: 4 }));
        assert_eq!(report.complete_death_count, Some(4));
    }
    #[test]
    fn pending_interventions_are_part_of_state_and_change_future_outcomes() {
        let mut plain = initial(spec(1, 20, 8));
        let mut granted = plain.clone();
        granted.pending.push(Grant { tick: 1, id: 1, units: 30 });
        assert_ne!(canonical(&plain), canonical(&granted));
        advance(&mut plain); advance(&mut granted);
        assert_eq!(summarize(&plain).survivors, 0);
        assert_eq!(summarize(&granted).survivors, 1);
        assert!(granted.pending.is_empty());
    }
    #[test]
    fn a_mean_cannot_replace_the_distribution() {
        let sparse = [0, 0, 0, 8];
        let even = [2, 2, 2, 2];
        assert_eq!(mean(sparse.into_iter()), mean(even.into_iter()));
        assert_ne!(sparse, even);
    }
    #[test]
    fn completed_tick_distinguishes_equal_rosters_with_different_future_eligibility() {
        let mut early = initial(spec(1, 20, 8));
        advance(&mut early);
        let finished = execute(spec(1, 20, 8));
        assert_eq!(early.spec, finished.spec);
        assert_eq!(early.animals, finished.animals);
        assert_eq!(early.pending, finished.pending);
        assert_eq!((early.collected_through, finished.collected_through), (1, 3));
        assert_ne!(canonical(&early), canonical(&finished));
        advance(&mut early);
        assert_eq!(early.collected_through, 2);
    }

}
