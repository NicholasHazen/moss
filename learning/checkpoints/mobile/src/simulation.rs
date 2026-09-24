//! One authoritative world, explicit tick phases, reset, and read-only projection.

use bevy_ecs::{
    prelude::*,
    schedule::{ApplyDeferred, ExecutorKind},
};

use crate::history::{History, complete_tick};
use crate::mobile::{MobileError, MobileScenario, MobileSnapshot};
use crate::model::{
    Clock, Daylight, Grazer, GrazerReading, Identity, Life, Patch, PatchReading, Scenario,
    ScenarioError, Snapshot, TickLedger,
};
use crate::population::{PopulationConfig, PopulationError, PopulationSnapshot};
use crate::{feeding, lifecycle, mobile, population, reproduction, supply};

// This index stores entity handles only, never a second copy of biological state.
#[derive(Resource)]
pub(crate) struct Roster {
    pub grazers: Vec<Entity>,
    pub patches: Vec<Entity>,
}

pub struct CourseWorld {
    world: World,
    schedule: Schedule,
    initial: Scenario,
    initial_population: Option<PopulationConfig>,
    initial_mobile: Option<MobileScenario>,
}

impl CourseWorld {
    /// Validate all authored inputs before creating any simulation state.
    pub fn new(scenario: Scenario) -> Result<Self, ScenarioError> {
        scenario.validate()?;
        Ok(Self {
            world: install(&scenario, 1),
            schedule: tick_schedule(),
            initial: scenario,
            initial_population: None,
            initial_mobile: None,
        })
    }

    /// Enable the declared population policy after validating all inputs.
    /// First-arc construction remains available through `new`.
    pub fn new_population(
        mut scenario: Scenario,
        mut config: PopulationConfig,
    ) -> Result<Self, PopulationError> {
        let next_child_id = config.validate(&scenario)?;
        scenario.grazers.sort_by_key(|grazer| grazer.id);
        scenario.patches.sort_by_key(|patch| patch.id);
        config.founders.sort_by_key(|founder| founder.id);
        let mut world = install(&scenario, 1);
        population::install(&mut world, config.clone(), next_child_id);
        Ok(Self {
            world,
            schedule: population_schedule(),
            initial: scenario,
            initial_population: Some(config),
            initial_mobile: None,
        })
    }

    /// Opt into local observations, paid travel, and spatial grazer births.
    pub fn new_mobile(mut initial: MobileScenario) -> Result<Self, MobileError> {
        let next_child_id = initial.validate_and_normalize()?;
        let world = install_mobile(&initial, 1, next_child_id);
        Ok(Self {
            world,
            schedule: mobile_schedule(),
            initial: initial.scenario().clone(),
            initial_population: Some(initial.population().clone()),
            initial_mobile: Some(initial),
        })
    }

    /// The complete mobile observation includes both earlier projections and space.
    pub fn mobile_snapshot(&self) -> Option<MobileSnapshot> {
        let initial = self.initial_mobile.as_ref()?;
        Some(mobile::snapshot(
            &self.world,
            initial,
            self.snapshot(),
            self.population_snapshot()
                .expect("mobile mode includes a population policy"),
        ))
    }

    /// Validate before changing any mode, run identity, or saved reset selection.
    pub fn reset_with_mobile(&mut self, mut initial: MobileScenario) -> Result<(), MobileError> {
        let next_child_id = initial.validate_and_normalize()?;
        let run = self
            .world
            .resource::<Clock>()
            .run
            .checked_add(1)
            .expect("run identity exhausted");
        let world = install_mobile(&initial, run, next_child_id);
        self.world = world;
        self.schedule = mobile_schedule();
        self.initial = initial.scenario().clone();
        self.initial_population = Some(initial.population().clone());
        self.initial_mobile = Some(initial);
        Ok(())
    }

    /// Return None when the population policy is disabled; never invent observations.
    pub fn population_snapshot(&self) -> Option<PopulationSnapshot> {
        population::snapshot(&self.world, &self.initial)
    }

    /// Execute exactly one complete tick. No wall clock or render frequency is read.
    ///
    /// Panics on exhausted u64 counters rather than wrapping history identities.
    pub fn step(&mut self) {
        self.schedule.run(&mut self.world);
    }

    /// Restore the selected scenario's population, supplies, clock, and empty history.
    ///
    /// Panics if the run identity is exhausted; the old world remains unchanged.
    pub fn reset(&mut self) {
        if let Some(initial) = self.initial_mobile.clone() {
            self.reset_with_mobile(initial)
                .expect("stored mobile scenario was already validated");
        } else if let Some(config) = self.initial_population.clone() {
            self.reset_with_population(self.initial.clone(), config)
                .expect("stored population was already validated");
        } else {
            self.reset_with(self.initial.clone())
                .expect("stored scenario was already validated");
        }
    }

    /// Select first-arc mode and a new scenario, then start the next run. Invalid inputs leave the
    /// current world and reset scenario unchanged. A later `reset` repeats this
    /// newly selected scenario, with another run identity.
    ///
    /// Panics if the run identity is exhausted; the old world remains unchanged.
    pub fn reset_with(&mut self, scenario: Scenario) -> Result<(), ScenarioError> {
        scenario.validate()?;
        let run = self
            .world
            .resource::<Clock>()
            .run
            .checked_add(1)
            .expect("run identity exhausted");
        self.world = install(&scenario, run);
        // System parameter state belongs to the world for which it was initialized.
        self.schedule = tick_schedule();
        self.initial = scenario;
        self.initial_population = None;
        self.initial_mobile = None;
        Ok(())
    }

    /// Atomically select a validated population scenario under the next run ID.
    /// A later `reset` repeats this mode and both selected configurations.
    pub fn reset_with_population(
        &mut self,
        mut scenario: Scenario,
        mut config: PopulationConfig,
    ) -> Result<(), PopulationError> {
        let next_child_id = config.validate(&scenario)?;
        scenario.grazers.sort_by_key(|grazer| grazer.id);
        scenario.patches.sort_by_key(|patch| patch.id);
        config.founders.sort_by_key(|founder| founder.id);
        let run = self
            .world
            .resource::<Clock>()
            .run
            .checked_add(1)
            .expect("run identity exhausted");
        let mut world = install(&scenario, run);
        population::install(&mut world, config.clone(), next_child_id);
        self.world = world;
        self.schedule = population_schedule();
        self.initial = scenario;
        self.initial_population = Some(config);
        self.initial_mobile = None;
        Ok(())
    }

    /// Return a sorted owned observation without executing a system or changing state.
    pub fn snapshot(&self) -> Snapshot {
        let roster = self.world.resource::<Roster>();
        let mut grazers: Vec<_> = roster
            .grazers
            .iter()
            .map(|entity| {
                let id = self
                    .world
                    .get::<Identity>(*entity)
                    .expect("rostered identity exists");
                let grazer = self
                    .world
                    .get::<Grazer>(*entity)
                    .expect("rostered grazer exists");
                GrazerReading {
                    id: id.0,
                    reserve_units: grazer.reserve_units,
                    capacity_units: grazer.capacity_units,
                    maintenance_units_per_tick: grazer.maintenance_units_per_tick,
                    meal_units_per_tick: grazer.meal_units_per_tick,
                    feeding_site: grazer.feeding_site,
                    life: grazer.life,
                }
            })
            .collect();
        grazers.sort_by_key(|grazer| grazer.id);
        let mut patches: Vec<_> = roster
            .patches
            .iter()
            .map(|entity| {
                let id = self
                    .world
                    .get::<Identity>(*entity)
                    .expect("rostered identity exists");
                let patch = self
                    .world
                    .get::<Patch>(*entity)
                    .expect("rostered patch exists");
                PatchReading {
                    id: id.0,
                    biomass_units: patch.biomass_units,
                    capacity_units: patch.capacity_units,
                    growth_units_per_lit_tick: patch.growth_units_per_lit_tick,
                }
            })
            .collect();
        patches.sort_by_key(|patch| patch.id);
        let clock = self.world.resource::<Clock>();
        let daylight = *self.world.resource::<Daylight>();
        Snapshot {
            run: clock.run,
            tick: clock.tick,
            daylight,
            lit: daylight.is_lit(clock.tick),
            grazers,
            patches,
            ledger: *self.world.resource::<TickLedger>(),
            history: self.world.resource::<History>().snapshot(),
        }
    }
}

fn install_mobile(initial: &MobileScenario, run: u64, next_child_id: crate::SimId) -> World {
    let mut world = install(initial.scenario(), run);
    population::install(&mut world, initial.population().clone(), next_child_id);
    mobile::install(&mut world, initial);
    world
}

fn mobile_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(
        (
            begin_tick,
            population::begin_population_tick,
            mobile::begin_mobile_tick,
            population::mature,
            lifecycle::maintain,
            supply::grow,
            mobile::perception::observe,
            mobile::perception::choose,
            mobile::movement::travel,
            mobile::derive_contacts,
            feeding::feed,
            lifecycle::starve,
            reproduction::resolve_births,
            ApplyDeferred,
            population::cleanup_dead,
            ApplyDeferred,
            complete_tick,
            population::complete_population_tick,
            mobile::complete_mobile_tick,
        )
            .chain(),
    );
    schedule
}

fn population_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(
        (
            begin_tick,
            population::begin_population_tick,
            population::mature,
            lifecycle::maintain,
            supply::grow,
            feeding::feed,
            lifecycle::starve,
            reproduction::resolve_births,
            ApplyDeferred,
            population::cleanup_dead,
            ApplyDeferred,
            complete_tick,
            population::complete_population_tick,
        )
            .chain(),
    );
    schedule
}

fn tick_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(
        (
            begin_tick,
            lifecycle::maintain,
            supply::grow,
            feeding::feed,
            lifecycle::starve,
            complete_tick,
        )
            .chain(),
    );
    schedule
}

fn install(scenario: &Scenario, run: u64) -> World {
    let mut world = World::new();
    world.insert_resource(Clock { run, tick: 0 });
    world.insert_resource(scenario.daylight);
    world.insert_resource(TickLedger::default());
    world.insert_resource(History::new(scenario.history_limit));
    let grazers = scenario
        .grazers
        .iter()
        .map(|seed| {
            world
                .spawn((
                    Identity(seed.id),
                    Grazer {
                        reserve_units: seed.reserve_units,
                        capacity_units: seed.capacity_units,
                        maintenance_units_per_tick: seed.maintenance_units_per_tick,
                        meal_units_per_tick: seed.meal_units_per_tick,
                        feeding_site: seed.feeding_site,
                        life: Life::Alive,
                        first_eligible_tick: 1,
                    },
                ))
                .id()
        })
        .collect();
    let patches = scenario
        .patches
        .iter()
        .map(|seed| {
            world
                .spawn((
                    Identity(seed.id),
                    Patch {
                        biomass_units: seed.biomass_units,
                        capacity_units: seed.capacity_units,
                        growth_units_per_lit_tick: seed.growth_units_per_lit_tick,
                    },
                ))
                .id()
        })
        .collect();
    world.insert_resource(Roster { grazers, patches });
    world
}

fn begin_tick(mut clock: ResMut<Clock>, mut ledger: ResMut<TickLedger>) {
    clock.tick = clock.tick.checked_add(1).expect("tick counter exhausted");
    *ledger = TickLedger::default();
}

#[cfg(test)]
mod mobile_tests;
#[cfg(test)]
mod tests;
