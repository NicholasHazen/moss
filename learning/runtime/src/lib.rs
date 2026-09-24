//! A small host for the existing, maintenance-only Moss simulation.
//! No copied biological rule and no camera state live in this crate.
use std::cell::RefCell;

use bevy_ecs::prelude::*;
use moss_sim::{Energy, FoodPatch, Position, RunRecord, SimClock, SimId, WorldConfig};

struct Runtime {
    world: World,
}

impl Runtime {
    fn new() -> Self {
        let mut world = World::new();
        moss_sim::install(&mut world, WorldConfig::default());
        Self { world }
    }

    /// Read one fixture field. -1 means absent/unknown, including an unknown field.
    /// Values in this authored fixture are nonnegative and fit i32.
    fn value(&mut self, id: u32, field: u32) -> i32 {
        let mut query = self.world.query::<(
            &SimId,
            Option<&Position>,
            Option<&Energy>,
            Option<&FoodPatch>,
        )>();
        let Some((_, position, energy, food)) = query
            .iter(&self.world)
            .find(|(entity_id, ..)| entity_id.0 == u64::from(id))
        else {
            return -1;
        };
        match field {
            0 => position.map(|p| p.x),
            1 => position.map(|p| p.y),
            2 => energy.and_then(|e| i32::try_from(e.reserve).ok()),
            3 => food.and_then(|f| i32::try_from(f.biomass).ok()),
            _ => None,
        }
        .unwrap_or(-1)
    }
}

thread_local! {
    // One authoritative world per module instance. Native tests use local worlds.
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

// SAFETY: the moss_fieldnotes_* symbols are unique to this host. All exported
// functions use scalar C ABI values; callers supply no pointers or references.
#[unsafe(no_mangle)]
pub extern "C" fn moss_fieldnotes_step() {
    RUNTIME.with_borrow_mut(|runtime| moss_sim::tick(&mut runtime.world));
}

// SAFETY: unique host export with no pointer arguments; see above.
#[unsafe(no_mangle)]
pub extern "C" fn moss_fieldnotes_reset() {
    RUNTIME.with_borrow_mut(|runtime| moss_sim::reset(&mut runtime.world));
}

// SAFETY: unique host export with no pointer arguments; see above.
#[unsafe(no_mangle)]
pub extern "C" fn moss_fieldnotes_tick() -> u64 {
    RUNTIME.with_borrow(|runtime| runtime.world.resource::<SimClock>().tick)
}

// SAFETY: unique host export with no pointer arguments; see above.
#[unsafe(no_mangle)]
pub extern "C" fn moss_fieldnotes_run() -> u64 {
    RUNTIME.with_borrow(|runtime| runtime.world.resource::<RunRecord>().number)
}

// SAFETY: unique host export with scalar arguments checked by value(); see above.
#[unsafe(no_mangle)]
pub extern "C" fn moss_fieldnotes_value(id: u32, field: u32) -> i32 {
    RUNTIME.with_borrow_mut(|runtime| runtime.value(id, field))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_schedule_spends_reserves_and_completes_ticks() {
        let mut runtime = Runtime::new();
        for _ in 0..3 {
            moss_sim::tick(&mut runtime.world);
        }
        assert_eq!(runtime.world.resource::<SimClock>().tick, 3);
        assert_eq!(runtime.value(1, 2), 57);
        assert_eq!(runtime.value(2, 2), 57);
        assert_eq!(runtime.value(3, 3), 80);
        assert_eq!((runtime.value(1, 0), runtime.value(1, 1)), (10, 10));
    }

    #[test]
    fn repeated_reads_leave_the_world_at_the_same_tick() {
        let mut runtime = Runtime::new();
        for _ in 0..100 {
            assert_eq!(runtime.value(1, 2), 60);
            assert_eq!(runtime.value(99, 2), -1);
            assert_eq!(runtime.value(1, 99), -1);
            assert_eq!(runtime.value(3, 2), -1);
        }
        assert_eq!(runtime.world.resource::<SimClock>().tick, 0);
    }

    #[test]
    fn depleted_animals_remain_present_under_current_rules() {
        let mut runtime = Runtime::new();
        for _ in 0..70 {
            moss_sim::tick(&mut runtime.world);
        }
        assert_eq!(runtime.value(1, 2), 0);
        assert_eq!(runtime.value(2, 2), 0);
        assert_eq!(runtime.value(3, 3), 80);
        assert_eq!(runtime.world.resource::<SimClock>().tick, 70);
    }

    #[test]
    fn reset_reauthors_fixture_and_advances_run_identity() {
        let mut runtime = Runtime::new();
        moss_sim::tick(&mut runtime.world);
        moss_sim::reset(&mut runtime.world);
        assert_eq!(runtime.world.resource::<RunRecord>().number, 2);
        assert_eq!(runtime.world.resource::<SimClock>().tick, 0);
        assert_eq!(runtime.value(1, 2), 60);
        assert_eq!(runtime.value(3, 3), 80);
    }
}
