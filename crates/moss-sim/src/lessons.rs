//! Four navigation landmarks for paired implementation, in intended order.
//!
//! **Maintenance is implemented and scheduled.** [`crate::install`] runs
//! [`spend_energy`] before the completed-tick counter. The other three functions
//! are unimplemented and unscheduled; their explicit panics prevent accidentally
//! treating a stub as a working rule.
//!
//! Implement one body, its focused regression in `tests/`, and its explicit
//! schedule ordering together. Never register all the stubs at once. Only the
//! first signature is selected; later parameter lists will grow with the data
//! introduced during those exercises. See `NOW.md` for the single active edit.

use bevy_ecs::prelude::*;

use crate::{Creature, Energy};

/// Exercise 1: spend one energy unit per completed tick on each creature.
///
/// `Query` borrows mutable energy only from entities carrying `Creature`.
/// Eligibility does not depend on name, species, or role. Bound reserve at zero;
/// zero causes no death yet. Order the completed implementation before the clock.
/// Observe both reserves going 60 → 57 after three Steps, with position and plant
/// biomass unchanged. Write the fixed-tick/zero-bound regression while pairing.
pub fn spend_energy(mut creatures: Query<&mut Energy, With<Creature>>) {
    for mut energy in &mut creatures {
        energy.reserve = energy.reserve.saturating_sub(1);
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

/// Exercise 3: move toward the accepted target within configured world bounds.
///
/// Planned reads: target, target position, and world dimensions. Planned write:
/// authoritative `Position`, never the view's `Transform`. Begin with one cell
/// per tick and an explicit axis/tie rule. Missing targets must not cause motion
/// toward invented positions. Add parameters when the target data exists.
///
/// # Panics
/// This learner-owned stub must be implemented before it is scheduled or called.
pub fn move_to_food() {
    unimplemented!("Paired exercise 3: implement bounded movement before scheduling");
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
