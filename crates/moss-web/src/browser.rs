//! Browser app setup; input, scene projection and inspection live in focused modules.
mod bridge;
mod frame;
mod scene;
mod snapshot;

use bevy::{
    prelude::*,
    render::{
        RenderPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
};
use moss_sim::{SimId, WorldConfig};

use crate::{playback::Playback, view::MapView};

use self::{bridge::fail, frame::frame, scene::setup_view};

// Schematic square glyphs, in map cells. Not body dimensions or interaction ranges.
// Drawing, picking and selection outlines share this one presentation measurement.
const MARKER_SIDE: f32 = 0.8;

#[derive(Resource, Default)]
struct Presentation {
    playback: Playback,
    map: MapView,
    selected: Option<SimId>,
    last_snapshot: String,
}

pub(super) fn run() {
    std::panic::set_hook(Box::new(|info| fail(&format!("Moss stopped: {info}"))));
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Moss".into(),
                    canvas: Some("#moss-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::GL),
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb_u8(235, 239, 228)))
    .init_resource::<Presentation>();

    // Install into the app's world: there is no second simulation world or mirror.
    moss_sim::install(app.world_mut(), WorldConfig::default());
    app.add_systems(Startup, setup_view)
        .add_systems(Update, frame)
        .run();
}
