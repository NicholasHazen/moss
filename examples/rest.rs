const START_REST_AT: u8 = 6;
const RESUME_AT: u8 = 2;
const MIN_REST_TICKS: u8 = 2;
const MAX_FATIGUE: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SimId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Activity {
    Foraging { target: Option<SimId> },
    Resting { ticks_left: u8 },
}

impl Activity {
    fn target(self) -> Option<SimId> {
        match self {
            Self::Foraging { target } => target,
            Self::Resting { .. } => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Foraging { .. } => "forage",
            Self::Resting { .. } => "rest",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Policy {
    Reactive,
    Committed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Creature {
    reserve_units: u32,
    fatigue_points: u8,
    activity: Activity,
}

fn decide(current: Activity, fatigue: u8, target_valid: bool, policy: Policy) -> Activity {
    if matches!(policy, Policy::Committed) {
        if let Activity::Resting { ticks_left } = current {
            if ticks_left > 0 {
                return current;
            }
            if fatigue > RESUME_AT {
                return Activity::Resting { ticks_left: 1 };
            }
            return Activity::Foraging { target: None };
        }
    }
    if fatigue >= START_REST_AT {
        let ticks_left = match policy {
            Policy::Reactive => 1,
            Policy::Committed => MIN_REST_TICKS,
        };
        Activity::Resting { ticks_left }
    } else {
        Activity::Foraging {
            target: current.target().filter(|_| target_valid),
        }
    }
}

fn step(creature: &mut Creature, policy: Policy, target_valid: bool, recovery: u8) {
    creature.reserve_units = creature.reserve_units.saturating_sub(1);
    creature.activity = decide(
        creature.activity,
        creature.fatigue_points,
        target_valid,
        policy,
    );
    match creature.activity {
        Activity::Resting { ticks_left } => {
            creature.fatigue_points = creature.fatigue_points.saturating_sub(recovery);
            creature.activity = Activity::Resting {
                ticks_left: ticks_left.saturating_sub(1),
            };
        }
        Activity::Foraging { .. } => {
            creature.fatigue_points = creature.fatigue_points.saturating_add(2).min(MAX_FATIGUE);
        }
    }
}

fn fixture() -> Creature {
    Creature {
        reserve_units: 12,
        fatigue_points: 6,
        activity: Activity::Foraging {
            target: Some(SimId(7)),
        },
    }
}

fn main() {
    for policy in [Policy::Reactive, Policy::Committed] {
        let mut creature = fixture();
        println!("{policy:?}");
        for tick in 1..=4 {
            step(&mut creature, policy, true, 2);
            println!(
                "tick={tick} {} reserve={} fatigue={} target={:?}",
                creature.activity.label(),
                creature.reserve_units,
                creature.fatigue_points,
                creature.activity.target()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_recovers_fatigue_while_maintenance_still_spends_energy() {
        let mut creature = fixture();
        step(&mut creature, Policy::Committed, true, 2);
        assert_eq!(
            creature,
            Creature {
                reserve_units: 11,
                fatigue_points: 4,
                activity: Activity::Resting { ticks_left: 1 }
            }
        );
    }

    #[test]
    fn commitment_changes_the_same_reactive_trace() {
        let trace = |policy| {
            let mut creature = fixture();
            (0..4)
                .map(|_| {
                    step(&mut creature, policy, true, 2);
                    (creature.activity.label(), creature.fatigue_points)
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            trace(Policy::Reactive),
            [("rest", 4), ("forage", 6), ("rest", 4), ("forage", 6)]
        );
        assert_eq!(
            trace(Policy::Committed),
            [("rest", 4), ("rest", 2), ("forage", 4), ("forage", 6)]
        );
    }

    #[test]
    fn minimum_commitment_survives_early_recovery() {
        let mut creature = fixture();
        step(&mut creature, Policy::Committed, true, 10);
        assert_eq!(creature.fatigue_points, 0);
        step(&mut creature, Policy::Committed, true, 10);
        assert_eq!(creature.activity, Activity::Resting { ticks_left: 0 });
        assert_eq!(creature.reserve_units, 10);
        step(&mut creature, Policy::Committed, true, 10);
        assert_eq!(creature.activity, Activity::Foraging { target: None });
    }

    #[test]
    fn completed_commitment_waits_for_the_recovery_threshold() {
        let mut creature = Creature {
            fatigue_points: 10,
            ..fixture()
        };
        for _ in 0..3 {
            step(&mut creature, Policy::Committed, true, 2);
        }
        assert_eq!(creature.fatigue_points, 4);
        assert_eq!(creature.activity, Activity::Resting { ticks_left: 0 });
        step(&mut creature, Policy::Committed, true, 2);
        assert_eq!(creature.fatigue_points, 2);
        step(&mut creature, Policy::Committed, true, 2);
        assert_eq!(creature.activity, Activity::Foraging { target: None });
    }

    #[test]
    fn resting_cancels_a_target_and_waking_does_not_restore_it() {
        let mut creature = fixture();
        assert_eq!(creature.activity.target(), Some(SimId(7)));
        for _ in 0..3 {
            step(&mut creature, Policy::Committed, true, 2);
        }
        assert_eq!(creature.activity, Activity::Foraging { target: None });
    }

    #[test]
    fn an_invalid_target_is_cleared_even_without_an_activity_switch() {
        let mut creature = Creature {
            fatigue_points: 0,
            ..fixture()
        };
        let mut valid = creature;
        step(&mut valid, Policy::Committed, true, 2);
        assert_eq!(valid.activity.target(), Some(SimId(7)));
        step(&mut creature, Policy::Committed, false, 2);
        assert_eq!(creature.activity, Activity::Foraging { target: None });
        assert_eq!(creature.reserve_units, 11);
    }

    #[test]
    fn rest_never_refills_an_empty_energy_store() {
        let mut creature = Creature {
            reserve_units: 0,
            ..fixture()
        };
        step(&mut creature, Policy::Committed, true, 2);
        assert_eq!(creature.reserve_units, 0);
        assert_eq!(creature.fatigue_points, 4);
    }

    #[test]
    fn reevaluating_a_decision_does_not_spend_a_committed_tick() {
        let creature = Creature {
            activity: Activity::Resting { ticks_left: 2 },
            ..fixture()
        };
        for _ in 0..100 {
            assert_eq!(
                decide(creature.activity, 0, false, Policy::Committed),
                creature.activity
            );
        }
        assert_eq!(creature.reserve_units, 12);
    }
}
