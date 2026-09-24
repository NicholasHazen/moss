use bevy_ecs::{prelude::*, schedule::ExecutorKind};

#[derive(Component)]
struct Source;
#[derive(Component)]
struct RowId(u32);
#[derive(Component)]
struct PreviewFor(u32);
#[derive(Component, Debug, PartialEq, Eq)]
struct Sample(i32);

fn update_previews(
    sources: Query<(&RowId, &Sample), With<Source>>,
    mut previews: Query<(&PreviewFor, &mut Sample), Without<Source>>,
) {
    for (wanted, mut preview) in &mut previews {
        if let Some((_, source)) = sources.iter().find(|(id, _)| id.0 == wanted.0) {
            preview.0 = source.0;
        }
    }
}

fn preview_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(update_previews);
    schedule
}

struct Fixture {
    world: World,
    source: Entity,
    preview: Entity,
    protected: Entity,
    missing: Entity,
    unmarked: Entity,
}

fn fixture() -> Fixture {
    let mut world = World::new();
    world.spawn((Source, RowId(2), Sample(20)));
    let source = world.spawn((Source, RowId(1), Sample(60))).id();
    let preview = world.spawn((PreviewFor(1), Sample(0))).id();
    let protected = world.spawn((Source, RowId(9), PreviewFor(1), Sample(900))).id();
    let missing = world.spawn((PreviewFor(99), Sample(777))).id();
    let unmarked = world.spawn(Sample(333)).id();
    Fixture { world, source, preview, protected, missing, unmarked }
}

fn sample(world: &World, entity: Entity) -> i32 {
    world.get::<Sample>(entity).expect("fixture entity has a sample").0
}

fn main() {
    let mut f = fixture();
    preview_schedule().run(&mut f.world);
    println!("source={} preview={} protected={} missing={} unmarked={}",
        sample(&f.world, f.source), sample(&f.world, f.preview),
        sample(&f.world, f.protected), sample(&f.world, f.missing),
        sample(&f.world, f.unmarked));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_source_is_copied_without_mutation() {
        let mut f = fixture();
        preview_schedule().run(&mut f.world);
        assert_eq!(sample(&f.world, f.preview), 60);
        assert_eq!(sample(&f.world, f.source), 60);
    }

    #[test]
    fn source_marker_protects_an_entity_that_also_has_preview_data() {
        let mut f = fixture();
        preview_schedule().run(&mut f.world);
        assert_eq!(sample(&f.world, f.protected), 900);
    }

    #[test]
    fn missing_source_and_unmarked_rows_keep_their_previous_values() {
        let mut f = fixture();
        preview_schedule().run(&mut f.world);
        assert_eq!(sample(&f.world, f.missing), 777);
        assert_eq!(sample(&f.world, f.unmarked), 333);
    }

    #[test]
    fn later_run_reads_the_changed_source() {
        let mut f = fixture();
        let mut schedule = preview_schedule();
        schedule.run(&mut f.world);
        f.world.get_mut::<Sample>(f.source).unwrap().0 = 41;
        schedule.run(&mut f.world);
        assert_eq!(sample(&f.world, f.preview), 41);
        assert_eq!(sample(&f.world, f.source), 41);
        assert_eq!(sample(&f.world, f.protected), 900);
    }
}
