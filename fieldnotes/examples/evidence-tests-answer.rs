#[test]
fn a_fraction_requires_flooring() {
    assert_eq!(percentage(3, 8), Some(37));
}

#[test]
fn a_large_full_sample_is_still_full() {
    assert_eq!(percentage(u32::MAX, u32::MAX), Some(100));
}

#[test]
fn the_installed_observer_sees_the_selected_projection() {
    let mut world = fixture();
    installed_schedule().run(&mut world);
    assert_eq!(world.resource::<DisplayValue>().0, Some(37));
    assert_eq!(world.resource::<Trace>().0, vec![Some(37)]);
}

#[test]
fn unavailable_selection_clears_the_display_and_observation() {
    for selected_count in [0, 2] {
        let mut world = fixture();
        if selected_count == 0 {
            let selected: Vec<_> = world
                .query_filtered::<Entity, With<Selected>>()
                .iter(&world)
                .collect();
            assert_eq!(selected.len(), 1);
            for entity in selected {
                world.entity_mut(entity).remove::<Selected>();
            }
        } else {
            world.spawn((Selected, Sample { used: 1, capacity: 2 }));
        }
        installed_schedule().run(&mut world);
        assert_eq!(world.resource::<DisplayValue>().0, None);
        assert_eq!(world.resource::<Trace>().0, vec![None]);
    }
}
