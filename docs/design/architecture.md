# Architecture — small rules, one world

**Status:** The two-crate, one-world browser foundation is implemented. `SimTick` explicitly chains baseline maintenance before `complete_tick`. Later biological phases below remain proposals; movement, feeding, and death are not installed.

This document records the foundation and gives later rules somewhere sensible to live. Build evidence is in [verification](../development/verification.md). Do not implement every type or future mechanism described here ahead of the paired learning steps.

## The smallest useful split

The Cargo workspace has two packages:

```text
moss-sim     Biological state, rules, scenario configuration, history, tests
    ↑
moss-web     Bevy application, camera, drawing, input, inspector, browser glue
```

`moss-sim` uses `bevy_ecs` and ordinary Rust. It must not require a window, renderer, DOM, network, filesystem, or wall-clock timer. `moss-web` depends on it and hosts the browser application.

**The running browser app has one ECS world.** Simulation entities may also have view components, but their biological state is owned by the simulation module. The view reads that state and updates its own sprites or transforms. Do not create a second mirrored simulation or a generic engine adapter.

This boundary is primarily a code-ownership rule, not a claim that Bevy makes incorrect mutation impossible. Keep setters and rule functions internal where practical. UI interventions go through typed requests, not arbitrary component edits.

Native tests instantiate a small ECS world, install the same simulation resources and schedule, and advance it without graphics. The code map should show Nick where this happens. Bevy's official ECS introduction explains the ordinary structs/functions/queries underlying this approach. [R1](../research/README.md#r1-bevy-ecs)

## Modules grow with actual behavior

Keep the two Cargo packages as the dependency boundary. Within each crate, group
related code in modules and keep the crate root easy to scan. Add modules when
existing responsibilities separate naturally; a short type does not need its
own file. Do not scaffold empty future systems.

`moss-sim/src/lib.rs` declares private implementation modules and re-exports the
public types and functions, so callers still use paths such as `moss_sim::Energy`:

| Module | Responsibility |
| --- | --- |
| `components.rs` | Identity, position, species, roles, and food-patch data. |
| `energy.rs` | Reserves, species maintenance rates and the prepared shared travel rate. |
| `config.rs` | Validated world dimensions and their local tests. |
| `journal.rs` | Bounded outcome history and eviction tests. |
| `simulation.rs` | Installation, tick ordering, clocks, and reset. |
| `fixture.rs` | Authored starting entities and their placement events. |
| `lessons.rs` | Live maintenance, the movement exercise/adapter, and later choice/eating landmarks. |

Integration tests in `tests/bootstrap.rs` cover the installed scene and reset;
`tests/maintenance.rs` isolates maintenance with no food available.
`tests/movement.rs` holds the intentional red helper test and two explicit
activation checks, ignored until helper review. Their small shared setup
lives in `tests/common/mod.rs`, so Cargo does not discover it as another test
target. Unit tests stay beside the configuration and journal code they check.

`moss-web/src/browser.rs` builds the Bevy app and owns presentation state. Its
`browser/` children handle `bridge.rs` (wire input), `frame.rs` (ordered controls
and tick requests), `scene.rs` (drawing and camera projection), and `snapshot.rs`
(read-only inspection). Their interfaces use `pub(super)` where siblings need
access. `playback.rs` and `view.rs` retain their renderer-free tests.

The [coding conventions](../agents/coding-style.md) own module, visibility,
formatting and test style. Names and classification belong to simulation data;
maintenance selects relevant components rather than an individual's name or ID.
The unfinished movement helper and later choice/eating stubs are excluded
from the schedule. The prepared movement adapter is also unscheduled. [Ecology](ecology.md) owns species and future model details.

## One tick has one meaning

The renderer asks for zero or more complete simulation ticks. The simulation never reads render delta time to determine hunger or movement. Pause runs no ticks; Step runs exactly one. Playback speed changes how many ticks are requested, not what each tick means.

The running schedule contains maintenance → complete tick. Movement is prepared
but unscheduled. [Fieldnotes](../../learning/README.md) practices additional phases in its separate
course project. Live Moss adds phases only when their rules have been reviewed.
Its proposed dependencies are:

| Relationship | Why the order matters |
| --- | --- |
| Environment → growth → choice | A patch replenished this tick can become this tick's destination. |
| Maintenance → choice → movement | Thresholds and affordability use the reserve after upkeep. |
| Choice → movement → eating | A new or withdrawn target is visible before action, and arrival permits a same-tick meal. |
| Eating → proposed starvation resolution | A permitted meal may rescue an animal whose reserve reached zero earlier. |
| Resolution → completed-tick observation | Counts and recorded outcomes describe the committed result. |

The first foraging schedule therefore needs only maintenance, choice, movement,
eating and completion. Renewal and then environment extend its beginning.
Starvation remains an explicit future pairing decision, with the recommended
after-meal boundary in [the ecology plan](ecology.md#a-later-starvation-boundary-proposed-for-pairing).
An earlier terminal check would prohibit the last-chance meal; it is a different
model, not another simultaneous requirement. The code, traces and tests must
use the selected boundary together.

Rest, hunting and reproduction will need their own ordering when implemented.
In particular, prey consumed during a hunt must become ineligible for later
actions immediately; end-of-tick starvation does not solve that conflict.
Record actual outcomes at resolution and sample aggregates after the relevant
changes are visible. An activity's feedback influences a later decision
opportunity rather than recursively executing an unbounded loop of actions.

Keep the simulation schedule single-threaded initially. Any deferred spawn/despawn boundary must be explicit. Resolve contested outcomes by stable application IDs or another documented deterministic key, not ECS iteration order. Bevy provides explicit system ordering, but the application still owns its conflict rules. [R1](../research/README.md#r1-bevy-ecs)

## Decision and execution are distinct responsibilities

An activity-selection function reads the creature's own state and a bounded observation. It chooses or continues an activity. Execution validates targets and applies real effects. The UI's omniscient inspector is never an actor's perception source.

Today, the fixture supplies Fern with `FoodTarget(SimId(3))`. The prepared
movement adapter resolves that stable ID each tick and delegates to an ordinary
Rust helper. This is authored intent, not autonomous perception or choice.
The helper accepts position, energy, target and rate directly, so the future
source of those settings can change without rewriting the step rule.

When choice is added, begin with direct conditions and a concrete activity enum.
The proposed start/stop thresholds remember whether an animal is seeking;
`FoodTarget` separately identifies a currently eligible opportunity. Remembering
an activity does not guarantee that its old target is still valid. The
[choice lesson](../../learning/content/16-targets.html)
explains that data flow. Longer target commitment is a later policy choice if
reselection exposes a problem; it needs no general policy engine in advance.

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

### Species templates and individual attributes

**Live:** maintenance looks up shared species rates. **Planned:** species
defaults and authored overrides initialize each animal's authoritative typed
cost component. Defaults remain fixed during a run; reset rebuilds the authored
values. Temporary effects and live retuning need explicit policies when added.

Group fields by ownership and query use. Keep small frequently read values on
entities and larger shared definitions behind keys. The [ecology plan](ecology.md)
owns the selected policy; [individual variation lesson](../../learning/content/17-variation.html)
explains its tradeoffs. This is not a performance claim. Add derived caches or a
new storage abstraction only after measuring a real need.

### Energy accounting

An illustrative accounting structure for actual expenditure is:

```text
energy spent = maintenance actually deducted this tick
             + movement rate × actual cells moved
             + explicit costs actually paid for completed actions
```

The expression is a design proposal, not a selected balance formula. Maintenance
currently deducts at most the available reserve; requested cost and actual
expenditure can differ. Movement instead rejects an unaffordable step. Preserve
that distinction in observations and in the proposed after-meal starvation rule.
Name every unit and check the arithmetic. If moving faster is meant to be more
expensive, specify whether that comes from more distance, a higher per-distance
cost, or both.

Rest reduces fatigue; it does not manufacture food energy. Food replenishment is an explicit environmental source. Reproduction transfers or spends parental resources according to an authored rule. Unknown rates do not need a universal need framework to become tunable.

## Lifecycle has a single resolver

Two hunters cannot consume the same prey. Mark it unavailable in authoritative resolution before queuing structural removal. Do not use deferred despawn alone as a lock. [R8](../research/README.md#r8-deferred-ecs-commands)

When reproduction is added, resolve a mating pair once, check both creatures again, apply costs/cooldowns together, and record the parent links. Newborns first act on the following tick. Terminal transitions occur once; dead entities cannot complete later actions in the same tick.

## Repeatability grows with the code

Start with authored fixtures and fixed tick counts. Add a controlled seeded random generator when behavior actually needs randomness. Record the seed, algorithm/version, relevant state, configuration, and rules/build identifiers.

Canonical comparisons include simulation state and relevant pending work, ordered by stable IDs. Exclude camera, render transforms, wall-clock metadata, and visual entities. A seed alone is not a replay guarantee.

Full save/resume is later work. A correct snapshot will need pending actions, clock, IDs, random state, retained observations, lifecycle/progression state, and version metadata. A history export is not automatically a resumable save.

## How the architecture earns its complexity

Add an abstraction when concrete uses reveal a shared need. Keep a failing fixture when it reveals a bad assumption. Explain any refactor with a before/after example and the friction it removes.

The target is not code that never changes. It is code Nick can find, understand, test, and change without losing the living world around it.
