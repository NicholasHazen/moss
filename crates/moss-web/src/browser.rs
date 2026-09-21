//! Thin browser host: requests ticks, projects ECS state, and publishes read-only inspection.
use bevy::{
    camera::ScalingMode,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{
        RenderPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
};
use moss_sim::{
    Creature, EcologicalRole, Energy, FoodPatch, Journal, Position, RunRecord, SimClock, SimId,
    Species, WorldConfig,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{playback::Playback, view::MapView};

// Schematic square glyphs, in map cells. Not body dimensions or interaction ranges.
// Drawing, picking and selection outlines share this one presentation measurement.
const MARKER_SIDE: f32 = 0.8;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = MossBridge, js_name = takeInput)]
    fn take_input() -> String;
    #[wasm_bindgen(js_namespace = MossBridge)]
    fn render(snapshot: &str);
    #[wasm_bindgen(js_namespace = MossBridge)]
    fn fail(message: &str);
}

#[derive(Deserialize)]
struct Input {
    now_ms: f64,
    hidden: bool,
    hidden_epoch: u32,
    width: f32,
    height: f32,
    actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Action {
    TogglePlay,
    Step,
    Reset,
    Fit,
    Select { id: String },
    Pan { dx: f32, dy: f32 },
    Zoom { x: f32, y: f32, factor: f32 },
    Click { x: f32, y: f32 },
}

#[derive(Resource, Default)]
struct Presentation {
    playback: Playback,
    map: MapView,
    selected: Option<SimId>,
    last_snapshot: String,
}

#[derive(Component)]
struct MapCamera;

#[derive(Component)]
struct SelectionOutline;

#[derive(Serialize)]
struct EntityView {
    id: String,
    name: &'static str,
    species: &'static str,
    role: &'static str,
    category: &'static str,
    x: i32,
    y: i32,
    energy: Option<u32>,
    capacity: Option<u32>,
    biomass: Option<u32>,
}

#[derive(Serialize)]
struct EventView {
    tick: String,
    text: String,
    participants: Vec<String>,
}

#[derive(Serialize)]
struct Snapshot {
    run: String,
    tick: String,
    running: bool,
    suspended: bool,
    slowed: bool,
    width: u32,
    height: u32,
    zoom: f32,
    selected: Option<String>,
    entities: Vec<EntityView>,
    events: Vec<EventView>,
    retained: usize,
    evicted: String,
    event_capacity: usize,
}

pub fn run() {
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

fn setup_view(world: &mut World) {
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

fn frame(world: &mut World) {
    let input = match serde_json::from_str::<Input>(&take_input()) {
        Ok(input) => input,
        Err(error) => {
            fail(&format!("Browser input could not be read: {error}"));
            return;
        }
    };
    world.resource_scope(|world, mut ui: Mut<Presentation>| {
        let width = world.resource::<WorldConfig>().width();
        let height = world.resource::<WorldConfig>().height();
        let new_size = [input.width.max(1.0), input.height.max(1.0)];
        if ui.map.size != new_size {
            ui.map.size = new_size;
            if ui.map.fitted {
                ui.map.fit(width, height);
            }
        }
        ui.playback.visibility(input.hidden, input.hidden_epoch);
        for action in input.actions {
            match action {
                Action::TogglePlay if !input.hidden => {
                    let running = !ui.playback.running;
                    ui.playback.set_running(running);
                }
                Action::Step if !input.hidden => {
                    ui.playback.set_running(false);
                    moss_sim::tick(world);
                }
                Action::Reset => {
                    moss_sim::reset(world);
                    ui.playback.set_running(false);
                    ui.playback.suspended = false;
                    ui.selected = None;
                    ui.map.fit(width, height);
                }
                Action::Fit => ui.map.fit(width, height),
                Action::Select { id } => ui.selected = id.parse::<u64>().ok().map(SimId),
                Action::Pan { dx, dy } => ui.map.pan(dx, dy),
                Action::Zoom { x, y, factor } => ui.map.zoom(x, y, factor),
                Action::Click { x, y } => {
                    let point = ui.map.world_at(x, y);
                    // Accessible click targets can grow with zoom; biology never
                    // reads this square hit area or the renderer's marker size.
                    let tolerance = (10.0 * ui.map.units_per_pixel).max(MARKER_SIDE / 2.0);
                    ui.selected = world
                        .query::<(&SimId, &Position)>()
                        .iter(world)
                        .filter_map(|(id, position)| {
                            let dx = position.x as f32 + 0.5 - point[0];
                            let dy = position.y as f32 + 0.5 - point[1];
                            let distance = dx * dx + dy * dy;
                            (dx.abs() <= tolerance && dy.abs() <= tolerance)
                                .then_some((*id, distance))
                        })
                        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
                        .map(|(id, _)| id);
                }
                Action::TogglePlay | Action::Step => {}
            }
        }
        for _ in 0..ui.playback.advance(input.now_ms) {
            moss_sim::tick(world);
        }
        sync_view(world, &ui);
        let snapshot = snapshot(world, &ui, width, height);
        // UI is a projection of ECS state, sent only when its displayed values change.
        let json = serde_json::to_string(&snapshot).expect("finite presentation values");
        if json != ui.last_snapshot {
            render(&json);
            ui.last_snapshot = json;
        }
    });
}

fn sync_view(world: &mut World, ui: &Presentation) {
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

fn snapshot(world: &mut World, ui: &Presentation, width: u32, height: u32) -> Snapshot {
    // A query is a typed borrow of the components on matching entities.
    let mut entities: Vec<_> = world
        .query::<(
            &SimId,
            &Position,
            Option<&Species>,
            Option<&EcologicalRole>,
            Option<&Creature>,
            Option<&Energy>,
            Option<&FoodPatch>,
        )>()
        .iter(world)
        .map(
            |(id, position, species, role, creature, energy, food)| EntityView {
                id: id.0.to_string(),
                name: creature
                    .map(|c| c.name)
                    .or_else(|| food.map(|f| f.name))
                    .unwrap_or("Unnamed"),
                species: species.map(|s| s.label()).unwrap_or("Unknown"),
                role: role.map(|r| r.label()).unwrap_or("unknown"),
                category: if creature.is_some() {
                    "creature"
                } else if food.is_some() {
                    "patch"
                } else {
                    "unknown"
                },
                x: position.x,
                y: position.y,
                energy: energy.map(|e| e.reserve),
                capacity: energy.map(|e| e.capacity),
                biomass: food.map(|f| f.biomass),
            },
        )
        .collect();
    entities.sort_by_key(|entity| entity.id.parse::<u64>().expect("numeric SimId"));
    let journal = world.resource::<Journal>();
    let events = journal
        .entries()
        .iter()
        .map(|entry| EventView {
            tick: entry.tick.to_string(),
            text: match entry.kind {
                moss_sim::EventKind::RunStarted { number } => format!("Run {number} initialized"),
                moss_sim::EventKind::FixturePlaced { name } => {
                    format!("{name} placed in the authored fixture")
                }
            },
            participants: entry
                .participants
                .iter()
                .map(|id| id.0.to_string())
                .collect(),
        })
        .collect();
    Snapshot {
        run: world.resource::<RunRecord>().number.to_string(),
        tick: world.resource::<SimClock>().tick.to_string(),
        running: ui.playback.running,
        suspended: ui.playback.suspended,
        slowed: ui.playback.slowed,
        width,
        height,
        zoom: 1.0 / ui.map.units_per_pixel,
        selected: ui.selected.map(|id| id.0.to_string()),
        entities,
        events,
        retained: journal.entries().len(),
        evicted: journal.evicted().to_string(),
        event_capacity: journal.capacity(),
    }
}
