# 3. Choose nearby food without moving yet

[Guide home](README.md) · Previous: [populations](02-populations.md) · Next: [movement](04-movement.md)

**Future chapter; preparation follows Chapter 2's review.** The helper and
activity data below are not installed. See [verification](authoring/verification.md)
for which worked examples were tested separately.

Fern can now spend energy, but the loss does not make anything happen. Meadow
could be nearby or across the chamber and the maintenance loop would behave
the same way. The first step toward feeding is to turn the animal's state and
nearby food into a choice we can inspect.

We will give a hungry grazer a target while leaving its position unchanged.
That makes a useful boundary: if the inspector names the wrong patch, we can
investigate the choice without also wondering whether movement or eating caused
it. The first edit is an ordinary Rust function; then we connect that function
to the ECS system and check a complete tick.

> Prepare Chapter 3 with a hungry-grazer fixture, explicit activity/target data and inspector fields. Leave target selection for me. Keep movement and eating unscheduled, and reconcile this guide with the current code first.

## On this page

- [The small rule](#the-small-rule)
- [A: a test for selection](#checkpoint-a--a-test-for-selection)
- [B: choosing becomes an ECS behavior](#checkpoint-b--choosing-becomes-an-ecs-behavior)
- [Optional: why a tie needs a rule](#optional-why-a-tie-needs-a-rule)

## The small rule

Among nonempty grass patches within a bounded Manhattan distance, choose the
nearest. Manhattan distance counts cardinal grid steps: horizontal distance
plus vertical distance. Equal distance chooses the lowest `SimId`, independent
of spawn or query order. No eligible food returns `None`.

For the first choice-only experiment, use a diagnostic radius of 10 cells:
Fern at (10, 10) is 9 steps from Meadow at (16, 13). A smaller radius would make
that existing patch invisible. This is an authored demonstration setting,
not a claim about real hare vision.

Hunger and eligibility belong in the system around this helper. Proposed
initial settings are: begin seeking below 45 reserve, stop at 75, and preserve
the previous seeking state between those thresholds. These are energy units
for the current capacity-100 animals. Maintenance runs first, so the decision
uses the reserve after that tick's passive cost. Foxes do not become grass eaters.

Using separate start and stop thresholds gives the animal a reason to continue
an activity. With one threshold, a small meal could make it stop, the next
maintenance cost could make it start, and the activity label could alternate
every tick. The gap gives the previous state a defined role. This technique is
often called *hysteresis*: the same current reserve can produce a different
decision depending on whether the animal was already seeking.

## Checkpoint A — a test for selection

During preparation I will add the helper signature below to `lessons.rs` with
a `todo!()` body, retaining the existing `choose_food` system stub for integration.
I will create `crates/moss-sim/tests/foraging.rs` with the required imports.
Its small input is a slice of `(stable ID, position, available biomass)` tuples;
the ECS adapter will supply grass patches only.

<!-- example: choice-test -->
```rust
use moss_sim::{Position, SimId, lessons::nearest_food};

#[test]
fn nearest_food_is_local_nonempty_and_stable() {
    let origin = Position { x: 2, y: 2 };
    let mut patches = vec![
        (SimId(9), Position { x: 1, y: 2 }, 80),
        (SimId(4), Position { x: 3, y: 2 }, 80),
        (SimId(3), Position { x: 4, y: 2 }, 80), // Eligible, but farther away.
        (SimId(1), origin, 0), // Closest, but empty.
        (SimId(2), Position { x: 7, y: 2 }, 80), // Outside radius.
    ];

    assert_eq!(nearest_food(origin, &patches, 2), Some(SimId(4)));
    patches.reverse();
    assert_eq!(nearest_food(origin, &patches, 2), Some(SimId(4)));
    assert_eq!(nearest_food(origin, &patches, 0), None);
    assert_eq!(nearest_food(origin, &[], 2), None);
}
```

Run after preparation:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test foraging nearest_food_is_local_nonempty_and_stable -- --exact
```

**Red:** one test reaches the unfinished helper and fails. **Green:** it passes
once your selection loop handles distance, availability and ties.

Here is a complete worked implementation. Add `Position` and `SimId` to the
existing `use crate::{...};` imports in `lessons.rs`; don't replace earlier imports.

<!-- example: choice-helper -->
```rust
pub fn nearest_food(
    origin: Position,
    patches: &[(SimId, Position, u32)],
    radius_cells: u32,
) -> Option<SimId> {
    let mut best: Option<(u32, SimId)> = None;

    for &(id, position, biomass) in patches {
        if biomass == 0 {
            continue;
        }
        let distance = origin.x.abs_diff(position.x)
            .checked_add(origin.y.abs_diff(position.y))
            .expect("grid distance overflow");
        if distance > radius_cells {
            continue;
        }

        let candidate = (distance, id);
        if best.is_none_or(|previous| candidate < previous) {
            best = Some(candidate);
        }
    }

    best.map(|(_, id)| id)
}
```

`&[...]` borrows a slice; it does not take ownership of the caller's vector.
`for &(...)` copies each small tuple out of its reference because all three
values are `Copy`. `continue` skips this iteration. Rust compares tuples from
left to right, so `(distance, id)` makes distance primary and stable ID the tie rule.

`|previous| ...` is a closure, a small unnamed function. `is_none_or` accepts the
first candidate or one better than the previous best. The final `map` turns
`Some((distance, id))` into `Some(id)` and leaves `None` alone. We could write
both using `match`; no iterator tricks are required to understand the rule.
The optional [Option method reference](https://doc.rust-lang.org/std/option/enum.Option.html#method.is_none_or)
shows `is_none_or` and `map` in isolation.

**Send:**

> The Chapter 3 selection test is green. Review the loop and tie rule, then help me connect it to choose_food. Do not add movement yet.

## Checkpoint B — choosing becomes an ECS behavior

This is another prepared pairing session. I will turn the acceptance cases
below into a concrete fixture and one next edit before you begin; they are not
one large assignment to implement all at once. Start with hungry versus idle,
then add target invalidation and the remaining boundary cases through review.

I will prepare the query/resource parameters and activity component. Together
we fill in the system: check grazer eligibility and hunger, gather eligible
patches, call the helper and write the selected stable target. Missing or empty
targets must be cleared or replaced; target IDs are resolved in the current run.
The agent wires the completed system before `complete_tick`, after maintenance.

The installed-schedule regression must prove:

| Before a tick, maintenance = 1 | After the tick |
| --- | --- |
| Idle hare at 45, nearby food | Reserve 44; seeking that patch; position unchanged. |
| Idle hare at 60 | Reserve 59; remains idle. |
| Seeking hare at 60 | Reserve 59; remains seeking with valid nearby food. |
| Seeking hare at 76 | Reserve 75; stops seeking. |
| Hungry hare, no eligible food | No target; no food or energy invented. |
| Hungry fox beside grass | No grass target. |

The adapter tests also reverse patch spawn order and confirm the same stable
target. Calling choice directly twice must not spend energy or change position;
the installed tick still charges its one maintenance cost. Record actual target
changes without filling the bounded journal with an identical event every tick.

**Browser:** let the diagnostic hare drop below the seeking threshold. The
inspector should show its activity and target, while its position and Meadow's
biomass stay fixed. Selection by the user does not select food for the animal.

Finish only when helper and installed-schedule tests pass and this browser
observation works. Review and stop before [movement](04-movement.md).

## Optional: why a tie needs a rule

If two patches are equally near, either could make sense for the animal. Why
choose the lower ID? It gives this version a repeatable answer that survives a
change in spawn order. We can later choose randomness deliberately and control
its seed; relying on storage traversal order would make the choice accidental.

The worked test already gives you a check: reversing the input leaves the
answer at patch 4. If we moved patch 9 onto the origin while leaving it nonempty,
patch 9 should win because distance is compared before ID. You can reason that
out from `(distance, id)` without adding another feature.

[Guide home](README.md) · Previous: [populations](02-populations.md) · Next after review: [movement](04-movement.md)
