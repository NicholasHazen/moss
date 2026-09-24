use bevy_ecs::{prelude::*, schedule::ExecutorKind};
use std::collections::BTreeSet;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct SimId(u32);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum Species {
    Hare,
    Fox,
}

#[derive(Component)]
struct Reserve(u32);

#[derive(Component)]
struct Maintenance(u32);

#[derive(Resource, Clone, Copy)]
struct Defaults {
    hare: u32,
    fox: u32,
}

impl Defaults {
    fn rate(self, species: Species) -> u32 {
        match species {
            Species::Hare => self.hare,
            Species::Fox => self.fox,
        }
    }
}

#[derive(Resource)]
struct AreaCells(u32);

#[derive(Clone, Copy)]
struct Profile {
    id: u32,
    species: Species,
    initial_reserve_units: u32,
    maintenance_override: Option<u32>,
}

const PAIR: [Profile; 2] = [
    Profile {
        id: 10,
        species: Species::Hare,
        initial_reserve_units: 60,
        maintenance_override: None,
    },
    Profile {
        id: 20,
        species: Species::Hare,
        initial_reserve_units: 60,
        maintenance_override: Some(1),
    },
];

const DENSER: [Profile; 4] = [
    PAIR[0],
    PAIR[1],
    Profile { id: 30, ..PAIR[0] },
    Profile { id: 40, ..PAIR[1] },
];

#[derive(Debug, PartialEq, Eq)]
enum AuthorError {
    ZeroId,
    DuplicateId(u32),
}

fn populate(world: &mut World, profiles: &[Profile]) -> Result<(), AuthorError> {
    let mut ids: BTreeSet<u32> = world.query::<&SimId>().iter(world).map(|id| id.0).collect();
    for profile in profiles {
        if profile.id == 0 {
            return Err(AuthorError::ZeroId);
        }
        if !ids.insert(profile.id) {
            return Err(AuthorError::DuplicateId(profile.id));
        }
    }
    let defaults = *world.resource::<Defaults>();
    for profile in profiles {
        let rate = profile
            .maintenance_override
            .unwrap_or(defaults.rate(profile.species));
        world.spawn((
            SimId(profile.id),
            profile.species,
            Reserve(profile.initial_reserve_units),
            Maintenance(rate),
        ));
    }
    Ok(())
}

fn empty_world() -> World {
    let mut world = World::new();
    world.insert_resource(Defaults { hare: 2, fox: 4 });
    world.insert_resource(AreaCells(64));
    world
}

fn fixture(profiles: &[Profile]) -> World {
    let mut world = empty_world();
    populate(&mut world, profiles).expect("authored fixture IDs are valid");
    world
}

fn upkeep(mut creatures: Query<(&Maintenance, &mut Reserve)>) {
    for (rate, mut reserve) in &mut creatures {
        reserve.0 = reserve.0.saturating_sub(rate.0);
    }
}

fn execute(world: &mut World, ticks: u32) {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(upkeep);
    for _ in 0..ticks {
        schedule.run(world);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Reading {
    id: u32,
    species: Species,
    reserve_units: u32,
    units_per_tick: u32,
}

fn readings(world: &mut World) -> Vec<Reading> {
    let mut query = world.query::<(&SimId, &Species, &Reserve, &Maintenance)>();
    let mut rows: Vec<_> = query
        .iter(world)
        .map(|(id, species, reserve, rate)| Reading {
            id: id.0,
            species: *species,
            reserve_units: reserve.0,
            units_per_tick: rate.0,
        })
        .collect();
    rows.sort_by_key(|row| row.id);
    rows
}

fn total_units(rows: &[Reading]) -> u64 {
    rows.iter().map(|row| u64::from(row.reserve_units)).sum()
}

fn main() {
    let defaults = *empty_world().resource::<Defaults>();
    println!(
        "defaults: Hare={}/tick Fox={}/tick",
        defaults.rate(Species::Hare),
        defaults.rate(Species::Fox)
    );
    for (label, profiles) in [("pair", PAIR.as_slice()), ("denser", DENSER.as_slice())] {
        let mut world = fixture(profiles);
        execute(&mut world, 3);
        let rows = readings(&mut world);
        println!(
            "{label}: count={} cells={} reserve_total={}",
            rows.len(),
            world.resource::<AreaCells>().0,
            total_units(&rows)
        );
        for row in rows {
            println!(
                "  #{}: reserve={} cost={}/tick",
                row.id, row.reserve_units, row.units_per_tick
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pair_shares_initial_inputs_except_its_configured_cost() {
        let mut world = fixture(&PAIR);
        assert_eq!(
            readings(&mut world),
            vec![
                Reading {
                    id: 10,
                    species: Species::Hare,
                    reserve_units: 60,
                    units_per_tick: 2
                },
                Reading {
                    id: 20,
                    species: Species::Hare,
                    reserve_units: 60,
                    units_per_tick: 1
                },
            ]
        );
        assert_eq!(world.resource::<AreaCells>().0, 64);
    }

    #[test]
    fn equal_execution_counts_reveal_the_individual_rate() {
        let mut world = fixture(&PAIR);
        execute(&mut world, 3);
        let rows = readings(&mut world);
        assert_eq!((rows[0].reserve_units, rows[1].reserve_units), (54, 57));
        execute(&mut world, 70);
        assert!(
            readings(&mut world)
                .iter()
                .all(|row| row.reserve_units == 0)
        );
        assert_eq!(readings(&mut world).len(), 2);
    }

    #[test]
    fn none_uses_species_default_but_some_zero_remains_zero() {
        let mut world = fixture(&[
            Profile {
                id: 1,
                maintenance_override: Some(0),
                ..PAIR[0]
            },
            Profile {
                id: 2,
                species: Species::Fox,
                ..PAIR[0]
            },
        ]);
        execute(&mut world, 1);
        let rows = readings(&mut world);
        assert_eq!((rows[0].units_per_tick, rows[0].reserve_units), (0, 60));
        assert_eq!((rows[1].units_per_tick, rows[1].reserve_units), (4, 56));
    }

    #[test]
    fn defaults_initialize_components_without_rewriting_existing_individuals() {
        let mut world = fixture(&PAIR);
        world.resource_mut::<Defaults>().hare = 99;
        execute(&mut world, 1);
        let rows = readings(&mut world);
        assert_eq!((rows[0].reserve_units, rows[1].reserve_units), (58, 59));
    }

    #[test]
    fn an_invalid_batch_does_not_partly_extend_the_population() {
        let mut world = fixture(&PAIR);
        let before = readings(&mut world);
        assert_eq!(
            populate(
                &mut world,
                &[Profile { id: 30, ..PAIR[0] }, Profile { id: 10, ..PAIR[0] },]
            ),
            Err(AuthorError::DuplicateId(10))
        );
        assert_eq!(readings(&mut world), before);
    }

    #[test]
    fn duplicate_and_zero_ids_are_rejected_before_any_spawn() {
        let mut world = empty_world();
        assert_eq!(
            populate(&mut world, &[PAIR[0], PAIR[0]]),
            Err(AuthorError::DuplicateId(10))
        );
        assert!(readings(&mut world).is_empty());
        assert_eq!(
            populate(&mut world, &[Profile { id: 0, ..PAIR[0] }]),
            Err(AuthorError::ZeroId)
        );
        assert!(readings(&mut world).is_empty());
    }

    #[test]
    fn authored_ids_and_sorted_readings_survive_reordered_input() {
        let mut first = fixture(&PAIR);
        let mut reordered = fixture(&[PAIR[1], PAIR[0]]);
        execute(&mut first, 3);
        execute(&mut reordered, 3);
        assert_eq!(readings(&mut first), readings(&mut reordered));
    }

    #[test]
    fn density_changes_the_total_without_improving_either_original_individual() {
        let mut pair = fixture(&PAIR);
        let mut denser = fixture(&DENSER);
        execute(&mut pair, 3);
        execute(&mut denser, 3);
        let a = readings(&mut pair);
        let b = readings(&mut denser);
        assert_eq!((a.len(), b.len()), (2, 4));
        assert_eq!(
            pair.resource::<AreaCells>().0,
            denser.resource::<AreaCells>().0
        );
        assert_eq!(total_units(&a), 111);
        assert_eq!(total_units(&b), 222);
        assert_eq!(b.iter().find(|row| row.id == 10), Some(&a[0]));
        assert_eq!(b.iter().find(|row| row.id == 20), Some(&a[1]));
    }
}
