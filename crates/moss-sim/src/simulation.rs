//! Install, reset, and advance the single authoritative simulation world.

use bevy_ecs::{
    prelude::*,
    schedule::{ExecutorKind, ScheduleLabel},
};

use crate::{
    EventKind, Journal, JournalEntry, MovementRules, SimId, SpeciesEnergyRules, WorldConfig,
    fixture, lessons,
};

/// Simulated seconds represented by one complete tick (four ticks per second).
pub const TICK_SECONDS: f32 = 0.25;

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimClock {
    /// Number of fully completed simulation ticks in this run.
    pub tick: u64,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunRecord {
    pub number: u64,
}

/// The browser and headless tests request the very same schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimTick;

/// Install the simulation in the host's ECS world and start its first run.
pub fn install(world: &mut World, config: WorldConfig) {
    world.insert_resource(config);
    world.init_resource::<SpeciesEnergyRules>();
    world.init_resource::<MovementRules>();
    world.init_resource::<RunRecord>();

    let mut schedule = Schedule::new(SimTick);
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    // `chain()` makes tuple order explicit: spend energy, then complete the tick.
    // After the movement helper is green and reviewed, the agent adds
    // lessons::move_to_food between maintenance and complete_tick, then verifies
    // the installed tests and browser. Do not schedule the unfinished helper.
    schedule.add_systems((lessons::spend_energy, complete_tick).chain());
    world.add_schedule(schedule);
    reset(world);
}

/// Replace only entities with simulation IDs; preserve camera/UI entities and resources.
pub fn reset(world: &mut World) {
    let simulation_entities: Vec<Entity> = world
        .query_filtered::<Entity, With<SimId>>()
        .iter(world)
        .collect();
    for entity in simulation_entities {
        world.despawn(entity);
    }

    let run_number = {
        let mut run = world.resource_mut::<RunRecord>();
        run.number = run.number.checked_add(1).expect("run number overflow");
        run.number
    };
    world.insert_resource(SimClock::default());
    world.insert_resource(Journal::default());
    world.resource_mut::<Journal>().record(JournalEntry {
        tick: 0,
        participants: Vec::new(),
        kind: EventKind::RunStarted { number: run_number },
    });
    fixture::place(world);
}

/// Execute exactly one complete tick, independent of rendering or elapsed real time.
pub fn tick(world: &mut World) {
    world.run_schedule(SimTick);
}

fn complete_tick(mut clock: ResMut<SimClock>) {
    clock.tick = clock.tick.checked_add(1).expect("simulation tick overflow");
}
