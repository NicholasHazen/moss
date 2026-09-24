//! Whole-transfer capture preflights both bodies, terminal state, and evidence.

use super::{HuntCounters, HuntEvent, HuntEventKind, HuntState, Hunter};
use crate::mobile::rest::RestState;
use crate::mobile::{MobileActor, Position};
use crate::model::{Body, Clock, Grazer, Identity, Terminal};
use bevy_ecs::prelude::*;
use std::collections::BTreeMap;

type Bodies<'a> = (
    Entity,
    &'a Identity,
    &'a mut Body,
    &'a Position,
    Option<&'a Grazer>,
    Option<&'a mut Hunter>,
    &'a mut MobileActor,
);

pub(crate) fn capture(
    clock: Res<Clock>,
    mut bodies: Query<Bodies<'_>>,
    state: Option<ResMut<HuntState>>,
    rest: Option<Res<RestState>>,
) {
    let Some(mut state) = state else {
        return;
    };
    let identities: BTreeMap<_, _> = bodies
        .iter()
        .map(|(e, id, _, _, _, _, _)| (id.0, e))
        .collect();
    let order: Vec<_> = identities
        .iter()
        .filter(|(_, e)| bodies.get(**e).expect("listed body").5.is_some())
        .map(|(id, e)| (*id, *e))
        .collect();
    for (hunter_id, hunter_entity) in order {
        let (_, _, body, position, _, hunter, actor) =
            bodies.get(hunter_entity).expect("listed hunter");
        let hunter = hunter.expect("hunter role");
        if !body.alive()
            || clock.tick < body.first_eligible_tick
            || !actor.can_forage()
            || hunter.last_capture_tick == Some(clock.tick)
        {
            continue;
        }
        let Some(target) = hunter.target.filter(|t| t.observed_tick == clock.tick) else {
            continue;
        };
        let Some(&prey_entity) = identities
            .get(&target.prey)
            .filter(|&&e| e != hunter_entity)
        else {
            continue;
        };
        let (_, _, prey, prey_position, diet, prey_hunter, _) =
            bodies.get(prey_entity).expect("listed prey");
        if diet.is_none()
            || prey_hunter.is_some()
            || !prey.alive()
            || clock.tick < prey.first_eligible_tick
            || prey.reserve_units == 0
            || position.0 != prey_position.0
        {
            continue;
        }
        let Some(after_cost) = body.reserve_units.checked_sub(hunter.attack_units) else {
            continue;
        };
        let transfer = prey.reserve_units;
        let Some(after_gain) = after_cost
            .checked_add(transfer)
            .filter(|&gain| gain <= body.capacity_units)
        else {
            continue;
        };
        let effort = if let Some(policy) = &rest {
            let fatigue = actor.rest.expect("rest enabled").fatigue_points;
            if hunter.attack_effort_points > policy.policy.maximum_fatigue_points - fatigue {
                continue;
            }
            hunter.attack_effort_points
        } else {
            0
        };
        let attack = hunter.attack_units;
        let cell = position.0;
        let planned = state.planned(HuntCounters {
            captures: 1,
            transferred_units: u64::from(transfer),
            attack_units: u64::from(attack),
            effort_points: u64::from(effort),
            ..Default::default()
        });
        let [
            (_, _, mut hunter_body, _, _, hunter, mut actor),
            (_, _, mut prey_body, _, _, _, _),
        ] = bodies
            .get_many_mut([hunter_entity, prey_entity])
            .expect("distinct current bodies");
        // No fallible biological result remains after this preflight.
        state.commit(
            planned,
            HuntEvent {
                run: clock.run,
                tick: clock.tick,
                kind: HuntEventKind::Captured {
                    hunter: hunter_id,
                    prey: target.prey,
                    cell,
                    transferred_units: transfer,
                    attack_units: attack,
                    effort_points: effort,
                },
            },
        );
        hunter_body.reserve_units = after_gain;
        prey_body.reserve_units = 0;
        prey_body.terminal = Terminal::Captured { hunter: hunter_id };
        hunter.expect("hunter role").last_capture_tick = Some(clock.tick);
        if let Some(fatigue) = &mut actor.rest {
            fatigue.fatigue_points += effort;
        }
    }
}
