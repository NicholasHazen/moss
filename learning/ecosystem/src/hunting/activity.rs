//! Hunter effects use the existing pure rest decisions and recovery calculation.

use super::{HuntCounters, HuntEvent, HuntEventKind, HuntState, Hunter};
use crate::Activity;
use crate::mobile::MobileActor;
use crate::mobile::rest::{
    RestState,
    systems::{next_activity, rest_effect},
};
use crate::model::{Body, Clock, Identity};
use bevy_ecs::prelude::*;

type Actors<'a> = (Entity, &'a Identity, &'a Body, &'a mut MobileActor);
pub(crate) fn decide(
    clock: Res<Clock>,
    mut actors: Query<Actors<'_>, With<Hunter>>,
    state: Option<ResMut<HuntState>>,
    rest: Option<Res<RestState>>,
) {
    let (Some(mut state), Some(policy)) = (state, rest) else {
        return;
    };
    let mut order: Vec<_> = actors.iter().map(|(e, id, _, _)| (id.0, e)).collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, e) in order {
        let (_, _, body, mut actor) = actors.get_mut(e).expect("listed hunter");
        if !body.alive() || clock.tick < body.first_eligible_tick {
            continue;
        }
        let rest = actor.rest.as_mut().expect("rest enabled");
        if rest.last_transition_tick == Some(clock.tick) {
            continue;
        }
        let next = next_activity(*rest, policy.policy, clock.tick);
        if next == rest.activity {
            continue;
        }
        let (kind, delta) = match next {
            Activity::Foraging => (
                HuntEventKind::Woke {
                    hunter: id,
                    fatigue_points: rest.fatigue_points,
                },
                HuntCounters {
                    woke: 1,
                    ..Default::default()
                },
            ),
            Activity::Resting { remaining_ticks } => (
                HuntEventKind::EnteredRest {
                    hunter: id,
                    fatigue_points: rest.fatigue_points,
                    committed_ticks: remaining_ticks,
                },
                HuntCounters {
                    entered_rest: 1,
                    ..Default::default()
                },
            ),
        };
        let planned = state.planned(delta);
        state.commit(
            planned,
            HuntEvent {
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
    mut actors: Query<Actors<'_>, With<Hunter>>,
    state: Option<ResMut<HuntState>>,
    rest: Option<Res<RestState>>,
) {
    let (Some(mut state), Some(policy)) = (state, rest) else {
        return;
    };
    let mut order: Vec<_> = actors.iter().map(|(e, id, _, _)| (id.0, e)).collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, e) in order {
        let (_, _, body, mut actor) = actors.get_mut(e).expect("listed hunter");
        if !body.alive() || clock.tick < body.first_eligible_tick {
            continue;
        }
        let rest = actor.rest.as_mut().expect("rest enabled");
        if !matches!(rest.activity, Activity::Resting { .. })
            || rest.last_rest_tick == Some(clock.tick)
        {
            continue;
        }
        let (recovered, fatigue_after, remaining_ticks) = rest_effect(*rest, policy.policy);
        let planned = state.planned(HuntCounters {
            rest_actions: 1,
            recovered_points: u64::from(recovered),
            ..Default::default()
        });
        state.commit(
            planned,
            HuntEvent {
                run: clock.run,
                tick: clock.tick,
                kind: HuntEventKind::Rested {
                    hunter: id,
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
