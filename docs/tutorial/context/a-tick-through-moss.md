# Follow one Step through Moss

[Context shelf](README.md) · [Guide home](../README.md) · [Species energy](../01-species-energy.md)

Fern has 60 energy. You press Step, and the inspector shows 59. Several files
cooperate to make that happen, but only one short loop decides the energy
change. Following that single number gives us a useful tour of the codebase.

This page describes the implemented foundation, checked against the source on
September 22, 2026. Species rates pass native tests, and the rebuilt browser's
default-rate smoke check passes. Custom-rate browser acceptance remains pending.
Foraging remains planned work.

## On this page

- [The browser requests a tick](#the-browser-requests-a-tick)
- [The schedule gives the tick an order](#the-schedule-gives-the-tick-an-order)
- [The query reaches both animals](#the-query-reaches-both-animals)
- [The inspector reads the result](#the-inspector-reads-the-result)
- [A test takes the same route](#a-test-takes-the-same-route)

## The browser requests a tick

The controls live in [shell.js](../../../crates/moss-web/shell.js).
[browser.rs](../../../crates/moss-web/src/browser.rs) sets up the application;
[browser/bridge.rs](../../../crates/moss-web/src/browser/bridge.rs) defines its
JavaScript connection and input messages. The `frame` function in
[browser/frame.rs](../../../crates/moss-web/src/browser/frame.rs) applies those
messages in order. Step pauses continuous playback and requests one complete
simulation tick. Play requests ticks as time accumulates. Neither control
contains the rule that subtracts energy.

That separation lets the rendering code draw as often as it needs. A fast monitor
does not make Fern hungrier. Autonomous biological rules advance only through
executed simulation ticks. An explicit Reset replaces the run through the
separate `moss_sim::reset` command; it is not another tick.
[playback.rs](../../../crates/moss-web/src/playback.rs) handles timing decisions,
including bounded catch-up and hidden-tab suspension. Returning to a hidden
page leaves playback paused; there is no offline progress.

## The schedule gives the tick an order

In [simulation.rs](../../../crates/moss-sim/src/simulation.rs), `install` creates
`SimTick`, an explicitly ordered, single-threaded schedule. This is an excerpt
from the existing installation code, not a new edit:

```rust
schedule.add_systems((lessons::spend_energy, complete_tick).chain());
```

`chain()` orders the two functions. Maintenance changes reserves first;
`complete_tick` advances the completed-tick counter afterward. `tick(&mut world)`
runs this schedule once. At the point the inspector says tick 1, that tick's
maintenance is already complete.

A schedule is where execution order belongs. Movement is the next planned
addition between maintenance and completion. Its adapter exists but remains
unscheduled until the helper is implemented and reviewed. Choice and eating
are later rules. A named function is not a running rule merely because it
appears in `lessons.rs`.

## The query reaches both animals

Chapter 1 updated this function in
[lessons.rs](../../../crates/moss-sim/src/lessons.rs). This block shows its
current implementation for reading:

```rust
pub fn spend_energy(
    rules: Res<SpeciesEnergyRules>,
    mut creatures: Query<(&Species, &mut Energy), With<Creature>>,
) {
    for (species, mut energy) in &mut creatures {
        if let Some(cost) = rules.maintenance_units_per_tick(*species) {
            energy.reserve = energy.reserve.saturating_sub(cost);
        }
    }
}
```

The query requests `Species` and mutable `Energy` from entities carrying `Creature`.
Fern and Flint both match. Meadow has `FoodPatch` instead of `Creature`, so
this loop never treats its biomass as an animal reserve. The query does not
look up either animal by nickname.

With the default rate of 1, starting at 60 leaves 59 after one pass. Starting
at 0, `saturating_sub(cost)` leaves
0 rather than underflowing an unsigned integer. No code here despawns the animal.
That is why zero energy currently means an exhausted reserve, not a defined
death event.

## The inspector reads the result

Back in the browser, `sync_view` in
[browser/scene.rs](../../../crates/moss-web/src/browser/scene.rs) updates the
visible markers. Then `snapshot` in
[browser/snapshot.rs](../../../crates/moss-web/src/browser/snapshot.rs) reads
simulation components into presentation data for the DOM inspector. It shows the
same authoritative reserve that the rule changed. It does not maintain its own
copy of Fern's metabolism.

Similarly, simulation `Position` contains the cell coordinates. A view transform
places a marker on screen. Panning changes how you see the cell; it does not
move the animal through the world. These boundaries are conventions in our
code, not a claim that ECS prevents every accidental mutation automatically.

## A test takes the same route

[maintenance.rs](../../../crates/moss-sim/tests/maintenance.rs) constructs an empty
`World`, calls the real `install`, and executes `tick`. No browser or renderer
is involved. The existing maintenance regression checks this trace:

| Executed ticks | Each animal's reserve |
| --- | --- |
| 0 | 60 |
| 3 | 57 |
| 63 | 0, with both animals still present |

Run that existing test from the workspace root:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test maintenance maintenance_spends_energy_and_stops_at_zero -- --exact
```

Expect one passing test. It proves the installed native simulation follows this
trace; it does not prove that the browser loaded the WASM bundle or displayed it
correctly. That is why a completed behavior also gets a browser check.

An optional prediction: at the current default rate, what remains after ten
executed ticks? The answer is 50 for each animal, independent of how many frames
were drawn between those ticks. Reset and ten Steps let you inspect that same
prediction in the browser.

We have now located the whole 60 → 59 change: input requests a tick, the schedule
calls a query-based rule, and presentation reads the resulting state. Chapter 1
changed the rate lookup inside this route; the route itself stayed intact.
