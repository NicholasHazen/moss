# Follow one Step through Moss

[Context shelf](README.md) · [Guide home](../README.md) · [Species energy](../01-species-energy.md)

Fern has 60 energy. You press Step, and the inspector shows 59. Several files
cooperate to make that happen, but only one short loop decides the energy
change. Following that single number gives us a useful tour of the codebase.

This source walkthrough was reviewed on September 23, 2026. The
[runtime record](../../development/verification.md) retains the earlier native
species-rate and default-rate browser checks. Custom-rate browser acceptance
remains pending. Foraging remains planned work.

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

Drawing another frame does not itself spend energy. Autonomous biological rules
advance through executed simulation ticks. Compare outcomes after the same ticks
and accepted inputs, starting from the same world and rules. An explicit Reset
replaces the run through the
separate `moss_sim::reset` command; it is not another tick.
[playback.rs](../../../crates/moss-web/src/playback.rs) handles timing decisions,
including bounded automatic playback and hidden-tab suspension. Returning to a hidden
page leaves playback paused; there is no offline progress.

Equal wall time is a different comparison. During Play, a delayed frame requests
at most four automatic ticks and discards excess elapsed time. It does not save
that excess for later. Here are two calls-to-ticks traces, each starting with
Play enabled, a baseline frame at 0 ms, and the default reserve of 60:

| Automatic playback updates over two seconds | Result after those updates |
| --- | --- |
| Every 250 ms, through 2,000 ms | Eight ticks; each animal's reserve is 52. |
| One late update at 2,000 ms | Four ticks; each reserve is 56; playback reports that it slowed. |

In both cases, a following update at 2,250 ms requests one tick. The delayed
case does not recover the discarded four. The animals experienced different
amounts of simulation time during the same two seconds of wall time; their
maintenance rule still costs one per executed tick. Manual Step continues to
request exactly one complete tick. An [isolated native check](../../history/tutorial/2026-09-23.md#wall-time-and-executed-ticks)
verified these numbers using the existing playback code and installed simulation;
it did not simulate a real browser stall.

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
