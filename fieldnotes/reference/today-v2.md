# Today, v2: give Fern one affordable step

[Guide home](README.md) · [Current task](../../NOW.md) · [Original lesson](today.md)

Fern already loses energy when the world ticks. This session gives her somewhere
to go: she will walk to Meadow, pay for the distance she actually travels, and
stop there. The useful result is both visible and explainable: you can point to
the code that changed her cell and account for the energy it spent.

**Your one edit:** open [`lessons.rs`](../../crates/moss-sim/src/lessons.rs), find
`move_one_cell`, and replace its unfinished body. Its signature, caller and tests
are prepared. The complete answer is on this page; keep it beside your code and
use as much of it as helps. The [original lesson](today.md) remains available,
but you do not need to read both versions.

**Current state:** maintenance runs; movement is still unscheduled and its helper
contains `todo!()`. We will get the helper test green, review it together, then
I will activate and verify the browser journey. That is one complete session.
Plan for roughly 60–90 minutes, with a useful stopping point at the passing helper
if you need it. Builds and integration are my work, not extra exercises for you.

## On this page

- [1. Run the test before changing the rule](#1-run-the-test-before-changing-the-rule)
- [2. Follow one attempted step](#2-follow-one-attempted-step)
- [3. Write one affordable step](#3-write-one-affordable-step)
- [4. Check the edit and read the result](#4-check-the-edit-and-read-the-result)
- [5. Follow your helper into the world](#5-follow-your-helper-into-the-world)
- [6. Watch the journey and stop](#6-watch-the-journey-and-stop)

## 1. Run the test before changing the rule

In RustRover, open this lesson beside `lessons.rs` and select **Moss - Movement
test** in the Run menu. Use normal Run. The equivalent command, from
`/Users/nick/Code/moss`, is:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test movement movement_charges_only_an_affordable_actual_step -- --exact
```

**Before your edit, expect one failed test** with the message:

```text
not yet implemented: Paired movement exercise: propose, validate, then commit one step
```

This is a useful starting result. Rust compiled the code, the test found your
function, and the function stopped at its explicit placeholder. It did not return
an incorrect distance; it has not returned a distance at all. If the command
reports a missing target or zero matching tests, send me that output so I can
repair the setup before you spend time on the rule.

The prepared test lives in [`tests/movement.rs`](../../crates/moss-sim/tests/movement.rs).
You do not need to write a test harness today. Its first case supplies ordinary
position and energy values, calls your helper once, then checks three things:
distance returned, position afterward, and reserve afterward. Checking only the
return value would miss a function that says “one cell” without actually moving.

## 2. Follow one attempted step

The browser fixture starts Fern, a hare, at `(10, 10)` and gives her Meadow's ID
as an authored destination. Meadow is a grass patch at `(16, 13)`. She does not
choose food yet; today's rule carries out that existing instruction.

For the helper test, use the smaller coordinates in the diagram. Start at
`(2, 2)`, target `(4, 4)`, reserve **59**, and travel cost **2 energy units per
cell**. The test supplies 59 directly; maintenance is outside this function.

![A proposed step from 2,2 to 3,2 costs two energy. With reserve 59 it commits position 3,2 and reserve 57. An alternative start with reserve 1 keeps position 2,2 and reserve 1.](visuals/movement-step.svg)

First propose `(3, 2)`. We change x before y and move at most one cardinal cell
per call. The proposed distance is one, so its cost is `1 × 2 = 2`. Reserve 59
can pay: write the new position, subtract 2, and return **1 cell traveled**.

Now imagine that same attempt starting with only **1** energy. Moving and using
`saturating_sub(2)` would leave zero, but it would also grant a step the animal
could not afford. Our movement rule instead returns **0** and leaves both values
alone. This differs deliberately from maintenance, which already clamps reserves
at zero. Rust accepts either calculation; the game rule determines which one is
correct here.

That gives the function its shape: **propose a change, check it, then commit it**.
If the animal is already at its destination, the proposed distance is zero and
travel costs nothing. Invalid starting or target coordinates also leave the
caller’s position and energy unchanged.

## 3. Write one affordable step

Your helper receives temporary access to two values it may change:
`position: &mut Position` and `energy: &mut Energy`. The target and configuration
arrive as small values copied into the call. It returns a `u32`: the nonnegative
number of cells actually traveled, either zero or one under today's rule.

Start with this **reasoning excerpt from the helper body**:

```rust
let mut next = *position;
```

`position` refers to the caller's real position. `*position` accesses that value;
because `Position` implements `Copy`, assigning it to `next` makes an independent
copy. `mut next` lets us change this tentative position. Editing `next.x` does
not move Fern. The later `*position = next` is the write back to real state.
This is why rejecting a proposal needs no undo operation.

Write the body in that order, or use the complete answer below and trace it with
59 energy and then 1. Keep the rest of `lessons.rs` as it is. There is no loop
inside this helper: one call attempts one step. The ECS adapter will loop over
eligible animals; later simulation ticks will call it again for later steps.

### Complete worked replacement

The following is a **replacement for `move_one_cell` only** in
[`lessons.rs`](../../crates/moss-sim/src/lessons.rs). The file already imports
`Energy`, `Position` and `WorldConfig`. You may replace the entire function with
this block, or keep its matching signature and replace only the body. Remove
the placeholder's `let _ = (...)` and `todo!(...)` lines; do not append a second
function with the same name.

<!-- example: movement-v2-helper -->
```rust
pub fn move_one_cell(
    position: &mut Position,
    energy: &mut Energy,
    target: Position,
    units_per_cell: u32,
    config: WorldConfig,
) -> u32 {
    let in_bounds = |p: Position| {
        (0..config.width() as i32).contains(&p.x) && (0..config.height() as i32).contains(&p.y)
    };
    if !in_bounds(*position) || !in_bounds(target) {
        return 0;
    }

    let mut next = *position;
    if next.x != target.x {
        next.x += if next.x < target.x { 1 } else { -1 };
    } else if next.y != target.y {
        next.y += if next.y < target.y { 1 } else { -1 };
    }

    let distance = position
        .x
        .abs_diff(next.x)
        .checked_add(position.y.abs_diff(next.y))
        .expect("step distance overflow");
    let cost = distance
        .checked_mul(units_per_cell)
        .expect("movement cost overflow");
    if energy.reserve < cost {
        return 0;
    }

    *position = next;
    energy.reserve -= cost;
    distance
}
```

### Read the decisions in that answer

`in_bounds` is a small local function, called a closure. Its `|p: Position|`
names the argument. The range `0..config.width()` excludes the upper bound:
width 32 allows x coordinates 0 through 31. Coordinates use signed integers so
an invalid negative position can be represented and rejected. The `as i32`
conversions fit because `WorldConfig` validates dimensions in the range 8–256.

The `if` expression inside `next.x += ...` produces a value: **1** when the
target is to the right, otherwise **−1**. The `else if` is consequential: y changes
only when x is already aligned. Using two independent `if` statements could move
both axes in one call. From `(2, 2)` toward `(4, 4)`, that would violate the
test's expected position `(3, 2)`.

`abs_diff` measures the unsigned change on an axis. Adding the two axis changes
gives the proposed travel distance for this one-cell model. `checked_add` and
`checked_mul` return an `Option`; `expect` takes a valid result or reports an
unexpected arithmetic overflow with its message. They protect an assumption
about representable numbers. The separate affordability check protects the
animal's reserve. Those are different questions.

The first writes to the caller's values happen at the end, after the checks.
The final `distance` has no semicolon because its value is returned. `return 0;`
exits early on rejection; the final expression supplies the accepted result.

## 4. Check the edit and read the result

Save the file and run **Moss - Movement test** again, or rerun the same command
from section 1. **Your checkpoint is one named test passing.** Besides the first
step, that test checks rejection, arrival, both axes and directions, world edges,
a different configured rate and a reserve that pays the cost exactly.

The test's low-energy case comes after its first successful move: position is
already `(3, 2)`, then reserve is set to 1. That call must preserve `(3, 2)` and
1. Follow the fixture's current values when reading an assertion; each call does
not automatically reset the world or the ordinary variables around it.

| If you see this | The next useful check |
| --- | --- |
| The same `not yet implemented` panic | Check that the saved body of this `move_one_cell` replaced its `todo!()`. |
| `expected u32, found ()` | Look at the final expression. In this answer, `distance;` discards the return value; `distance` returns it. Other type errors may have other causes. |
| A failed position assertion only on low energy | Check whether you wrote `*position` before checking affordability. Returning 0 afterward cannot undo that write. |
| `0 passed; 0 failed` with tests filtered out | Check the exact test name and target. No selected test ran. |

For other output, send the first diagnostic and its source location. You do not
need to diagnose the whole project before asking for help. The optional
[test-reading guide](context/reading-a-test.md#find-out-how-far-the-run-got)
explains the difference between compilation, a panic and a failed comparison.

**Stop here for review.** A passing helper is already a useful contribution.
Send:

> The movement helper test passes. Review the bounds, borrowing and mutation
> order, then activate the prepared system and verify Fern's journey with me.

## 5. Follow your helper into the world

This section is the review walkthrough. You do not need another code edit before
seeing motion. A passing helper test checks a function call, while the browser
needs that call connected to real entities and an executed tick.

The prepared `move_to_food` in the same source file is that connection. An ECS
**query** asks the world for components from matching entities. Here is an
**existing query-parameter excerpt**, not code to add:

```rust
mut creatures: Query<(&FoodTarget, &mut Position, &mut Energy), With<Creature>>,
```

Read it as: find entities carrying `Creature`; read their `FoodTarget`; borrow
their `Position` and `Energy` so they can change. Fern has that set of components.
Flint has no authored `FoodTarget`, so this query skips him even though he has
position and energy. Query membership depends on component presence, not names.

![The prepared adapter matches Fern's target ID 3 to Meadow, borrows Fern's position and energy, copies Meadow's destination, and supplies the ordinary movement helper. The expected first result after activation is position 11,10 and reserve 57.](visuals/adapter-to-helper.svg)

The patch query reads Meadow's ID, position and biomass. The adapter resolves the
target against the current patch: a remembered ID alone is not proof food still
exists. Missing or empty food means no call and no travel charge. Meadow's
position is copied as the destination; Fern's own position and energy are
borrowed for mutation. Both belong to the same authoritative ECS world.

During review I will place this system between maintenance and tick completion,
enable the two prepared world-level tests, and run the integration checks. The
intended order is **maintenance → movement → complete tick**. Until that happens,
a green helper test can coexist with a stationary Fern: the live schedule still
runs maintenance alone.

The files now have concrete jobs: [`lessons.rs`](../../crates/moss-sim/src/lessons.rs)
contains your rule and its adapter; [`movement.rs`](../../crates/moss-sim/tests/movement.rs)
checks the helper and installed world;
[`simulation.rs`](../../crates/moss-sim/src/simulation.rs) chooses which systems
run. The browser draws their resulting state. A sprite transform or camera move
is not another source of biological position.

## 6. Watch the journey and stop

After review and rebuilding, I will verify the browser with you. **These are
expected results after activation, not observations of today's unfinished code.**
Start paused, Reset, select Fern, and use Step so each change has a visible tick.

| Executed ticks since Reset | Expected observation |
| --- | --- |
| 0 | Fern at `(10, 10)`, reserve 60. Flint reserve 60. Meadow biomass 80. |
| 1 | Fern at `(11, 10)`, reserve 57. Flint stays put with reserve 59. |
| 9 | Fern reaches `(16, 13)`, reserve 33. Flint reserve 51. Meadow still has 80 biomass. |
| 10 | Fern stays at Meadow, reserve 32. Flint reserve 50. Meadow still has 80 biomass. |

On the first tick, maintenance takes 60 to 59, then one traveled cell costs 2,
leaving 57. The route takes six x steps and three y steps:
`60 − 9 × 1 − 9 × 2 = 33`. On tick ten there is no new travel, so only maintenance
is charged. Standing at food does not consume it; eating has its own later rule.

That tenth Step is a useful contrast to the first nine. If it costs three units
again, inspect what the movement helper returned and where travel is charged.
Do not infer traveled distance from the fact that the animal still has a target.
While paused, pan and zoom should change the view while preserving these values.
I will check Reset, the controls and inspector as part of integration.

Stop when the reviewed helper, installed tests and browser agree on this journey.
If you have energy left, the recommended next edit is a
[finite meal](05-eating.md), prepared separately after movement review. It reuses
the same central idea: calculate what can actually happen before changing the
values that pay for it. I will update `NOW.md` to the reviewed stopping point so
your next visit starts there.

**Verification:** the [v2 evidence record](authoring/verification.md#todays-lesson-v2-september-22-2026)
separates the tested written answer from live activation and browser acceptance.
