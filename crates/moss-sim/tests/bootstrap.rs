//! Authored scene, startup history, and tick/reset lifecycle.

mod common;

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;
use moss_sim::{
    Creature, EcologicalRole, Energy, EventKind, FoodPatch, Journal, Position, RunRecord, SimClock,
    SimId, Species, WorldConfig, reset, tick,
};

use common::fixture;

#[test]
fn authored_fixture_has_unique_ids_and_valid_positions_at_supported_sizes() {
    for config in [
        WorldConfig::new(8, 8).unwrap(),
        WorldConfig::default(),
        WorldConfig::new(256, 256).unwrap(),
    ] {
        let mut world = fixture(config);
        let mut ids = BTreeSet::new();
        for (id, position) in world.query::<(&SimId, &Position)>().iter(&world) {
            assert!(ids.insert(*id), "duplicate simulation ID: {id:?}");
            assert!((0..config.width() as i32).contains(&position.x));
            assert!((0..config.height() as i32).contains(&position.y));
        }
        assert_eq!(ids, BTreeSet::from([SimId(1), SimId(2), SimId(3)]));
        let mut creatures: Vec<_> = world
            .query::<(&SimId, &Creature, &Energy, &Species, &EcologicalRole)>()
            .iter(&world)
            .map(|(id, creature, energy, species, role)| (*id, *creature, *energy, *species, *role))
            .collect();
        creatures.sort_by_key(|(id, ..)| *id);
        assert_eq!(creatures.len(), 2);
        assert_eq!(creatures[0].1.name, "Fern");
        assert_eq!(creatures[1].1.name, "Flint");
        assert_eq!(
            (creatures[0].3, creatures[0].4),
            (Species::Hare, EcologicalRole::Grazer)
        );
        assert_eq!(
            (creatures[1].3, creatures[1].4),
            (Species::Fox, EcologicalRole::Hunter)
        );
        for (_, _, energy, _, _) in creatures {
            assert_eq!(energy.reserve, 60);
            assert_eq!(energy.capacity, 100);
        }
        let food: Vec<_> = world
            .query::<(&SimId, &FoodPatch, &Species, &EcologicalRole)>()
            .iter(&world)
            .collect();
        assert_eq!(food.len(), 1);
        assert_eq!(*food[0].0, SimId(3));
        assert_eq!(food[0].1.name, "Meadow");
        assert_eq!(food[0].1.biomass, 80);
        assert_eq!(
            (*food[0].2, *food[0].3),
            (Species::Grass, EcologicalRole::Producer)
        );
    }
}

#[test]
fn startup_journal_reports_actual_placement_with_participant_links() {
    let world = fixture(WorldConfig::default());
    let journal = world.resource::<Journal>();
    assert_eq!(journal.entries().len(), 4);
    assert_eq!(journal.evicted(), 0);
    assert_eq!(journal.oldest_tick(), Some(0));
    let first = journal.entries().front().unwrap();
    assert_eq!(first.kind, EventKind::RunStarted { number: 1 });
    assert!(first.participants.is_empty());
    for (entry, id) in journal.entries().iter().skip(1).zip(1..=3) {
        assert_eq!(entry.tick, 0);
        assert_eq!(entry.participants, vec![SimId(id)]);
        assert!(matches!(entry.kind, EventKind::FixturePlaced { .. }));
    }
}

#[derive(Component)]
struct PresentationOnly;

#[derive(Resource, PartialEq, Eq, Debug)]
struct CameraZoom(u32);

#[test]
fn ticks_are_explicit_and_reset_preserves_presentation_state() {
    let mut world = fixture(WorldConfig::default());
    let camera_entity = world.spawn(PresentationOnly).id();
    world.insert_resource(CameraZoom(3));
    assert_eq!(world.resource::<SimClock>().tick, 0);
    tick(&mut world);
    assert_eq!(world.resource::<SimClock>().tick, 1);
    tick(&mut world);
    assert_eq!(world.resource::<SimClock>().tick, 2);
    assert_eq!(world.resource::<RunRecord>().number, 1);

    let old_entities: Vec<_> = world
        .query_filtered::<Entity, With<SimId>>()
        .iter(&world)
        .collect();
    reset(&mut world);
    assert_eq!(world.resource::<SimClock>().tick, 0);
    assert_eq!(world.resource::<RunRecord>().number, 2);
    assert_eq!(*world.resource::<CameraZoom>(), CameraZoom(3));
    assert!(world.get::<PresentationOnly>(camera_entity).is_some());
    assert!(
        old_entities
            .iter()
            .all(|entity| world.get_entity(*entity).is_err())
    );
    assert_eq!(world.query::<&SimId>().iter(&world).count(), 3);
    let journal = world.resource::<Journal>();
    assert_eq!(journal.entries().len(), 4);
    assert_eq!(journal.evicted(), 0);
    assert_eq!(
        journal.entries().front().unwrap().kind,
        EventKind::RunStarted { number: 2 }
    );
    tick(&mut world);
    assert_eq!(world.resource::<SimClock>().tick, 1);
}
