//! Apply browser actions in order, request ticks, then publish the current view.
use bevy::prelude::*;
use moss_sim::{Position, SimId, WorldConfig};

use super::{
    MARKER_SIDE, Presentation,
    bridge::{Action, Input, fail, render, take_input},
    scene::sync_view,
    snapshot::snapshot,
};

pub(super) fn frame(world: &mut World) {
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
