//! Preflight two-parent births, then commit costs and reserve deferred child slots.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::prelude::*;

use crate::SimId;
use crate::model::{Clock, Grazer, Identity, Life, Patch};
use crate::population::{
    BirthOrigin, BirthRecord, Individual, PopulationEvent, PopulationEventKind, PopulationLedger,
    PopulationState, Stage,
};
use crate::simulation::Roster;

#[derive(Clone, Copy)]
struct ParentView {
    id: SimId,
    reserve_units: u32,
    life: Life,
    first_eligible_tick: u64,
    site: Option<SimId>,
    individual: Individual,
}

impl ParentView {
    fn new(id: &Identity, grazer: &Grazer, individual: &Individual) -> Self {
        Self {
            id: id.0,
            reserve_units: grazer.reserve_units,
            life: grazer.life,
            first_eligible_tick: grazer.first_eligible_tick,
            site: grazer.feeding_site,
            individual: *individual,
        }
    }

    fn eligible(self, tick: u64, debit: u32) -> bool {
        self.life == Life::Alive
            && self.individual.stage == Stage::Adult
            && tick >= self.first_eligible_tick
            && tick >= self.individual.next_birth_tick
            && self.reserve_units > debit
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Rejection {
    Ineligible,
    Capacity,
    Exhausted,
}

struct BirthPlan {
    reserve_after: [u32; 2],
    next_parent_birth_tick: u64,
    child: Grazer,
    individual: Individual,
    record: BirthRecord,
    next_child_id: Option<SimId>,
    next_total_births: u64,
    next_ledger: PopulationLedger,
}

fn plan_birth(
    clock: &Clock,
    state: &PopulationState,
    mut parents: [ParentView; 2],
    sites: &BTreeSet<SimId>,
    available_slots: usize,
) -> Result<BirthPlan, Rejection> {
    let config = &state.config;
    let debit = config
        .contribution_units_per_parent
        .checked_add(config.birth_cost_units_per_parent)
        .ok_or(Rejection::Exhausted)?;
    parents.sort_by_key(|parent| parent.id);
    if parents[0].id == parents[1].id
        || parents
            .iter()
            .any(|parent| !parent.eligible(clock.tick, debit))
    {
        return Err(Rejection::Ineligible);
    }
    let site = parents[0].site.ok_or(Rejection::Ineligible)?;
    if parents[1].site != Some(site) || !sites.contains(&site) {
        return Err(Rejection::Ineligible);
    }
    if available_slots == 0 {
        return Err(Rejection::Capacity);
    }
    let child_id = state.next_child_id.ok_or(Rejection::Exhausted)?;
    let first_eligible_tick = clock.tick.checked_add(1).ok_or(Rejection::Exhausted)?;
    let matures_at_tick = clock
        .tick
        .checked_add(config.maturation_ticks)
        .ok_or(Rejection::Exhausted)?;
    let next_parent_birth_tick = clock
        .tick
        .checked_add(config.cooldown_ticks)
        .ok_or(Rejection::Exhausted)?;
    let ordinal = state.totals.births;
    let next_total_births = ordinal.checked_add(1).ok_or(Rejection::Exhausted)?;
    let donor = parents[(ordinal % 2) as usize];
    let proposed_delta =
        config.mutation_cycle[(ordinal % config.mutation_cycle.len() as u64) as usize];
    let (child_trait, applied_delta) = donor.individual.upkeep.mutate(proposed_delta);
    let reserve_units = config
        .contribution_units_per_parent
        .checked_mul(2)
        .ok_or(Rejection::Exhausted)?;
    let origin = BirthOrigin {
        run: clock.run,
        birth_tick: clock.tick,
        birth_ordinal: ordinal,
        parents: [parents[0].id, parents[1].id],
        donor: donor.id,
        inherited: donor.individual.upkeep,
        proposed_delta,
        applied_delta,
        child_trait,
    };
    let record = BirthRecord {
        child: child_id,
        origin,
        contribution_units_per_parent: config.contribution_units_per_parent,
        cost_units_per_parent: config.birth_cost_units_per_parent,
        initial_reserve_units: reserve_units,
        first_eligible_tick,
        matures_at_tick,
    };
    let next_ledger = PopulationLedger {
        births: state
            .ledger
            .births
            .checked_add(1)
            .ok_or(Rejection::Exhausted)?,
        transferred_units: state
            .ledger
            .transferred_units
            .checked_add(u64::from(reserve_units))
            .ok_or(Rejection::Exhausted)?,
        dissipated_units: state
            .ledger
            .dissipated_units
            .checked_add(u64::from(config.birth_cost_units_per_parent) * 2)
            .ok_or(Rejection::Exhausted)?,
        capacity_blocked_pairs: state.ledger.capacity_blocked_pairs,
    };
    if !state.history.can_record() {
        return Err(Rejection::Exhausted);
    }
    Ok(BirthPlan {
        reserve_after: [
            parents[0].reserve_units - debit,
            parents[1].reserve_units - debit,
        ],
        next_parent_birth_tick,
        child: Grazer {
            reserve_units,
            capacity_units: config.child_capacity_units,
            maintenance_units_per_tick: child_trait.units(),
            meal_units_per_tick: config.child_meal_units_per_tick,
            feeding_site: Some(site),
            life: Life::Alive,
            first_eligible_tick,
        },
        individual: Individual {
            stage: Stage::Juvenile,
            matures_at_tick: Some(matures_at_tick),
            next_birth_tick: matures_at_tick,
            upkeep: child_trait,
            origin: Some(origin),
        },
        record,
        next_child_id: child_id.0.checked_add(1).map(SimId),
        next_total_births,
        next_ledger,
    })
}

pub(crate) fn resolve_births(
    clock: Res<Clock>,
    mut commands: Commands,
    mut parents: Query<(Entity, &Identity, &mut Grazer, &mut Individual)>,
    patches: Query<&Identity, With<Patch>>,
    mut roster: ResMut<Roster>,
    mut state: ResMut<PopulationState>,
) {
    let sites: BTreeSet<_> = patches.iter().map(|id| id.0).collect();
    let living = parents
        .iter()
        .filter(|(_, _, grazer, _)| grazer.life == Life::Alive)
        .count();
    let mut available_slots = state
        .config
        .max_living
        .checked_sub(living)
        .expect("population already exceeds its configured cap");
    let debit =
        state.config.contribution_units_per_parent + state.config.birth_cost_units_per_parent;
    let mut groups: BTreeMap<SimId, Vec<(SimId, Entity)>> = BTreeMap::new();
    for (entity, id, grazer, individual) in parents.iter() {
        let view = ParentView::new(id, grazer, individual);
        if view.eligible(clock.tick, debit)
            && let Some(site) = view.site.filter(|site| sites.contains(site))
        {
            groups.entry(site).or_default().push((id.0, entity));
        }
    }
    for group in groups.values_mut() {
        group.sort_by_key(|(id, _)| *id);
        for pair in group.chunks_exact(2) {
            let entities = [pair[0].1, pair[1].1];
            let views = entities.map(|entity| {
                let (_, id, grazer, individual) =
                    parents.get(entity).expect("candidate parent exists");
                ParentView::new(id, grazer, individual)
            });
            let plan = match plan_birth(&clock, &state, views, &sites, available_slots) {
                Ok(plan) => plan,
                Err(Rejection::Ineligible) => continue,
                Err(Rejection::Capacity) => {
                    state.ledger.capacity_blocked_pairs = state
                        .ledger
                        .capacity_blocked_pairs
                        .checked_add(1)
                        .expect("capacity-blocked pair ledger exhausted");
                    continue;
                }
                Err(Rejection::Exhausted) => {
                    panic!("population birth counter or deadline exhausted before debit")
                }
            };
            {
                let [
                    (_, _, mut first, mut first_stage),
                    (_, _, mut second, mut second_stage),
                ] = parents
                    .get_many_mut(entities)
                    .expect("distinct candidate parents still exist");
                first.reserve_units = plan.reserve_after[0];
                second.reserve_units = plan.reserve_after[1];
                first_stage.next_birth_tick = plan.next_parent_birth_tick;
                second_stage.next_birth_tick = plan.next_parent_birth_tick;
            }
            available_slots -= 1;
            state.next_child_id = plan.next_child_id;
            state.totals.births = plan.next_total_births;
            state.ledger = plan.next_ledger;
            state.history.record(PopulationEvent {
                run: clock.run,
                tick: clock.tick,
                kind: PopulationEventKind::Born(plan.record),
            });
            let entity = commands
                .spawn((Identity(plan.record.child), plan.child, plan.individual))
                .id();
            roster.grazers.push(entity);
        }
    }
}
