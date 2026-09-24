//! Hunter adapter reuses the same bounded paid travel calculation as grazers.

use super::{HuntCounters, HuntEvent, HuntEventKind, HuntState, Hunter};
use crate::mobile::movement::plan_travel;
use crate::mobile::rest::RestState;
use crate::mobile::{MobileActor, Position};
use crate::model::{Body, Clock, Grazer, Identity};
use bevy_ecs::prelude::*;
use std::collections::BTreeSet;

type PreyFilter = (With<Grazer>, Without<Hunter>);
type MovingHunter<'a> = (
    Entity,
    &'a Identity,
    &'a mut Body,
    &'a mut Position,
    &'a mut MobileActor,
    &'a Hunter,
);

pub(crate) fn travel(
    clock: Res<Clock>,
    prey: Query<(&Identity, &Body), PreyFilter>,
    mut hunters: Query<MovingHunter<'_>, Without<Grazer>>,
    state: Option<ResMut<HuntState>>,
    rest: Option<Res<RestState>>,
) {
    let Some(mut state) = state else {
        return;
    };
    let available: BTreeSet<_> = prey
        .iter()
        .filter(|(_, body)| body.alive() && clock.tick >= body.first_eligible_tick)
        .map(|(id, _)| id.0)
        .collect();
    let mut order: Vec<_> = hunters
        .iter()
        .map(|(e, id, _, _, _, _)| (id.0, e))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, e) in order {
        let (_, _, mut body, mut position, mut actor, hunter) =
            hunters.get_mut(e).expect("listed hunter");
        if !body.alive()
            || clock.tick < body.first_eligible_tick
            || !actor.can_forage()
            || actor.last_travel_tick == Some(clock.tick)
        {
            continue;
        }
        let destination = hunter
            .target
            .filter(|t| t.observed_tick == clock.tick && available.contains(&t.prey))
            .map(|t| t.observed_cell);
        let plan = plan_travel(
            position.0,
            destination,
            &body,
            &actor,
            rest.as_ref().map(|s| s.policy),
        );
        if plan.cells > 0 {
            let planned = state.planned(HuntCounters {
                travel_cells: u64::from(plan.cells),
                travel_units: u64::from(plan.spent),
                effort_points: u64::from(plan.effort),
                ..Default::default()
            });
            state.commit(
                planned,
                HuntEvent {
                    run: clock.run,
                    tick: clock.tick,
                    kind: HuntEventKind::Travelled {
                        hunter: id,
                        from: position.0,
                        to: plan.next,
                        cells: plan.cells,
                        spent_units: plan.spent,
                        effort_points: plan.effort,
                    },
                },
            );
            body.reserve_units -= plan.spent;
            position.0 = plan.next;
            if let Some(fatigue) = &mut actor.rest {
                fatigue.fatigue_points += plan.effort;
            }
        }
        actor.last_travel_tick = Some(clock.tick);
    }
}
