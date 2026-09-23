//! Authored diagnostic scene: one hare, one fox, one grass patch.
//!
//! Names are optional storytelling labels in the model, not unique identities.
//! Species supports many individuals; role describes a food-web responsibility.
//! This tiny fixture isolates rules. The planned population fixture will reuse
//! these component combinations with distinct IDs and configured counts.
//! See docs/design/ecology.md for that experiment and the plant/environment model.

use bevy_ecs::prelude::*;

use crate::{
    Creature, EcologicalRole, Energy, EventKind, FoodPatch, FoodTarget, Journal, JournalEntry,
    Position, SimId, Species, WorldConfig,
};

pub(super) fn place(world: &mut World) {
    let config = *world.resource::<WorldConfig>();
    // These casts are bounded by WorldConfig's maximum of 256 cells.
    let width = config.width() as i32;
    let height = config.height() as i32;
    for (id, name, species, role, position) in [
        (
            SimId(1),
            "Fern",
            Species::Hare,
            EcologicalRole::Grazer,
            Position {
                x: width / 3,
                y: height / 2,
            },
        ),
        (
            SimId(2),
            "Flint",
            Species::Fox,
            EcologicalRole::Hunter,
            Position {
                x: width * 2 / 3,
                y: height / 3,
            },
        ),
    ] {
        let entity = world
            .spawn((
                id,
                position,
                species,
                role,
                Creature { name },
                Energy {
                    reserve: 60,
                    capacity: 100,
                },
            ))
            .id();
        if species == Species::Hare {
            // Authored destination only: no food-choice policy runs yet.
            world.entity_mut(entity).insert(FoodTarget(SimId(3)));
        }
        record_placement(world, id, name);
    }

    let food_id = SimId(3);
    let food = FoodPatch {
        name: "Meadow",
        biomass: 80,
    };
    world.spawn((
        food_id,
        Position {
            x: width / 2,
            y: height * 2 / 3,
        },
        Species::Grass,
        EcologicalRole::Producer,
        food,
    ));
    record_placement(world, food_id, food.name);
}

fn record_placement(world: &mut World, id: SimId, name: &'static str) {
    world.resource_mut::<Journal>().record(JournalEntry {
        tick: 0,
        participants: vec![id],
        kind: EventKind::FixturePlaced { name },
    });
}
