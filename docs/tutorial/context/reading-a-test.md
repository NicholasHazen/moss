# Read a test as a small experiment

[Context shelf](README.md) · [Short-session path](../path/README.md#checking-your-work)

When a test is unfamiliar, start with the sentence it is trying to defend.
Moss's existing maintenance test says: two animals start with 60, three executed
ticks leave both at 57, and further maintenance stops at zero without removing
them. The Rust code gives that sentence a controlled starting point, an action,
and an observation. You already wrote this behavior; the test is a way to ask it
the same question again after another change.

**Jump to:** [Read the setup](#find-the-cause-before-the-assertion) ·
[Read a failure](#find-out-how-far-the-run-got) ·
[Know what green establishes](#a-passing-test-has-a-boundary)

## Find the cause before the assertion

Open [maintenance.rs](../../../crates/moss-sim/tests/maintenance.rs) and find
`maintenance_spends_energy_and_stops_at_zero`. This **excerpt from its setup**
is not another edit:

```rust
let mut world = fixture(WorldConfig::default());
remove_food(&mut world);
```

`fixture` returns a Bevy `World` with Moss installed. `mut world` lets the test
pass a mutable reference when it advances the simulation. Removing food isolates
maintenance from future meals and travel; it does not remove the maintenance
system. Test preparation deliberately makes other causes unavailable, so the
final energy can answer a specific question.

The action is three calls to `tick(&mut world)`. Each runs the installed simulation
schedule once. Calling `spend_energy` indirectly through that schedule matters:
this test can catch a system that exists in the source but is no longer scheduled.
A test of a standalone subtraction helper could not catch that omission.

## Read the query as a request

The **existing observation excerpt** is:

```rust
let mut energies = world.query_filtered::<&Energy, With<Creature>>();
assert_eq!(energies.iter(&world).count(), 2);
for energy in energies.iter(&world) {
    assert_eq!(energy.reserve, 57);
}
```

`::<...>` supplies type arguments to the method. Here they say “read `Energy`
from entities that also have `Creature`.” `With<Creature>` checks presence; it
does not add a second value to each loop item. `query_filtered` prepares a query,
and `iter(&world)` obtains its matching rows. Each new iterator can visit those
rows again; counting one iterator has not consumed the entities.

The count assertion is essential. If a future component change accidentally
makes the query match no animals, the loop would make zero assertions and finish
successfully. Checking two rows makes “both animals have 57” mean both animals
were actually inspected. It also catches accidental removal before looking at
the values.

## Make the expected number independent

`assert_eq!(energy.reserve, 57)` compares the observed value on the left with
the expected value on the right. This convention makes Rust's failure report
easy to read. A diagnostic such as **left: 60, right: 57** means the query found
an animal, but the observed reserve did not change as expected. Check the
executed ticks and scheduled system before changing the expected number.

Why write literal 57 rather than call the cost function to calculate the answer?
The test needs an expectation independent enough to catch a wrong cost. If the
implementation accidentally charges 2 and the assertion asks that same
implementation what to expect, both can agree on 54 and hide the mistake.
The story supplies 57: start at 60 and spend 1 on each of three ticks.

One small variation makes the scope clear. Suppose one animal has 54 and the other
60. Their total is still 114, the same total as 57 and 57. A sum-only assertion
would pass. The existing per-animal assertions reject that outcome because each
animal's reserve is part of the claim. Later comparisons also use stable IDs
when the expected values intentionally differ.

## Run the question you intended

From the repository root, this existing test can be run alone:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test maintenance maintenance_spends_energy_and_stops_at_zero -- --exact
```

`-p moss-sim` selects the package. `--test maintenance` selects the integration
test file. The following name filters to the function, and `-- --exact` tells
the test harness to use the exact name. Expect **one passed test**, not zero
matches. Compilation errors happen before this experiment runs; they require
fixing setup or Rust code, rather than revising the biological expectation.

## Find out how far the run got

Start with the first diagnostic that names your source, rather than Cargo's
final `error: test failed` summary. These fragments identify different stages;
you only need the row matching the output in front of you.

| Recognizable output | What happened and where to look next |
| --- | --- |
| `error[E0308]: mismatched types` | Compilation stopped before this test ran. Read the named expression and its expected/actual types. The movement example below shows one possible cause. |
| `not yet implemented: Paired movement exercise…` | The test called the unfinished `move_one_cell` and its `todo!` panicked. This is today's intended starting failure; the helper never returned a distance to compare. |
| `assertion ... failed`, followed by `left` and `right` | The test reached a comparison. Follow those values back to its setup and action, as with the maintenance example above. Preserve the intended expectation while investigating. |
| `0 passed; 0 failed`, with tests filtered out | No selected test ran. Compare the target and exact function name with the guide; a successful process exit does not establish the rule. |

For one concrete Rust diagnostic, imagine adding a semicolon to the worked
movement helper's last line, changing `distance` to `distance;`. The signature
promises a `u32`, but that body would finish with `()`, Rust's unit value, instead
of supplying the distance. The compiler reports **expected `u32`, found `()`**
and suggests removing that semicolon. Keep semicolons on statements such as
`energy.reserve -= cost;`; it is this final expression that returns the value.
The [Rust book's function explanation](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values)
shows the same distinction. This diagnosis applies to that particular expression,
not every E0308 error.

An unchanged `todo!()` behaves differently: it compiles, then panics when reached.
In today's first assertion, Rust must call `move_one_cell` before comparing its
result with 1. The panic stops that call; it does not mean the helper returned 0.
That is why replacing the body is the intended edit and changing the assertion
would not fix the failure. The [verification record](../authoring/verification.md#test-diagnostics-checkpoint-september-22-2026)
separates these isolated examples from live runtime evidence.

## A passing test has a boundary

The maintenance test advances the installed schedule. Today's movement test
instead calls `move_one_cell` directly with ordinary values. Although Cargo
builds files under `tests/` as separate integration-test targets, that label does
not tell us whether a particular test runs the ECS schedule. Follow the call:
`tick(&mut world)` asks the installed systems to act; `move_one_cell(...)` asks
one function to calculate and apply a step.

One passing helper test therefore fits with Fern still standing still in the
browser. The two prepared world-level movement tests remain explicitly ignored
until review; **ignored means skipped, not passed**. The agent handles
[reviewed activation](../today-v2.md#5-follow-your-helper-into-the-world),
enables those tests and checks the browser journey. You do not need to unignore
them merely to make the helper checkpoint green.

On the short-session path, use the live command prepared for your current edit;
the reference runner checks only the answer printed in the guide. When asking
for review, name the test, its result and what you changed. If it failed, include
the first diagnostic with its location. That gives us a useful starting point
without requiring you to diagnose the entire project first.
