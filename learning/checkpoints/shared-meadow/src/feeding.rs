//! Local contact eligibility and stable-ID contention for immediately depleted food.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

use crate::history::{EventKind, History};
use crate::model::{Clock, Grazer, Identity, Life, Patch, TickLedger};

pub(crate) fn feed(
    clock: Res<Clock>,
    mut grazers: Query<(Entity, &Identity, &mut Grazer)>,
    mut patches: Query<(Entity, &Identity, &mut Patch)>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
) {
    let sites: BTreeMap<_, _> = patches
        .iter()
        .map(|(entity, id, _)| (id.0, entity))
        .collect();
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
        let Some(site) = grazer.feeding_site else {
            continue;
        };
        let Some(patch_entity) = sites.get(&site) else {
            continue;
        };
        let (_, _, mut patch) = patches
            .get_mut(*patch_entity)
            .expect("listed patch still exists");
        let eaten = grazer
            .meal_units_per_tick
            .min(patch.biomass_units)
            .min(grazer.capacity_units - grazer.reserve_units);
        // Both authoritative stores change immediately, before the next claimant.
        patch.biomass_units -= eaten;
        grazer.reserve_units += eaten;
        ledger.eaten_units = ledger
            .eaten_units
            .checked_add(u64::from(eaten))
            .expect("meal ledger exhausted");
        if eaten > 0 {
            history.record(
                &clock,
                EventKind::Meal {
                    grazer: id,
                    patch: site,
                    units: eaten,
                },
            );
        }
    }
}
