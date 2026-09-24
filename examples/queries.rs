use bevy_ecs::{prelude::*, schedule::ExecutorKind};

#[derive(Component)]
struct Creature;

#[derive(Component)]
struct Name(&'static str);

#[derive(Component, Clone, Copy)]
enum Species { Hare, Fox }

#[derive(Component, Debug, PartialEq, Eq)]
struct Reserve(u32);

#[derive(Resource)]
struct Rates { hare: u32, fox: u32 }

fn maintenance(
    rates: Res<Rates>,
    mut creatures: Query<(&Species, &mut Reserve), With<Creature>>,
) {
    for (species, mut reserve) in &mut creatures {
        let cost = match *species {
            Species::Hare => rates.hare,
            Species::Fox => rates.fox,
        };
        reserve.0 = reserve.0.saturating_sub(cost);
    }
}

fn fixture() -> (World, Schedule, Entity) {
    let mut world = World::new();
    world.insert_resource(Rates { hare: 1, fox: 2 });
    world.spawn((Creature, Name("Fern"), Species::Hare, Reserve(60)));
    world.spawn((Creature, Name("Flint"), Species::Fox, Reserve(60)));
    world.spawn((Creature, Name("Incomplete"), Species::Hare));
    let display = world.spawn((Name("Display sample"), Species::Hare, Reserve(60))).id();
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(maintenance);
    (world, schedule, display)
}

fn audit(world: &mut World) -> Vec<(&'static str, Option<u32>)> {
    let mut query = world.query_filtered::<(&Name, Option<&Reserve>), With<Creature>>();
    let mut rows: Vec<_> = query.iter(world)
        .map(|(name, reserve)| (name.0, reserve.map(|value| value.0)))
        .collect();
    rows.sort_by_key(|row| row.0);
    rows
}

fn main() {
    let (mut world, mut schedule, _) = fixture();
    for _ in 0..3 { schedule.run(&mut world); }
    for (name, reserve) in audit(&mut world) {
        println!("{name}: {reserve:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_components_and_marker_define_membership() {
        let (mut world, mut schedule, display) = fixture();
        for _ in 0..3 { schedule.run(&mut world); }
        assert_eq!(audit(&mut world), vec![
            ("Fern", Some(57)), ("Flint", Some(54)), ("Incomplete", None),
        ]);
        assert_eq!(world.get::<Reserve>(display), Some(&Reserve(60)));
    }

    #[test]
    fn optional_read_exposes_missing_data() {
        let (mut world, _, _) = fixture();
        let required_count = world.query_filtered::<&Reserve, With<Creature>>()
            .iter(&world).count();
        assert_eq!(required_count, 2);
        let rows = audit(&mut world);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows.iter().filter(|(_, reserve)| reserve.is_none()).count(), 1);
    }

    #[test]
    fn next_execution_reads_the_current_resource() {
        let (mut world, mut schedule, _) = fixture();
        world.resource_mut::<Rates>().hare = 4;
        schedule.run(&mut world);
        assert_eq!(audit(&mut world), vec![
            ("Fern", Some(56)), ("Flint", Some(58)), ("Incomplete", None),
        ]);
    }
}
