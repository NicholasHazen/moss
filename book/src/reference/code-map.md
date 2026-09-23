<p class="eyebrow">At the workbench · Find the responsibility</p>

# Where to look when something happens

Start with the question the behavior raises. If Fern pays the wrong amount,
you need the rule and its inputs. If the amount is right in a native test but
the browser shows an old number, you need the path from simulation state to
inspection. Reading every file in order would obscure that distinction.

Moss has two Cargo packages. `moss-sim` owns the biological state and executed
ticks. `moss-web` depends on it and owns browser input, the camera, drawing and
inspection. The running app has one ECS world. It does not synchronize a hidden
simulation world with a second authoritative world in the view.

```text
Browser input → typed requests → execute a simulation tick
                                      ↓
                              one authoritative world
                                      ↓
                          sprites and inspector values
```

This diagram shows responsibility, not a claim that Rust prevents every possible
misuse. Many component fields are publicly mutable. The project rule is that
simulation-owned code changes biology while the view derives presentation from
it. Typed requests keep intentional interventions explicit.

## Follow one tick through the code

In [`simulation.rs`](../../../crates/moss-sim/src/simulation.rs), `install`
creates the resources and schedule, `tick` executes that schedule, and `reset`
reconstructs the starting fixture. This is the first file to inspect when a
correct system never seems to run. The edition baseline schedules maintenance
before completing the tick; the movement adapter is prepared but absent from
that chain.

The rule lives in [`lessons.rs`](../../../crates/moss-sim/src/lessons.rs).
`spend_energy` already iterates over animals and spends the applicable species
rate. `move_one_cell` is the current unfinished helper. `move_to_food` is its
prepared query adapter. The later `choose_food` and `eat_food` stubs are
landmarks, not systems to register before their rules are implemented.

Values used by those functions come from a few small modules:

| If you want to understand… | Open… |
| --- | --- |
| Stable identity, species, role, position and patch supply | [`components.rs`](../../../crates/moss-sim/src/components.rs) |
| Reserve, capacity, shared upkeep and travel rates | [`energy.rs`](../../../crates/moss-sim/src/energy.rs) |
| Valid world dimensions and why coordinate casts fit | [`config.rs`](../../../crates/moss-sim/src/config.rs) |
| Why Fern, Flint and Meadow begin where they do | [`fixture.rs`](../../../crates/moss-sim/src/fixture.rs) |
| Which events are retained and when old detail is evicted | [`journal.rs`](../../../crates/moss-sim/src/journal.rs) |

The crate root, [`lib.rs`](../../../crates/moss-sim/src/lib.rs), is a useful
small map of its public interface. It declares implementation modules and
re-exports selected types. That is why a test can write `moss_sim::Energy`
without depending on the internal `energy` module path. A re-export changes
how a caller names a type; it does not create another copy of the type.

## When the rule is right but the screen is wrong

[`browser/frame.rs`](../../../crates/moss-web/src/browser/frame.rs) applies
ordered controls and requests ticks. [`playback.rs`](../../../crates/moss-web/src/playback.rs)
handles the pacing calculation without needing a renderer. The clock advances
through executed simulation work, not simply because a wall-clock interval
passed or a frame arrived.

[`browser/scene.rs`](../../../crates/moss-web/src/browser/scene.rs) turns
simulation positions into presentation and handles the camera projection.
[`browser/snapshot.rs`](../../../crates/moss-web/src/browser/snapshot.rs)
projects data for inspection. A sprite transform is a view of the simulation
position. Editing only that transform would not be a movement rule.

The browser entry and interface are assembled in
[`browser.rs`](../../../crates/moss-web/src/browser.rs). Its neighboring
[`browser/bridge.rs`](../../../crates/moss-web/src/browser/bridge.rs) handles
the browser boundary. These files are useful when a control fails to submit
the expected request; they are not where maintenance should learn to charge
an animal.

## Ask a test one narrow question

`crates/moss-sim/tests/maintenance.rs` checks upkeep in an installed world.
`crates/moss-sim/tests/movement.rs` starts with the ordinary helper and contains
two later installed-world acceptance tests. At this edition's starting point,
the helper test is intentionally red and the two activation tests are ignored.

The three evidence levels are worth keeping separate:

1. A **helper test** establishes what a particular call does to its inputs.
2. An **installed-world test** establishes what the configured schedule does
   to the entities and resources it can reach.
3. A **browser observation** establishes what a particular build presents
   through actual controls and inspection.

None replaces the others. A green helper can remain unscheduled. A correct
schedule can feed a stale inspector. A moving square alone cannot tell whether
the energy account is right. The first chapter crosses those boundaries in
that order so a mismatch has somewhere specific to be investigated.

The links on this page open exact source snapshots in the exported web book.
In the Markdown edition they point to files in the checkout. A snapshot says
what this edition was built from; consult the live file before making a new edit.

Return to [one affordable step](../chapters/01-one-affordable-step.md#your-first-edit)
when you want the next concrete contribution.
