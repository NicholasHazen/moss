use bevy_ecs::{prelude::*, schedule::{ExecutorKind, ScheduleBuildSettings}};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AppId(u64);
#[derive(Resource)]
struct NextId(u64);
#[derive(Resource, Default)]
struct Handles(Vec<Entity>);
#[derive(Clone, Debug, PartialEq, Eq)]
struct Seen { phase: &'static str, ids: Vec<AppId> }
#[derive(Resource, Default)]
struct Trace(Vec<Seen>);

fn enqueue_row(mut commands: Commands, mut next: ResMut<NextId>, mut handles: ResMut<Handles>) {
    let id = AppId(next.0);
    next.0 = next.0.checked_add(1).expect("application ID overflow");
    handles.0.push(commands.spawn(id).id());
}

fn observe_pending(rows: Query<&AppId>, mut trace: ResMut<Trace>) {
    let mut ids: Vec<_> = rows.iter().copied().collect();
    ids.sort();
    trace.0.push(Seen { phase: "Pending", ids });
}

fn observe_visible(rows: Query<&AppId>, mut trace: ResMut<Trace>) {
    let mut ids: Vec<_> = rows.iter().copied().collect();
    ids.sort();
    trace.0.push(Seen { phase: "Visible", ids });
}

fn manual_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.set_build_settings(ScheduleBuildSettings {
        auto_insert_apply_deferred: false,
        ..Default::default()
    });
    schedule.set_apply_final_deferred(true);
    schedule
}

fn command_schedule() -> Schedule {
    let mut schedule = manual_schedule();
    schedule.add_systems((enqueue_row, observe_pending, ApplyDeferred, observe_visible).chain());
    schedule
}

fn empty_world() -> World {
    let mut world = World::new();
    world.insert_resource(NextId(7));
    world.init_resource::<Handles>();
    world.init_resource::<Trace>();
    world
}

fn main() {
    let mut world = empty_world();
    command_schedule().run(&mut world);
    for seen in &world.resource::<Trace>().0 {
        println!("{}: {:?}", seen.phase, seen.ids);
    }
    let handle = world.resource::<Handles>().0[0];
    assert!(world.despawn(handle));
    let count = world.query::<&AppId>().iter(&world).count();
    println!("after removal: entities={} retained={:?}", count, world.resource::<Trace>().0[1].ids);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_row_becomes_queryable_at_explicit_application() {
        let mut world = empty_world();
        command_schedule().run(&mut world);
        assert_eq!(world.resource::<Trace>().0, vec![
            Seen { phase: "Pending", ids: vec![] },
            Seen { phase: "Visible", ids: vec![AppId(7)] },
        ]);
        let handle = world.resource::<Handles>().0[0];
        assert_eq!(world.get::<AppId>(handle), Some(&AppId(7)));
    }

    #[test]
    fn final_application_does_not_rewrite_an_earlier_observation() {
        let mut world = empty_world();
        let mut schedule = manual_schedule();
        schedule.add_systems((enqueue_row, observe_pending, observe_visible).chain());
        schedule.run(&mut world);
        assert_eq!(world.resource::<Trace>().0, vec![
            Seen { phase: "Pending", ids: vec![] },
            Seen { phase: "Visible", ids: vec![] },
        ]);
        let handle = world.resource::<Handles>().0[0];
        assert_eq!(world.get::<AppId>(handle), Some(&AppId(7)));
    }

    #[test]
    fn retained_application_id_survives_entity_removal() {
        let mut world = empty_world();
        command_schedule().run(&mut world);
        let handle = world.resource::<Handles>().0[0];
        assert!(world.despawn(handle));
        assert!(world.get_entity(handle).is_err());
        assert_eq!(world.query::<&AppId>().iter(&world).count(), 0);
        assert_eq!(world.resource::<Trace>().0[1].ids, vec![AppId(7)]);
    }

    #[test]
    fn another_execution_preserves_old_rows_and_allocates_a_new_id() {
        let mut world = empty_world();
        let mut schedule = command_schedule();
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Trace>().0[2..], [
            Seen { phase: "Pending", ids: vec![AppId(7)] },
            Seen { phase: "Visible", ids: vec![AppId(7), AppId(8)] },
        ]);
        assert_eq!(world.resource::<NextId>().0, 9);
    }
}
