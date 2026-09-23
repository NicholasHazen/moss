//! One deliberately red paired helper test; installed acceptance waits for review.

use bevy_ecs::prelude::*;
use moss_sim::{
    Energy, FoodPatch, FoodTarget, Position, SimClock, SimId, WorldConfig, install,
    lessons::move_one_cell, reset, tick,
};

#[test]
fn movement_charges_only_an_affordable_actual_step() {
    let mut position = Position { x: 2, y: 2 };
    let mut energy = Energy {
        reserve: 59,
        capacity: 100,
    };
    let target = Position { x: 4, y: 4 };
    let config = WorldConfig::default();

    assert_eq!(
        move_one_cell(&mut position, &mut energy, target, 2, config),
        1
    );
    assert_eq!(position, Position { x: 3, y: 2 });
    assert_eq!(energy.reserve, 57);

    energy.reserve = 1;
    assert_eq!(
        move_one_cell(&mut position, &mut energy, target, 2, config),
        0
    );
    assert_eq!(position, Position { x: 3, y: 2 });
    assert_eq!(energy.reserve, 1);

    energy.reserve = 10;
    let current = position;
    assert_eq!(
        move_one_cell(&mut position, &mut energy, current, 2, config),
        0
    );
    let outside = Position { x: -1, y: 2 };
    assert_eq!(
        move_one_cell(&mut position, &mut energy, outside, 2, config),
        0
    );
    assert_eq!(position, current);
    assert_eq!(energy.reserve, 10);

    // Review cases: all directions, x-before-y, bounds and exact affordability.
    for (start, goal, expected) in [
        ((1, 1), (0, 0), (0, 1)),
        ((1, 1), (2, 2), (2, 1)),
        ((1, 1), (1, 0), (1, 0)),
        ((1, 1), (1, 2), (1, 2)),
        ((30, 19), (31, 19), (31, 19)),
    ] {
        position = Position {
            x: start.0,
            y: start.1,
        };
        energy.reserve = 2;
        assert_eq!(
            move_one_cell(
                &mut position,
                &mut energy,
                Position {
                    x: goal.0,
                    y: goal.1
                },
                2,
                config
            ),
            1
        );
        assert_eq!(
            position,
            Position {
                x: expected.0,
                y: expected.1
            }
        );
        assert_eq!(energy.reserve, 0);
    }
    position = Position { x: -1, y: 0 };
    energy.reserve = 10;
    assert_eq!(
        move_one_cell(&mut position, &mut energy, target, 2, config),
        0
    );
    assert_eq!(position, Position { x: -1, y: 0 });
    assert_eq!(energy.reserve, 10);
}

fn installed() -> World {
    let mut world = World::new();
    install(&mut world, WorldConfig::default());
    world
}

fn animal(world: &mut World, id: SimId) -> (Position, Energy) {
    world
        .query::<(&SimId, &Position, &Energy)>()
        .iter(world)
        .find(|(candidate, ..)| **candidate == id)
        .map(|(_, position, energy)| (*position, *energy))
        .unwrap()
}

#[test]
#[ignore = "Agent activation checkpoint: enable movement only after helper review"]
fn movement_tick_resolves_target_and_reset() {
    let mut world = installed();
    tick(&mut world);
    assert_eq!(
        animal(&mut world, SimId(1)),
        (
            Position { x: 11, y: 10 },
            Energy {
                reserve: 57,
                capacity: 100
            }
        )
    );
    assert_eq!(
        animal(&mut world, SimId(2)),
        (
            Position { x: 21, y: 6 },
            Energy {
                reserve: 59,
                capacity: 100
            }
        )
    );
    for _ in 0..8 {
        tick(&mut world);
    }
    assert_eq!(
        animal(&mut world, SimId(1)),
        (
            Position { x: 16, y: 13 },
            Energy {
                reserve: 33,
                capacity: 100
            }
        )
    );
    tick(&mut world);
    assert_eq!(animal(&mut world, SimId(1)).1.reserve, 32);
    assert_eq!(
        world.query::<&FoodPatch>().single(&world).unwrap().biomass,
        80
    );
    reset(&mut world);
    assert_eq!(world.resource::<SimClock>().tick, 0);
    assert_eq!(animal(&mut world, SimId(1)).0, Position { x: 10, y: 10 });
    assert_eq!(animal(&mut world, SimId(1)).1.reserve, 60);
    assert_eq!(
        world.query::<&FoodTarget>().single(&world).unwrap().0,
        SimId(3)
    );
}

#[test]
#[ignore = "Agent activation checkpoint: enable movement only after helper review"]
fn movement_rejects_missing_empty_and_unaffordable_targets() {
    for case in 0..3 {
        let mut world = installed();
        let patch = world
            .query_filtered::<Entity, With<FoodPatch>>()
            .single(&world)
            .unwrap();
        match case {
            0 => {
                world.despawn(patch);
            }
            1 => {
                world.get_mut::<FoodPatch>(patch).unwrap().biomass = 0;
            }
            _ => {
                let hare = world
                    .query_filtered::<Entity, With<FoodTarget>>()
                    .single(&world)
                    .unwrap();
                world.get_mut::<Energy>(hare).unwrap().reserve = 2;
            }
        }
        tick(&mut world);
        let (position, energy) = animal(&mut world, SimId(1));
        assert_eq!(position, Position { x: 10, y: 10 });
        assert_eq!(energy.reserve, if case == 2 { 1 } else { 59 });
    }
}
