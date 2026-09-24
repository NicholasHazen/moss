use bevy_ecs::{prelude::*, schedule::ExecutorKind};

#[derive(Component)]
struct Selected;
#[derive(Component)]
struct Sample { used: u32, capacity: u32 }
#[derive(Resource)]
struct DisplayValue(Option<u32>);
#[derive(Resource, Default)]
struct Trace(Vec<Option<u32>>);

fn percentage(used: u32, capacity: u32) -> Option<u32> {
    if capacity == 0 { return None; }
    Some(((u64::from(used.min(capacity)) * 100) / u64::from(capacity)) as u32)
}

fn project_selected(
    samples: Query<&Sample, With<Selected>>,
    mut display: ResMut<DisplayValue>,
) {
    display.0 = match samples.single() {
        Ok(sample) => percentage(sample.used, sample.capacity),
        Err(_) => None,
    };
}

fn observe_display(display: Res<DisplayValue>, mut trace: ResMut<Trace>) {
    trace.0.push(display.0);
}

fn installed_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems((project_selected, observe_display).chain());
    schedule
}

fn fixture() -> World {
    let mut world = World::new();
    world.insert_resource(DisplayValue(Some(99)));
    world.init_resource::<Trace>();
    world.spawn(Sample { used: 1, capacity: 2 });
    world.spawn((Selected, Sample { used: 3, capacity: 8 }));
    world
}

fn main() {
    let mut world = fixture();
    installed_schedule().run(&mut world);
    println!("display={:?} trace={:?}", world.resource::<DisplayValue>().0,
        world.resource::<Trace>().0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_defines_rounding_bounds_and_zero_capacity() {
        assert_eq!(percentage(3, 8), Some(37));
        assert_eq!(percentage(9, 8), Some(100));
        assert_eq!(percentage(0, 0), None);
        assert_eq!(percentage(u32::MAX, u32::MAX), Some(100));
    }

    #[test]
    fn installed_projection_uses_the_selected_row_before_observation() {
        let mut world = fixture();
        installed_schedule().run(&mut world);
        assert_eq!(world.resource::<DisplayValue>().0, Some(37));
        assert_eq!(world.resource::<Trace>().0, vec![Some(37)]);
    }

    #[test]
    fn no_selected_row_clears_a_previous_display_value() {
        let mut world = fixture();
        let selected: Vec<_> = world.query_filtered::<Entity, With<Selected>>()
            .iter(&world).collect();
        for entity in selected { world.entity_mut(entity).remove::<Selected>(); }
        installed_schedule().run(&mut world);
        assert_eq!(world.resource::<DisplayValue>().0, None);
        assert_eq!(world.resource::<Trace>().0, vec![None]);
    }

    #[test]
    fn multiple_selected_rows_do_not_silently_pick_one() {
        let mut world = fixture();
        world.spawn((Selected, Sample { used: 1, capacity: 2 }));
        installed_schedule().run(&mut world);
        assert_eq!(world.resource::<DisplayValue>().0, None);
        assert_eq!(world.resource::<Trace>().0, vec![None]);
    }

    #[test]
    fn a_green_helper_does_not_detect_an_omitted_system() {
        assert_eq!(percentage(3, 8), Some(37));
        let mut world = fixture();
        let mut incomplete = Schedule::default();
        incomplete.set_executor_kind(ExecutorKind::SingleThreaded);
        incomplete.add_systems(observe_display);
        incomplete.run(&mut world);
        assert_eq!(world.resource::<DisplayValue>().0, Some(99));
        assert_eq!(world.resource::<Trace>().0, vec![Some(99)]);
        assert_ne!(world.resource::<Trace>().0, vec![Some(37)]);
    }

    #[test]
    fn correct_final_display_can_hide_an_early_observer() {
        let mut world = fixture();
        let mut reversed = Schedule::default();
        reversed.set_executor_kind(ExecutorKind::SingleThreaded);
        reversed.add_systems((observe_display, project_selected).chain());
        reversed.run(&mut world);
        assert_eq!(world.resource::<DisplayValue>().0, Some(37));
        assert_eq!(world.resource::<Trace>().0, vec![Some(99)]);
        assert_ne!(world.resource::<Trace>().0, vec![Some(37)]);
    }
}
