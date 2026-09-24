//! External daylight input. Growth is bounded before adding to avoid overflow.

use bevy_ecs::prelude::*;

use crate::history::{EventKind, History};
use crate::model::{Clock, Daylight, Identity, Patch, TickLedger};

pub(crate) fn grow(
    clock: Res<Clock>,
    daylight: Res<Daylight>,
    mut patches: Query<(Entity, &Identity, &mut Patch)>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
) {
    if daylight.is_lit(clock.tick) != Some(true) {
        return;
    }
    let mut order: Vec<_> = patches
        .iter()
        .map(|(entity, id, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut patch) = patches.get_mut(entity).expect("listed patch still exists");
        let added = patch
            .growth_units_per_lit_tick
            .min(patch.capacity_units - patch.biomass_units);
        patch.biomass_units += added;
        ledger.added_units = ledger
            .added_units
            .checked_add(u64::from(added))
            .expect("growth ledger exhausted");
        if added > 0 {
            history.record(
                &clock,
                EventKind::Growth {
                    patch: id,
                    units: added,
                },
            );
        }
    }
}
