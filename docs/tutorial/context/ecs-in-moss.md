# ECS through one Moss tick

[Guide home](../README.md) · [Return to species energy](../01-species-energy.md)

The browser shows Fern with 60 energy. Press Step three times and it shows 57.
Follow that change from the authored fixture, through the simulation schedule,
into the inspector. That path explains the ECS vocabulary we actually need.

**On this page:** [Identity](#fern-hare-and-grazer-answer-different-questions) ·
[Data](#individual-data-and-shared-settings) · [Queries](#reading-the-query) ·
[Adapter](#a-helper-reaches-the-world-through-an-adapter) ·
[Borrow checks](#when-rust-compiles-but-bevy-rejects-the-queries) ·
[Tick order](#a-function-runs-because-the-schedule-calls-it) ·
[Tests](#what-a-green-test-proves) · [Reset](#identity-across-a-reset)

## Fern, Hare and Grazer answer different questions

In [crates/moss-sim/src/fixture.rs](../../../crates/moss-sim/src/fixture.rs), Fern's entity receives a `Creature` with
nickname `"Fern"`, `Species::Hare`, `EcologicalRole::Grazer`, and its own
`Energy` and `Position`. She also carries `FoodTarget(SimId(3))`, an authored
destination. Renaming her would not change her species rate.

Flint has `Species::Fox` and the hunter role. Meadow carries `FoodPatch` with
biomass, `Species::Grass`, and the producer role. One patch represents a local
group of plants. It currently neither grows nor gets eaten; no cellular
automaton runs underneath it.

Bevy's **archetype** has a separate technical meaning: the set of component
types attached to an entity. Species and role values alone do not change an
archetype. Fern now also carries `FoodTarget`, while Flint does not, so their
current simulation component sets differ. Adding or removing a component changes
that set; changing `Species::Hare` to another enum value does not. This terminology follows
the installed [Bevy ECS 0.18.1 archetype documentation][archetypes].

The fixture assembles data without a `Creature → Animal → Hare` class hierarchy.
More hares can use the same component combination with distinct IDs and
positions. The population chapter prepares that experiment.

## Individual data and shared settings

A **component** belongs to an entity. Fern's `Energy` can be 57 while Flint's
is 54. A **resource** is one shared value of a type in the world. Today,
`WorldConfig` supplies dimensions and `SimClock` counts completed ticks.

Chapter 1 has added and installed `SpeciesEnergyRules`; the maintenance loop
now reads it through `Res<SpeciesEnergyRules>`. Each animal keeps its own reserve
while this resource supplies its species' rate. Individual variation may
eventually need additional data; shared species settings come first.

The browser hosts one ECS world containing simulation and presentation data.
Ownership is a code boundary: `moss-sim` changes biological state;
`moss-web` reads it for display and requests accepted controls such as a tick or
reset. Camera movement changes the view of a position, not the authoritative
`Position` component used by a future movement rule.

## Reading the query

The current `spend_energy` signature in [crates/moss-sim/src/lessons.rs](../../../crates/moss-sim/src/lessons.rs) contains
this **type excerpt**:

```rust
Query<(&Species, &mut Energy), With<Creature>>
```

Read the first argument as the data requested: shared access to `Species` and
mutable access to `Energy` for the same entity.
Read the second as an additional condition: the entity must also carry
`Creature`. This function does not need the nickname, so `With<Creature>` checks
presence without fetching that component. Entities lacking any required
component do not appear. See [Bevy's pinned query reference][query].

Each returned item contains two component accesses: read species, update energy.
The query itself does not decide that hares pay 1 or foxes pay 2; the lookup
and subtraction do that.

Maintenance changes reserves independently, so iteration order does not affect
its outcome. Later, two animals may want the same finite food. We will explicitly
order contenders by stable `SimId` and recheck biomass before transferring it.
That resolution is planned work, not a property supplied by today's query.

## A helper reaches the world through an adapter

The prepared `move_to_food` in
[lessons.rs](../../../crates/moss-sim/src/lessons.rs) shows how an ordinary Rust
helper connects to ECS. **It remains unscheduled, and `move_one_cell` is still
today's unfinished edit.** Read this connection during review; there is no new
code to write here.

![The movement adapter finds Fern through Creature and FoodTarget, matches target 3 to Meadow's stable ID, and passes borrowed animal state plus a copied destination to the helper.](../visuals/adapter-to-helper.svg)

Follow Fern through one intended movement phase. Maintenance has reduced her
reserve from 60 to 59. The creature query supplies her `FoodTarget`, writable
`Position` and writable `Energy`. Flint lacks `FoodTarget`, so he does not enter
this loop. The patch query supplies Meadow's ID, position and biomass. The
adapter matches target 3 to patch 3 and checks that food remains. It does not
select a new destination or spend energy itself.

These are the **two existing query-parameter excerpts** from the adapter:

```rust
patches: Query<(&SimId, &Position, &FoodPatch), Without<Creature>>,
mut creatures: Query<(&FoodTarget, &mut Position, &mut Energy), With<Creature>>,
```

Both queries mention `Position`, but they reach separate entities:
`Without<Creature>` reads patch positions; `With<Creature>` permits animal
positions to change. An entity cannot satisfy both filters. The distinction
also lets Bevy verify that this system's two queries do not borrow the same
position incompatibly. This is an application of the pinned
[disjoint-query rules](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html#disjoint-queries),
not a guarantee supplied by the names `patches` and `creatures`.

Once the target resolves, this **existing call excerpt** crosses into the helper:

```rust
move_one_cell(
    &mut position,
    &mut energy,
    *destination,
    rules.units_per_cell,
    *config,
);
```

The first two arguments grant temporary access to Fern's actual components;
the helper's accepted mutations remain in the world. Bevy's access wrappers can
be borrowed as the `&mut Position` and `&mut Energy` the helper expects.
`*destination` copies the patch's small `Position` value. The helper may compare
against that destination, but it cannot move Meadow through the copy. The rate
and world bounds are also values, so the helper does not need an ECS lookup.

Under today's proposed step rule, these inputs should leave Fern at (11, 10)
with reserve 57: one upkeep unit was already spent, and the actual step costs 2.
That is an expected post-activation trace, not a current browser observation.
If no patch resolves, the call never happens. If the helper rejects a step, the
call happens but its inputs remain unchanged. If the adapter is unscheduled,
neither query runs. The same stationary picture can therefore have different
causes; identify which connection failed before changing the movement arithmetic.

Timing can also make a correct result look surprising. The later
[foraging investigation](../path/04-when-to-seek.md#turn-the-decision-into-behavior)
follows an animal that finishes a meal above its stop threshold while still
Seeking. One more Step separates an expected delayed decision from a state that
was never updated. It shows how to choose the next observation, with the answer
available beside the trace.

## When Rust compiles but Bevy rejects the queries

Suppose we removed only `Without<Creature>` from the patch query above. The
parameters would still compile, but Bevy would reject the system during
initialization with `error[B0001]`, before its body ran. This is a hypothetical
change; the prepared adapter already has the correct filter.

Why isn't `FoodPatch` enough to distinguish a patch? ECS permits one entity to
carry both `FoodPatch` and `Creature`. If it also had `SimId`, `Position`,
`FoodTarget` and `Energy`, it would satisfy both queries. One parameter could
then read the same position that the other can mutate. Rust can check each
parameter's type; Bevy checks whether their declared component accesses are
compatible when initializing the system. See the pinned
[disjoint-query explanation](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html#disjoint-queries).

Fern and Meadow being separate entities today does not prove those selections
can never overlap. The conflicting declarations fail even in an empty world.
`Without<Creature>` makes the exclusion explicit: an entity carrying both
components cannot enter the patch query. It changes selection, without making
that component combination illegal.

A single-threaded schedule cannot repair this conflict inside one system.
Neither can `.chain()`, which orders separate systems. If you encounter B0001,
inspect the queries' possible membership and access first. These distinctions
were checked in [four isolated native tests](../../history/tutorial/2026-09-22.md#query-access-checkpoint-september-22-2026);
they add no work to today's movement edit.

## A function runs because the schedule calls it

A **system** is a function whose parameters let Bevy supply its data access. A
**schedule** selects and orders systems. In [simulation.rs](../../../crates/moss-sim/src/simulation.rs),
`install` builds `SimTick` with a single-threaded executor and this **existing
schedule-wiring excerpt**:

```rust
schedule.add_systems((lessons::spend_energy, complete_tick).chain());
```

`chain()` orders maintenance before the completed-tick counter advances.
`tick(&mut world)` runs this schedule once. Three accepted Steps cause three
subtractions, independent of how many frames the browser draws.

`move_to_food` is a prepared, unscheduled adapter that resolves the authored
target and calls the unfinished `move_one_cell` helper. `choose_food` and
`eat_food` remain unscheduled stubs. Only reviewed rules enter the schedule.

## What a green test proves

[maintenance.rs](../../../crates/moss-sim/tests/maintenance.rs) creates a fresh `World`, calls `install`,
removes food to isolate maintenance,
then calls `tick`. Its maintenance regression exercises the same schedule entry
point as the browser, with no renderer required. To check the current path, run
from the workspace root:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test maintenance maintenance_spends_energy_and_stops_at_zero -- --exact
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
`FoodTarget` already stores the authored destination as a `SimId`; Reset
recreates it in the new run. Autonomous choice and cross-run history remain
future work.

[archetypes]: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/archetype/index.html
[query]: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html
