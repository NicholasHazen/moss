//! Paired rules and the thin ECS adapter for today's movement exercise.
//!
//! **Maintenance is implemented and scheduled.** [`crate::install`] runs
//! [`spend_energy`] before the completed-tick counter. Movement is prepared but
//! unscheduled: Nick implements [`move_one_cell`] before its adapter is activated.
//! Choice and eating remain unimplemented and unscheduled.
//!
//! Implement one body, its focused regression in `tests/`, and its explicit
//! schedule ordering together. Never register all the stubs at once. Only the
//! later parameter lists grow with their exercises. The authored target gets us
//! to visible movement before autonomous choice. See `NOW.md` for the next edit.

use bevy_ecs::prelude::*;

use crate::{
    Creature, Energy, FoodPatch, FoodTarget, MovementRules, Position, SimId, Species,
    SpeciesEnergyRules, WorldConfig,
};

/// Exercise 1: spend the configured species cost per executed tick on each animal.
///
/// `Query` reads species and borrows mutable energy from entities carrying `Creature`.
/// The shared settings supply the cost; grass has no applicable animal rate.
/// Reserve stops at zero without causing death. This system runs before the clock.
/// Defaults leave both animals at 57 after three ticks; a test override of Hare = 1,
/// Fox = 2 leaves 57 and 54. Position and plant biomass are unaffected.
pub fn spend_energy(
    rules: Res<SpeciesEnergyRules>,
    mut creatures: Query<(&Species, &mut Energy), With<Creature>>,
) {
    for (species, mut energy) in &mut creatures {
        if let Some(cost) = rules.maintenance_units_per_tick(*species) {
            energy.reserve = energy.reserve.saturating_sub(cost);
        }
    }
}

/// Exercise 2: choose nearby food when a grazer needs it, without moving.
///
/// Planned reads: creature role, energy, positions, and finite food availability.
/// Introduce only the activity/target data required to remember the choice.
/// The actor receives bounded local observations, not the browser's whole-world
/// inspector. No available in-range food means no target. Equal candidates need
/// an explicit deterministic tie rule. Add parameters with this paired exercise.
///
/// # Panics
/// This learner-owned stub must be implemented before it is scheduled or called.
pub fn choose_food() {
    unimplemented!("Paired exercise 2: define the observation and target before scheduling");
}

/// Today's learner-owned rule: attempt one cardinal step, x before y.
/// Return actual cells traveled (zero or one). Reject invalid coordinates or an
/// unaffordable step without mutation; charge only accepted distance.
///
/// # Panics
/// Intentionally unfinished. The browser schedule does not call it yet.
pub fn move_one_cell(
    position: &mut Position,
    energy: &mut Energy,
    target: Position,
    units_per_cell: u32,
    config: WorldConfig,
) -> u32 {
    // Follow learning/content/14-movement.html: one affordable step.
    let _ = (position, energy, target, units_per_cell, config);
    todo!("Paired movement exercise: propose, validate, then commit one step")
}

/// Prepared ECS plumbing, deliberately unscheduled until the helper is reviewed.
/// Resolve an accepted stable target against live, nonempty food, then borrow
/// the animal's state for the rule. The fixture authors targets; this does not
/// decide when an animal is hungry or select a patch for it.
pub fn move_to_food(
    config: Res<WorldConfig>,
    rules: Res<MovementRules>,
    patches: Query<(&SimId, &Position, &FoodPatch), Without<Creature>>,
    mut creatures: Query<(&FoodTarget, &mut Position, &mut Energy), With<Creature>>,
) {
    for (target, mut position, mut energy) in &mut creatures {
        if let Some((_, destination, _)) = patches
            .iter()
            .find(|(id, _, patch)| **id == target.0 && patch.biomass > 0)
        {
            move_one_cell(
                &mut position,
                &mut energy,
                *destination,
                rules.units_per_cell,
                *config,
            );
        }
    }
}

/// Exercise 4: resolve a meal from biomass actually available at the target.
///
/// Planned reads: stable IDs, positions, eligibility, and target. Planned writes:
/// finite patch biomass, bounded creature energy, and the actual outcome event.
/// The first model uses same-cell contact; rendering size and click radius never
/// determine eligibility. Name biomass-to-energy conversion units and resolve
/// competing eaters explicitly. Growth and starvation are separate exercises.
///
/// # Panics
/// This learner-owned stub must be implemented before it is scheduled or called.
pub fn eat_food() {
    unimplemented!("Paired exercise 4: implement finite consumption before scheduling");
}
