//! Every body pays upkeep; role adapters keep grazer and hunter evidence separate.

use crate::history::{EventKind, History};
use crate::hunting::{HuntCounters, HuntEvent, HuntEventKind, HuntState, Hunter};
use crate::model::{Body, Clock, Grazer, Identity, Terminal, TickLedger};
use bevy_ecs::prelude::*;

type BodyAccess<'a> = (
    Entity,
    &'a Identity,
    &'a mut Body,
    Option<&'a Grazer>,
    Option<&'a Hunter>,
);

pub(crate) fn maintain(
    clock: Res<Clock>,
    mut bodies: Query<BodyAccess<'_>>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
    mut hunts: Option<ResMut<HuntState>>,
) {
    let mut order: Vec<_> = bodies
        .iter()
        .map(|(entity, id, _, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut body, grazer, hunter) = bodies.get_mut(entity).expect("listed body exists");
        if !body.alive() || clock.tick < body.first_eligible_tick {
            continue;
        }
        let paid = body.reserve_units.min(body.maintenance_units_per_tick);
        if hunter.is_some() {
            if paid > 0 {
                let state = hunts.as_mut().expect("hunter policy exists");
                let planned = state.planned(HuntCounters {
                    maintenance_units: u64::from(paid),
                    ..Default::default()
                });
                state.commit(
                    planned,
                    HuntEvent {
                        run: clock.run,
                        tick: clock.tick,
                        kind: HuntEventKind::Maintenance {
                            hunter: id,
                            units: paid,
                        },
                    },
                );
            }
        } else if grazer.is_some() {
            ledger.maintenance_units = ledger
                .maintenance_units
                .checked_add(u64::from(paid))
                .expect("maintenance ledger exhausted");
            if paid > 0 {
                history.record(
                    &clock,
                    EventKind::Maintenance {
                        grazer: id,
                        units: paid,
                    },
                );
            }
        }
        body.reserve_units -= paid;
    }
}

pub(crate) fn starve(
    clock: Res<Clock>,
    mut bodies: Query<BodyAccess<'_>>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
    mut hunts: Option<ResMut<HuntState>>,
) {
    let mut order: Vec<_> = bodies
        .iter()
        .map(|(entity, id, _, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut body, grazer, hunter) = bodies.get_mut(entity).expect("listed body exists");
        if !body.alive() || body.reserve_units != 0 || clock.tick < body.first_eligible_tick {
            continue;
        }
        if hunter.is_some() {
            let state = hunts.as_mut().expect("hunter policy exists");
            let planned = state.planned(HuntCounters {
                hunter_starvations: 1,
                ..Default::default()
            });
            state.commit(
                planned,
                HuntEvent {
                    run: clock.run,
                    tick: clock.tick,
                    kind: HuntEventKind::HunterStarved { hunter: id },
                },
            );
        } else if grazer.is_some() {
            ledger.starvations = ledger
                .starvations
                .checked_add(1)
                .expect("starvation ledger exhausted");
            history.record(&clock, EventKind::Starved { grazer: id });
        }
        body.terminal = Terminal::Starved;
    }
}
