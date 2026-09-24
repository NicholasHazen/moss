//! Decisions select an activity; only execute spends commitment and recovers.

use super::{Activity, RestActor, RestCounters, RestEvent, RestEventKind, RestPolicy, RestState};
use crate::mobile::MobileActor;
use crate::model::{Body, Clock, Grazer, Identity};
use bevy_ecs::prelude::*;

// Selecting an activity changes neither fatigue nor remaining commitment.
pub(crate) fn next_activity(rest: RestActor, policy: RestPolicy, tick: u64) -> Activity {
    match rest.activity {
        Activity::Foraging if rest.fatigue_points >= policy.enter_at_points => Activity::Resting {
            remaining_ticks: policy.minimum_rest_ticks,
        },
        Activity::Resting { remaining_ticks: 0 }
            if rest.fatigue_points <= policy.exit_at_points
                && rest.last_rest_tick != Some(tick) =>
        {
            Activity::Foraging
        }
        current => current,
    }
}

pub(crate) fn rest_effect(rest: RestActor, policy: RestPolicy) -> (u32, u32, u32) {
    let Activity::Resting { remaining_ticks } = rest.activity else {
        panic!("rest effect requires resting activity");
    };
    let recovered = rest.fatigue_points.min(policy.recovery_points_per_tick);
    (
        recovered,
        rest.fatigue_points - recovered,
        remaining_ticks.saturating_sub(1),
    )
}

pub(crate) fn decide(
    clock: Res<Clock>,
    mut actors: Query<(Entity, &Identity, &Body, &mut MobileActor), With<Grazer>>,
    state: Option<ResMut<RestState>>,
) {
    let Some(mut state) = state else {
        return;
    };
    let mut order: Vec<_> = actors
        .iter()
        .map(|(entity, id, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, grazer, mut actor) = actors.get_mut(entity).expect("listed actor exists");
        if !grazer.alive() || clock.tick < grazer.first_eligible_tick {
            continue;
        }
        let rest = actor.rest.as_mut().expect("rest enabled for actor");
        if rest.last_transition_tick == Some(clock.tick) {
            continue;
        }
        let next = next_activity(*rest, state.policy, clock.tick);
        if next == rest.activity {
            continue;
        }
        let (kind, delta) = match next {
            Activity::Resting { remaining_ticks } => (
                RestEventKind::EnteredRest {
                    grazer: id,
                    fatigue_points: rest.fatigue_points,
                    committed_ticks: remaining_ticks,
                },
                RestCounters {
                    entered_rest: 1,
                    ..Default::default()
                },
            ),
            Activity::Foraging => (
                RestEventKind::Woke {
                    grazer: id,
                    fatigue_points: rest.fatigue_points,
                },
                RestCounters {
                    woke: 1,
                    ..Default::default()
                },
            ),
        };
        let counters = state.planned(delta);
        state.commit(
            counters,
            RestEvent {
                run: clock.run,
                tick: clock.tick,
                kind,
            },
        );
        rest.activity = next;
        rest.last_transition_tick = Some(clock.tick);
    }
}

pub(crate) fn execute(
    clock: Res<Clock>,
    mut actors: Query<(Entity, &Identity, &Body, &mut MobileActor), With<Grazer>>,
    state: Option<ResMut<RestState>>,
) {
    let Some(mut state) = state else {
        return;
    };
    let mut order: Vec<_> = actors
        .iter()
        .map(|(entity, id, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, grazer, mut actor) = actors.get_mut(entity).expect("listed actor exists");
        if !grazer.alive() || clock.tick < grazer.first_eligible_tick {
            continue;
        }
        let rest = actor.rest.as_mut().expect("rest enabled for actor");
        let Activity::Resting { .. } = rest.activity else {
            continue;
        };
        if rest.last_rest_tick == Some(clock.tick) {
            continue;
        }
        let (recovered, fatigue_after, remaining_ticks) = rest_effect(*rest, state.policy);
        let counters = state.planned(RestCounters {
            recovered_points: u64::from(recovered),
            rest_actions: 1,
            ..Default::default()
        });
        state.commit(
            counters,
            RestEvent {
                run: clock.run,
                tick: clock.tick,
                kind: RestEventKind::Rested {
                    grazer: id,
                    recovered_points: recovered,
                    fatigue_after,
                    remaining_ticks,
                },
            },
        );
        rest.fatigue_points = fatigue_after;
        rest.activity = Activity::Resting { remaining_ticks };
        rest.last_rest_tick = Some(clock.tick);
    }
}
