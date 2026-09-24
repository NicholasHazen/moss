//! Local contact eligibility and stable-ID contention for immediately depleted food.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

use crate::history::{EventKind, History};
use crate::mobile::{MobileActor, Position};
use crate::model::{Body, Clock, Grazer, Identity, Patch, TickLedger};

type FeedingAccess<'a> = (
    Entity,
    &'a Identity,
    &'a mut Body,
    &'a Grazer,
    Option<&'a Position>,
    Option<&'a MobileActor>,
);

pub(crate) fn feed(
    clock: Res<Clock>,
    mut grazers: Query<FeedingAccess<'_>>,
    mut patches: Query<(Entity, &Identity, &mut Patch, Option<&Position>)>,
    mut ledger: ResMut<TickLedger>,
    mut history: ResMut<History>,
) {
    let sites: BTreeMap<_, _> = patches
        .iter()
        .map(|(entity, id, _, _)| (id.0, entity))
        .collect();
    let mut order: Vec<_> = grazers
        .iter()
        .map(|(entity, id, _, _, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut grazer, diet, position, actor) =
            grazers.get_mut(entity).expect("listed grazer still exists");
        if !grazer.alive() || clock.tick < grazer.first_eligible_tick {
            continue;
        }
        let Some(site) = diet.feeding_site else {
            continue;
        };
        let Some(patch_entity) = sites.get(&site) else {
            continue;
        };
        let (_, _, mut patch, patch_position) = patches
            .get_mut(*patch_entity)
            .expect("listed patch still exists");
        if let Some(actor) = actor {
            let current_target = actor
                .target
                .is_some_and(|target| target.patch == site && target.observed_tick == clock.tick);
            if !actor.can_forage()
                || !current_target
                || position.is_none()
                || position != patch_position
            {
                continue;
            }
        }
        let eaten = diet
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
