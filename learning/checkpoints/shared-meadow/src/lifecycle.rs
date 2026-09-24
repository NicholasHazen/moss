//! Upkeep before feeding, then permanent starvation after the chance to eat.

use bevy_ecs::prelude::*;

use crate::history::{EventKind, History};
use crate::model::{Clock, Grazer, Identity, Life, TickLedger};

pub(crate) fn maintain(
    clock: Res<Clock>,
    mut grazers: Query<(Entity, &Identity, &mut Grazer)>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
) {
    let mut order: Vec<_> = grazers
        .iter()
        .map(|(entity, id, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut grazer) = grazers.get_mut(entity).expect("listed grazer still exists");
        if grazer.life != Life::Alive {
            continue;
        }
        let paid = grazer.reserve_units.min(grazer.maintenance_units_per_tick);
        grazer.reserve_units -= paid;
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
}

pub(crate) fn starve(
    clock: Res<Clock>,
    mut grazers: Query<(Entity, &Identity, &mut Grazer)>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
) {
    let mut order: Vec<_> = grazers
        .iter()
        .map(|(entity, id, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut grazer) = grazers.get_mut(entity).expect("listed grazer still exists");
        if grazer.life == Life::Alive && grazer.reserve_units == 0 {
            grazer.life = Life::Dead;
            ledger.starvations = ledger
                .starvations
                .checked_add(1)
                .expect("starvation ledger exhausted");
            history.record(&clock, EventKind::Starved { grazer: id });
        }
    }
}
