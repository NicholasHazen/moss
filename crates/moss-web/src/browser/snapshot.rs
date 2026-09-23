//! Read-only inspection data serialized for the browser interface.
use bevy::prelude::*;
use moss_sim::{
    Creature, EcologicalRole, Energy, FoodPatch, FoodTarget, Journal, Position, RunRecord,
    SimClock, SimId, Species, SpeciesEnergyRules,
};
use serde::Serialize;

use super::Presentation;

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
    authored_target: Option<String>,
    maintenance_units_per_tick: Option<u32>,
}

#[derive(Serialize)]
struct EventView {
    tick: String,
    text: String,
    participants: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct Snapshot {
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

pub(super) fn snapshot(world: &mut World, ui: &Presentation, width: u32, height: u32) -> Snapshot {
    let rates = *world.resource::<SpeciesEnergyRules>();
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
            Option<&FoodTarget>,
        )>()
        .iter(world)
        .map(
            |(id, position, species, role, creature, energy, food, target)| EntityView {
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
                authored_target: target.map(|target| target.0.0.to_string()),
                maintenance_units_per_tick: species
                    .and_then(|species| rates.maintenance_units_per_tick(*species)),
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
