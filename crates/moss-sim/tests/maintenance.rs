//! Installed maintenance behavior, including configured species rates.

mod common;

use bevy_ecs::prelude::*;
use moss_sim::{Creature, Energy, Species, SpeciesEnergyRules, WorldConfig, install, tick};

use common::{fixture, remove_food};

#[test]
fn maintenance_spends_energy_and_stops_at_zero() {
    let mut world = fixture(WorldConfig::default());
    remove_food(&mut world);

    // Confirm the starting reserve is 60; its maximum capacity is 100.
    let mut energies = world.query_filtered::<&Energy, With<Creature>>();

    assert_eq!(energies.iter(&world).count(), 2);
    for energy in energies.iter(&world) {
        assert_eq!(energy.reserve, 60);
        assert_eq!(energy.capacity, 100);
    }

    tick(&mut world);
    tick(&mut world);
    tick(&mut world);

    let mut energies = world.query_filtered::<&Energy, With<Creature>>();
    assert_eq!(energies.iter(&world).count(), 2);
    for energy in energies.iter(&world) {
        assert_eq!(energy.reserve, 57);
    }

    // After 63 total ticks, both animals remain present at zero energy.
    for _ in 0..60 {
        tick(&mut world);
    }

    assert_eq!(energies.iter(&world).count(), 2);
    for energy in energies.iter(&world) {
        assert_eq!(energy.reserve, 0);
    }
}

#[test]
fn maintenance_uses_species_rates_and_stops_at_zero() {
    let mut world = World::new();
    world.insert_resource(SpeciesEnergyRules {
        hare_maintenance_units_per_tick: 1,
        fox_maintenance_units_per_tick: 2,
    });
    install(&mut world, WorldConfig::default());
    remove_food(&mut world);

    for _ in 0..3 {
        tick(&mut world);
    }

    let mut energies = world.query_filtered::<(&Species, &Energy), With<Creature>>();
    assert_eq!(energies.iter(&world).count(), 2);

    for (species, energy) in energies.iter(&world) {
        let expected = match *species {
            Species::Hare => 57,
            Species::Fox => 54,
            Species::Grass => panic!("the animal fixture contains grass"),
        };
        assert_eq!(energy.reserve, expected);
        assert_eq!(energy.capacity, 100);
    }

    for _ in 0..60 {
        tick(&mut world);
    }

    assert_eq!(energies.iter(&world).count(), 2);
    for (_, energy) in energies.iter(&world) {
        assert_eq!(energy.reserve, 0);
        assert_eq!(energy.capacity, 100);
    }
}
