# ECS through one Moss tick

[Guide home](../README.md) · [Return to species energy](../01-species-energy.md)

The browser shows Fern with 60 energy. Press Step three times and it shows 57.
Follow that change from the authored fixture, through the simulation schedule,
into the inspector. That path explains the ECS vocabulary we actually need.

**On this page:** [Identity](#fern-hare-and-grazer-answer-different-questions) ·
[Data](#individual-data-and-shared-settings) · [Queries](#reading-the-query) ·
[Tick order](#a-function-runs-because-the-schedule-calls-it) ·
[Tests](#what-a-green-test-proves) · [Reset](#identity-across-a-reset)

## Fern, Hare and Grazer answer different questions

In [crates/moss-sim/src/fixture.rs](../../../crates/moss-sim/src/fixture.rs), Fern's entity receives a `Creature` with
nickname `"Fern"`, `Species::Hare`, `EcologicalRole::Grazer`, and its own
`Energy` and `Position`. Renaming her would not change her species rate.

Flint has `Species::Fox` and the hunter role. Meadow carries `FoodPatch` with
biomass, `Species::Grass`, and the producer role. One patch represents a local
group of plants. It currently neither grows nor gets eaten; no cellular
automaton runs underneath it.

Bevy's **archetype** has a separate technical meaning: the set of component
types attached to an entity. Fern and Flint currently have the same simulation
component types even though their species and role values differ. Both fit the
same component shape. Adding or removing a component changes that shape;
changing `Species::Hare` to another enum value does not. This terminology follows
the installed [Bevy ECS 0.18.1 archetype documentation][archetypes].

The fixture assembles data without a `Creature → Animal → Hare` class hierarchy.
More hares can use the same component combination with distinct IDs and
positions. The population chapter prepares that experiment.

## Individual data and shared settings

A **component** belongs to an entity. Fern's `Energy` can be 57 while Flint's
is 54. A **resource** is one shared value of a type in the world. Today,
`WorldConfig` supplies dimensions and `SimClock` counts completed ticks.

Chapter 1 proposes a `SpeciesEnergyRules` resource. Each animal keeps its own
reserve while this resource supplies its species' rate. Individual variation
may eventually need additional data; shared species settings come first.

The browser hosts one ECS world containing simulation and presentation data.
Ownership is a code boundary: `moss-sim` changes biological state;
`moss-web` reads it for display and requests accepted controls such as a tick or
reset. Camera movement changes the view of a position, not the authoritative
`Position` component used by a future movement rule.

## Reading the query

The current `spend_energy` signature in [crates/moss-sim/src/lessons.rs](../../../crates/moss-sim/src/lessons.rs) contains
this **type excerpt**:

```rust
Query<&mut Energy, With<Creature>>
```

Read the first argument as the data requested: mutable access to `Energy`.
Read the second as an additional condition: the entity must also carry
`Creature`. This function does not need the nickname, so `With<Creature>` checks
presence without fetching that component. Entities lacking either required
component do not appear. See [Bevy's pinned query reference][query].

The proposed species-aware version requests `(&Species, &mut Energy)` as its
first argument. That means each returned item contains two component accesses
for the same entity: read species, update energy. The query itself does not
decide that hares pay 1 or foxes pay 2; the lookup and subtraction do that.

Maintenance changes reserves independently, so iteration order does not affect
its outcome. Later, two animals may want the same finite food. We will explicitly
order contenders by stable `SimId` and recheck biomass before transferring it.
That resolution is planned work, not a property supplied by today's query.

## A function runs because the schedule calls it

A **system** is a function whose parameters let Bevy supply its data access. A
**schedule** selects and orders systems. In [crates/moss-sim/src/lib.rs](../../../crates/moss-sim/src/lib.rs),
`install` builds `SimTick` with a single-threaded executor and this **existing
schedule-wiring excerpt**:

```rust
schedule.add_systems((lessons::spend_energy, complete_tick).chain());
```

`chain()` orders maintenance before the completed-tick counter advances.
`tick(&mut world)` runs this schedule once. Three accepted Steps cause three
subtractions, independent of how many frames the browser draws.

The later `choose_food`, `move_to_food`, and `eat_food` functions are unscheduled
stubs. Their names in the file do not make them execute. Each explicitly panics
if called before implementation; we register it only when its rule and tests are
ready.

## What a green test proves

[crates/moss-sim/tests/bootstrap.rs](../../../crates/moss-sim/tests/bootstrap.rs) creates a fresh `World`, calls `install`,
then calls `tick`. Its maintenance regression exercises the same schedule entry
point as the browser, with no renderer required. To check the current path, run
from the workspace root:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap maintenance_spends_energy_and_stops_at_zero -- --exact
```

Expect one named test passing. It checks both animals, their initial reserves,
the 60 → 57 change, and their continued presence at zero. Checking the count
matters: a loop containing assertions does nothing when the query returns no
animals.

Later helper tests isolate choices or arithmetic. Passing one will not prove
that its system runs, sees the intended entities, or handles competing eaters.
Each chapter therefore also requires an installed-schedule check and a browser
observation. Native tests cannot establish that a WebAssembly build actually
renders and accepts controls.

## Identity across a reset

An optional prediction: does Fern keep the same identity after Reset? The answer
depends on the identity you mean. Reset removes simulation entities and recreates
the authored scene. Fern again has `SimId(1)`, but the run number advances.
Our identity convention is **(run number, SimId)** for records that need to
distinguish these lives. The current browser does not retain earlier runs:
Reset discards the previous journal. Cross-run historical records are future work.

Bevy's `Entity` is the runtime handle for immediate ECS operations; it is not our
historical identity. The reset regression verifies that old simulation handles
no longer resolve and that `moss_sim::reset` preserves unrelated presentation
data. The browser's **Reset** action does additional UI work: it clears selection
and fits the camera, so your framing does change when you click that button.
Future target and history storage will be introduced with their chapters.

[archetypes]: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/archetype/index.html
[query]: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html
