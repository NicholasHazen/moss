//! Owned local readings precede every actor's movement; choice is not a reservation.

use super::{
    MobileActor, MobileEvent, MobileEventKind, MobileState, PatchObservation, PatchTarget, Position,
};
use crate::model::{Clock, Grazer, Identity, Life, Patch};
use bevy_ecs::prelude::*;

pub(crate) fn observe(
    clock: Res<Clock>,
    patches: Query<(&Identity, &Position, &Patch)>,
    mut grazers: Query<(&Position, &Grazer, &mut MobileActor)>,
) {
    for (position, grazer, mut actor) in &mut grazers {
        if grazer.life != Life::Alive || clock.tick < grazer.first_eligible_tick {
            continue;
        }
        let mut readings: Vec<_> = patches
            .iter()
            .filter(|(_, patch_position, _)| {
                position.0.distance(patch_position.0) <= u64::from(actor.motion.sensing_radius)
            })
            .map(|(id, position, patch)| PatchObservation {
                patch: id.0,
                cell: position.0,
                biomass_units: patch.biomass_units,
            })
            .collect();
        readings.sort_by_key(|reading| reading.patch);
        actor.visible_patches = readings;
        actor.observed_tick = Some(clock.tick);
        actor.observed_cell = Some(position.0);
    }
}

pub(crate) fn choose(
    clock: Res<Clock>,
    mut grazers: Query<(Entity, &Identity, &Position, &Grazer, &mut MobileActor)>,
    mut state: ResMut<MobileState>,
) {
    let mut order: Vec<_> = grazers
        .iter()
        .map(|(entity, id, _, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, position, grazer, mut actor) =
            grazers.get_mut(entity).expect("listed actor exists");
        if grazer.life != Life::Alive || clock.tick < grazer.first_eligible_tick {
            continue;
        }
        let previous = actor.target.map(|target| target.patch);
        let eligible = |reading: &&PatchObservation| {
            reading.biomass_units > 0
                && position.0.distance(reading.cell) <= u64::from(actor.motion.sensing_radius)
        };
        let selected = if actor.observed_tick == Some(clock.tick) {
            actor
                .visible_patches
                .iter()
                .filter(eligible)
                .find(|reading| Some(reading.patch) == previous)
                .or_else(|| {
                    actor
                        .visible_patches
                        .iter()
                        .filter(eligible)
                        .min_by_key(|reading| (position.0.distance(reading.cell), reading.patch))
                })
                .copied()
        } else {
            None
        };
        let next = selected.map(|reading| PatchTarget {
            patch: reading.patch,
            observed_cell: reading.cell,
            observed_tick: clock.tick,
        });
        if previous != next.map(|target| target.patch) {
            let ledger_changes = state
                .ledger
                .target_changes
                .checked_add(1)
                .expect("target-change ledger exhausted");
            let total_changes = state
                .totals
                .target_changes
                .checked_add(1)
                .expect("target-change total exhausted");
            assert!(
                state.history.can_record(),
                "mobile event eviction counter exhausted"
            );
            state.ledger.target_changes = ledger_changes;
            state.totals.target_changes = total_changes;
            state.history.record(MobileEvent {
                run: clock.run,
                tick: clock.tick,
                kind: MobileEventKind::TargetChanged {
                    grazer: id,
                    from: previous,
                    to: next.map(|target| target.patch),
                },
            });
        }
        actor.target = next;
    }
}
