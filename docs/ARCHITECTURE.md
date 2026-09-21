# Architecture — small rules, one world

**Status:** The two-crate, one-world browser foundation is implemented. `SimTick` explicitly chains baseline maintenance before `complete_tick`. Later biological phases below remain proposals; movement, feeding, and death are not installed.

This document records the foundation and gives later rules somewhere sensible to live. Build evidence is in [BUILD_NOTES.md](BUILD_NOTES.md). Do not implement every type or future mechanism described here ahead of the paired learning steps.

## The smallest useful split

Use a small Cargo workspace with two real crates:

```text
moss-sim     Biological state, rules, scenario configuration, history, tests
    ↑
moss-web     Bevy application, camera, drawing, input, inspector, browser glue
```

`moss-sim` uses `bevy_ecs` and ordinary Rust. It must not require a window, renderer, DOM, network, filesystem, or wall-clock timer. `moss-web` depends on it and hosts the browser application.

**Start with one ECS world in the running browser app.** Simulation entities may also have view components, but their biological state is owned by the simulation module. The view reads that state and updates its own sprites or transforms. Do not create a second mirrored simulation or a generic engine adapter.

This boundary is primarily a code-ownership rule, not a claim that Bevy makes incorrect mutation impossible. Keep setters and rule functions internal where practical. UI interventions go through typed requests, not arbitrary component edits.

Native tests instantiate a small ECS world, install the same simulation resources and schedule, and advance it without graphics. The code map should show Nick where this happens. Bevy's official ECS introduction explains the ordinary structs/functions/queries underlying this approach. [R1](RESEARCH.md#r1-bevy-ecs)

## Modules grow with actual behavior

Initially, a few files for state, scenario setup, and ticking are sufficient. Add modules for activities, perception, lifecycle, or history when real code needs them. Do not scaffold twenty empty systems.

The code currently contains `SimId`, `Position`, `Creature`, separate
`Species` and `EcologicalRole` components, `Energy`, `FoodPatch`, `WorldConfig`, `SimClock`, and a bounded
`Journal`. Names belong to simulation data, including the patch's name; the
browser never invents an entity's identity. Ordinary maintenance will select
creatures by relevant components, not by an individual's ID.

The starting entities live in `fixture.rs`. Nick requested code landmarks for
the first few examples: `lessons.rs` contains the implemented maintenance loop
and three documented unimplemented stubs excluded from the schedule. This is a bounded teaching scaffold,
not a set of silently working rules or a future subsystem inventory.
[ECOLOGY_PLAN.md](ECOLOGY_PLAN.md) is the current model plan for species,
populations, plant patches, geometry, daylight, and later weather.

Data vocabulary, with future fields/types introduced only as needed:

| Data | Meaning |
|---|---|
| `SimId` | Stable identity for one creature or resource within a run. |
| `Position` | Authoritative world-space position, independent of the camera. |
| `Species` | Authored population grouping: initially Hare, Fox, Grass. |
| `EcologicalRole` | Grazer, Hunter, Producer; separate from species and nickname. |
| `Energy` | Metabolic reserve with a named capacity and units. |
| `Fatigue` | Future rest pressure, separate from nutrition; added with resting. |
| `Activity` | Future concrete state such as idle, seeking food, eating, resting, or fleeing. |
| `FoodPatch` | Name and finite available biomass today; replenishment parameters arrive with renewal. |
| `WorldConfig` | Validated dimensions today; scenario rates/settings arrive with the rules that use them. |
| `SimClock` | Completed tick count and the fixed duration represented by one tick. |

Later speed, health, vision, reproductive state, and lineage become data because implemented rules use them. Avoid defining a new component solely to display an unsupported character-sheet field.

## One tick has one meaning

The renderer asks for zero or more complete simulation ticks. The simulation never reads render delta time to determine hunger or movement. Pause runs no ticks; Step runs exactly one. Playback speed changes how many ticks are requested, not what each tick means.

A proposed ordering, to be exercised and revised explicitly:

0. Once implemented, sample environment for the executing tick and replenish plants before consumption, as specified in `ECOLOGY_PLAN.md`.
1. Accept due simulation requests and charge basal maintenance for living creatures present at the tick's start.
2. Resolve terminal conditions caused by maintenance, then build local observations for eligible creatures.
3. Choose or continue one activity for each eligible creature.
4. Apply movement and its explicit costs; resolve any terminal conditions caused by those costs.
5. Resolve available eating, rest, hunting, and reproduction effects in a documented stable order, rechecking eligibility.
6. Commit structural changes, publish actual semantic outcomes, and sample post-commit aggregates when due.
7. Mark the tick complete. Feedback influences the next decision opportunity, not an unlimited recursive loop.

Only implement phases with actual work. The early clock can be much smaller. Before adding starvation, explicitly test the chosen “maintenance before eating” consequence: reaching zero can prevent a last-moment meal. This is a tunable game rule, not a biological truth.

Keep the simulation schedule single-threaded initially. Any deferred spawn/despawn boundary must be explicit. Resolve contested outcomes by stable application IDs or another documented deterministic key, not ECS iteration order. Bevy provides explicit system ordering, but the application still owns its conflict rules. [R1](RESEARCH.md#r1-bevy-ecs)

## Decision and execution are distinct responsibilities

An activity-selection function reads the creature's own state and a bounded observation. It chooses or continues an activity. Execution validates targets and applies real effects. The UI's omniscient inspector is never an actor's perception source.

Early logic may be direct:

```text
Danger nearby → flee.
Otherwise hungry → seek food or eat if already in range.
Otherwise tired → rest.
Otherwise → idle or wander.
```

This is a baseline to challenge through scenarios, not a universal priority law. Keep current activity and useful targets long enough to observe commitment. Add separate start/stop thresholds when oscillation appears. A small function returning a concrete enum is enough; no option-provider registry is required.

## Space and time stay configurable

Start with a bounded two-dimensional integer grid and simple movement in an open area. Integer cells are a provisional simplification, not a demand for tile artwork. Rendering may interpolate between cells without deciding whether an interaction succeeds.

World dimensions belong in validated configuration. Initially, changes start a new run. Define bounds and whether cells may be shared; an easy first rule is that creatures may share cells while interactions enforce eligibility. Obstacles and pathfinding are later decisions.

The selected first interaction model permits shared cells and requires same-cell
contact for eating. Current renderer glyphs are equal-sized schematic markers,
not bodies. Physical radii/patch footprints arrive only with a rule that needs
them. Mouse picking and camera scale never supply biological distances.

Speed can initially mean a movement interval. If fractional movement is introduced, use an explicit progress accumulator or a documented fixed-point representation. Do not let zero intervals create infinite movement or truncation make slower creatures permanently motionless.

Local perception may scan all creatures in a tiny fixture and filter by range. That is not omniscience if only the filtered view reaches the policy. Replace expensive scans with a spatial index after measuring a problem; indexing must not change the rules.

## Costs have units and consequences

An illustrative accounting structure is:

```text
energy spent = basal cost per tick
             + movement cost per actual cell moved
             + explicit costs of completed actions
```

The expression is a design proposal, not a selected balance formula. Name every unit and check the arithmetic. If moving faster is meant to be more expensive, specify whether that comes from more distance, a higher per-distance cost, or both.

Rest reduces fatigue; it does not manufacture food energy. Food replenishment is an explicit environmental source. Reproduction transfers or spends parental resources according to an authored rule. Unknown rates do not need a universal need framework to become tunable.

## Lifecycle has a single resolver

Two hunters cannot consume the same prey. Mark it unavailable in authoritative resolution before queuing structural removal. Do not use deferred despawn alone as a lock. [R8](RESEARCH.md#r8-deferred-ecs-commands)

When reproduction is added, resolve a mating pair once, check both creatures again, apply costs/cooldowns together, and record the parent links. Newborns first act on the following tick. Terminal transitions occur once; dead entities cannot complete later actions in the same tick.

## Repeatability grows with the code

Start with authored fixtures and fixed tick counts. Add a controlled seeded random generator when behavior actually needs randomness. Record the seed, algorithm/version, relevant state, configuration, and rules/build identifiers.

Canonical comparisons include simulation state and relevant pending work, ordered by stable IDs. Exclude camera, render transforms, wall-clock metadata, and visual entities. A seed alone is not a replay guarantee.

Full save/resume is later work. A correct snapshot will need pending actions, clock, IDs, random state, retained observations, lifecycle/progression state, and version metadata. A history export is not automatically a resumable save.

## How the architecture earns its complexity

Add an abstraction when concrete uses reveal a shared need. Keep a failing fixture when it reveals a bad assumption. Explain any refactor with a before/after example and the friction it removes.

The target is not code that never changes. It is code Nick can find, understand, test, and change without losing the living world around it.
