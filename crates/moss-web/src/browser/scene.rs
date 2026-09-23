//! Presentation entities and transforms derived from simulation state.
use bevy::{camera::ScalingMode, core_pipeline::tonemapping::Tonemapping, prelude::*};
use moss_sim::{EcologicalRole, Position, SimId, WorldConfig};

use super::{MARKER_SIDE, Presentation};

#[derive(Component)]
struct MapCamera;

#[derive(Component)]
struct SelectionOutline;

pub(super) fn setup_view(world: &mut World) {
    world.spawn((Camera2d, Tonemapping::None, MapCamera));
    let config = world.resource::<WorldConfig>();
    let width = config.width() as f32;
    let height = config.height() as f32;
    // A finite chamber, with a faint cell grid, all presentation-only entities.
    world.spawn((
        Sprite::from_color(
            Color::srgb_u8(64, 85, 65),
            Vec2::new(width + 0.2, height + 0.2),
        ),
        Transform::from_xyz(width / 2.0, height / 2.0, -3.0),
    ));
    world.spawn((
        Sprite::from_color(Color::srgb_u8(218, 229, 203), Vec2::new(width, height)),
        Transform::from_xyz(width / 2.0, height / 2.0, -2.0),
    ));
    for x in 1..width as u32 {
        world.spawn((
            Sprite::from_color(Color::srgb_u8(205, 219, 189), Vec2::new(0.025, height)),
            Transform::from_xyz(x as f32, height / 2.0, -1.0),
        ));
    }
    for y in 1..height as u32 {
        world.spawn((
            Sprite::from_color(Color::srgb_u8(205, 219, 189), Vec2::new(width, 0.025)),
            Transform::from_xyz(width / 2.0, y as f32, -1.0),
        ));
    }
    // Four lines around selection, keeping the object itself visible.
    for _ in 0..4 {
        world.spawn((
            Sprite::from_color(Color::srgb_u8(26, 47, 34), Vec2::ONE),
            Transform::default(),
            Visibility::Hidden,
            SelectionOutline,
        ));
    }
}

pub(super) fn sync_view(world: &mut World, ui: &Presentation) {
    let missing: Vec<_> = world
        .query_filtered::<(Entity, &EcologicalRole), (With<SimId>, Without<Sprite>)>()
        .iter(world)
        .map(|(entity, role)| (entity, *role))
        .collect();
    for (entity, role) in missing {
        let color = match role {
            EcologicalRole::Grazer => Color::srgb_u8(47, 106, 82),
            EcologicalRole::Hunter => Color::srgb_u8(167, 91, 50),
            EcologicalRole::Producer => Color::srgb_u8(136, 165, 53),
        };
        world.entity_mut(entity).insert((
            Sprite::from_color(color, Vec2::splat(MARKER_SIDE)),
            Transform::default(),
        ));
    }
    let mut selected_position = None;
    for (id, position, mut transform) in world
        .query::<(&SimId, &Position, &mut Transform)>()
        .iter_mut(world)
    {
        transform.translation = Vec3::new(position.x as f32 + 0.5, position.y as f32 + 0.5, 1.0);
        if ui.selected == Some(*id) {
            selected_position = Some(transform.translation);
        }
    }
    for (index, (mut sprite, mut transform, mut visibility)) in world
        .query_filtered::<(&mut Sprite, &mut Transform, &mut Visibility), With<SelectionOutline>>()
        .iter_mut(world)
        .enumerate()
    {
        *visibility = if selected_position.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(position) = selected_position {
            let edge = MARKER_SIDE / 2.0 + 0.14;
            let stroke = 0.065;
            let offsets = [
                Vec2::new(-edge, 0.0),
                Vec2::new(edge, 0.0),
                Vec2::new(0.0, -edge),
                Vec2::new(0.0, edge),
            ];
            sprite.custom_size = Some(if index < 2 {
                Vec2::new(stroke, 2.0 * edge + stroke)
            } else {
                Vec2::new(2.0 * edge + stroke, stroke)
            });
            transform.translation = position + offsets[index].extend(1.0);
        }
    }
    for (mut transform, mut projection) in world
        .query_filtered::<(&mut Transform, &mut Projection), With<MapCamera>>()
        .iter_mut(world)
    {
        transform.translation.x = ui.map.center[0];
        transform.translation.y = ui.map.center[1];
        if let Projection::Orthographic(projection) = &mut *projection {
            // Match input's CSS pixel space even across device-scale changes.
            // Winit's logical resolution can briefly use a different pixel ratio.
            projection.scaling_mode = ScalingMode::Fixed {
                width: ui.map.size[0],
                height: ui.map.size[1],
            };
            projection.scale = ui.map.units_per_pixel;
        }
    }
}
