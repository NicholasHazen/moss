//! Shared setup for integration tests; each test gets a fresh installed world.

use bevy_ecs::prelude::*;
use moss_sim::{FoodPatch, WorldConfig, install};

pub fn fixture(config: WorldConfig) -> World {
    let mut world = World::new();
    install(&mut world, config);
    world
}

/// Isolate maintenance while retaining the real installed schedule and assertions.
/// Future movement/feeding cannot act without a live food patch.
#[allow(dead_code)] // Integration-test crates compile this shared module separately.
pub fn remove_food(world: &mut World) {
    let patches: Vec<_> = world
        .query_filtered::<Entity, With<FoodPatch>>()
        .iter(world)
        .collect();
    for entity in patches {
        world.despawn(entity);
    }
}
