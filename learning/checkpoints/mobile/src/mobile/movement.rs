//! Bounded x-then-y cardinal travel, charged only for accepted cells.

use super::{Cell, MobileActor, MobileEvent, MobileEventKind, MobileState, Position};
use crate::model::{Clock, Grazer, Identity, Life, Patch};
use bevy_ecs::prelude::*;
use std::collections::BTreeSet;

fn advance(from: Cell, destination: Cell, cells: u32) -> Cell {
    let x_steps = cells.min(from.x.abs_diff(destination.x));
    let y_steps = cells - x_steps;
    Cell {
        x: if from.x < destination.x {
            from.x + x_steps
        } else {
            from.x - x_steps
        },
        y: if from.y < destination.y {
            from.y + y_steps
        } else {
            from.y - y_steps
        },
    }
}

pub(crate) fn travel(
    clock: Res<Clock>,
    patches: Query<&Identity, With<Patch>>,
    mut grazers: Query<(
        Entity,
        &Identity,
        &mut Position,
        &mut Grazer,
        &mut MobileActor,
    )>,
    mut state: ResMut<MobileState>,
) {
    let available: BTreeSet<_> = patches.iter().map(|id| id.0).collect();
    let mut order: Vec<_> = grazers
        .iter()
        .map(|(entity, id, _, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (id, entity) in order {
        let (_, _, mut position, mut grazer, mut actor) =
            grazers.get_mut(entity).expect("listed actor exists");
        if grazer.life != Life::Alive
            || clock.tick < grazer.first_eligible_tick
            || actor.last_travel_tick == Some(clock.tick)
        {
            continue;
        }
        let target = actor.target.filter(|target| {
            target.observed_tick == clock.tick && available.contains(&target.patch)
        });
        let (next, cells, spent) = if let Some(target) = target {
            let cells = position
                .0
                .distance(target.observed_cell)
                .min(u64::from(actor.motion.max_cells_per_tick))
                .min(u64::from(
                    grazer.reserve_units / actor.motion.travel_units_per_cell,
                )) as u32;
            (
                advance(position.0, target.observed_cell, cells),
                cells,
                cells
                    .checked_mul(actor.motion.travel_units_per_cell)
                    .expect("affordable travel fits reserve units"),
            )
        } else {
            (position.0, 0, 0)
        };
        if cells > 0 {
            let ledger_cells = state
                .ledger
                .travel_cells
                .checked_add(u64::from(cells))
                .expect("travel-cell ledger exhausted");
            let ledger_units = state
                .ledger
                .travel_units
                .checked_add(u64::from(spent))
                .expect("travel-unit ledger exhausted");
            let total_cells = state
                .totals
                .travel_cells
                .checked_add(u64::from(cells))
                .expect("travel-cell total exhausted");
            let total_units = state
                .totals
                .travel_units
                .checked_add(u64::from(spent))
                .expect("travel-unit total exhausted");
            assert!(
                state.history.can_record(),
                "mobile event eviction counter exhausted"
            );
            state.history.record(MobileEvent {
                run: clock.run,
                tick: clock.tick,
                kind: MobileEventKind::Travelled {
                    grazer: id,
                    from: position.0,
                    to: next,
                    cells,
                    spent_units: spent,
                },
            });
            state.ledger.travel_cells = ledger_cells;
            state.ledger.travel_units = ledger_units;
            state.totals.travel_cells = total_cells;
            state.totals.travel_units = total_units;
            grazer.reserve_units -= spent;
            position.0 = next;
        }
        actor.last_travel_tick = Some(clock.tick);
    }
}
