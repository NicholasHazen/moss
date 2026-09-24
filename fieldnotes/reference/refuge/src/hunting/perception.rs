//! Capture intentions use copied prey positions from before every actor travels.

use super::{
    HuntCounters, HuntEvent, HuntEventKind, HuntState, Hunter, PreyObservation, PreyTarget,
};
use crate::mobile::{MobileActor, Position};
use crate::model::{Body, Clock, Grazer, Identity};
use crate::refuge::RefugeMembership;
use bevy_ecs::prelude::*;

pub(crate) fn observe(
    clock: Res<Clock>,
    prey: Query<(&Identity, &Body, &Position, Option<&RefugeMembership>), With<Grazer>>,
    mut hunters: Query<(&Body, &Position, &MobileActor, &mut Hunter)>,
) {
    for (body, position, actor, mut hunter) in &mut hunters {
        if !body.alive() || clock.tick < body.first_eligible_tick || !actor.can_forage() {
            continue;
        }
        let mut visible: Vec<_> = prey
            .iter()
            .filter(|(_, body, cell, membership)| {
                body.alive()
                    && membership.is_none_or(|membership| membership.refuge.is_none())
                    && clock.tick >= body.first_eligible_tick
                    && body.reserve_units > 0
                    && position.0.distance(cell.0) <= u64::from(actor.motion.sensing_radius)
            })
            .map(|(id, body, cell, _)| PreyObservation {
                prey: id.0,
                cell: cell.0,
                reserve_units: body.reserve_units,
            })
            .collect();
        visible.sort_by_key(|p| p.prey);
        hunter.observed_tick = Some(clock.tick);
        hunter.observed_cell = Some(position.0);
        hunter.visible_prey = visible;
    }
}
pub(crate) fn choose(
    clock: Res<Clock>,
    mut hunters: Query<(
        Entity,
        &Identity,
        &Body,
        &Position,
        &MobileActor,
        &mut Hunter,
    )>,
    state: Option<ResMut<HuntState>>,
) {
    let Some(mut state) = state else {
        return;
    };
    let mut order: Vec<_> = hunters
        .iter()
        .map(|(e, id, _, _, _, _)| (id.0, e))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, e) in order {
        let (_, _, body, position, actor, mut hunter) = hunters.get_mut(e).expect("listed hunter");
        if !body.alive() || clock.tick < body.first_eligible_tick {
            continue;
        }
        let previous = hunter.target.map(|t| t.prey);
        let selected = if actor.can_forage() && hunter.observed_tick == Some(clock.tick) {
            hunter
                .visible_prey
                .iter()
                .find(|p| Some(p.prey) == previous)
                .or_else(|| {
                    hunter
                        .visible_prey
                        .iter()
                        .min_by_key(|p| (position.0.distance(p.cell), p.prey))
                })
                .copied()
        } else {
            None
        };
        let next = selected.map(|p| PreyTarget {
            prey: p.prey,
            observed_cell: p.cell,
            observed_tick: clock.tick,
        });
        if previous != next.map(|t| t.prey) {
            let planned = state.planned(HuntCounters {
                target_changes: 1,
                ..Default::default()
            });
            state.commit(
                planned,
                HuntEvent {
                    run: clock.run,
                    tick: clock.tick,
                    kind: HuntEventKind::TargetChanged {
                        hunter: id,
                        from: previous,
                        to: next.map(|t| t.prey),
                    },
                },
            );
        }
        hunter.target = next;
    }
}
