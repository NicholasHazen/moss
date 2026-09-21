# 4. Pay for the distance actually traveled

[Guide home](README.md) · Previous: [food choice](03-food-choice.md) · Next: [eating](05-eating.md)

**Future chapter; complete and review food choice first.** The step helper was
tested separately; the installed movement system is still future work. See
[the verification record](authoring/verification.md).

Fern has selected Meadow, but selecting a destination has not changed her cell.
Now the decision needs an executor: a rule that takes an accepted step and
updates the world. This is also the point where we know whether travel actually
happened, so it is where we can charge for it.

The first movement is one cardinal cell per tick, x before y, with no obstacles
or body collision. The constraint gives us an explainable route and a distance
of either zero or one. We can learn the relationship between movement and cost
before adding speed variation or pathfinding.

> Prepare Chapter 4 with species travel rates, a nearby-food fixture and query/schedule plumbing. Leave the affordable-step rule for me. Keep eating unscheduled and update this guide to match our current types.

## On this page

- [Where costs belong](#where-costs-belong)
- [A: an affordable step](#checkpoint-a--an-affordable-step)
- [B: move toward the accepted target](#checkpoint-b--move-toward-the-accepted-target)
- [Optional: a journey that ends where it started](#optional-a-journey-that-ends-where-it-started)

## Where costs belong

The agent will extend the species resource with named movement units per cell,
alongside passive units per tick. The worked test uses a travel rate of 2.
Selecting a target does not pay it. The movement executor knows the old position
and accepted destination; it computes actual distance and applies the movement
and charge together.

Maintenance runs before movement. With reserve 60, passive cost 1 and an accepted
one-cell move costing 2, the complete tick leaves **57**. If maintenance leaves
only 1, the animal cannot buy that move. Saturating subtraction is appropriate
for baseline maintenance, but would wrongly permit unaffordable movement here.

No persistent distance component is necessary for this single step. Later travel
history can record actual outcomes. For several segments, charge their summed
lengths; returning to the starting cell does not erase the journey's cost.

## Checkpoint A — an affordable step

The agent will prepare a `move_one_cell` helper stub in `lessons.rs` and
`crates/moss-sim/tests/movement.rs`. It accepts mutable position/energy references,
a target position, the configured travel rate and validated world dimensions.
It returns actual cells traveled, either zero or one.

<!-- example: movement-test -->
```rust
use moss_sim::{Energy, Position, WorldConfig, lessons::move_one_cell};

#[test]
fn movement_charges_only_an_affordable_actual_step() {
    let mut position = Position { x: 2, y: 2 };
    let mut energy = Energy { reserve: 59, capacity: 100 };
    let target = Position { x: 4, y: 4 };
    let config = WorldConfig::default();

    // Maintenance has already reduced 60 to 59 for this helper example.
    assert_eq!(move_one_cell(&mut position, &mut energy, target, 2, config), 1);
    assert_eq!(position, Position { x: 3, y: 2 }); // x first, no diagonal.
    assert_eq!(energy.reserve, 57);

    energy.reserve = 1;
    assert_eq!(move_one_cell(&mut position, &mut energy, target, 2, config), 0);
    assert_eq!(position, Position { x: 3, y: 2 });
    assert_eq!(energy.reserve, 1);

    energy.reserve = 10;
    let current = position;
    assert_eq!(move_one_cell(&mut position, &mut energy, current, 2, config), 0);
    let outside = Position { x: -1, y: 2 };
    assert_eq!(move_one_cell(&mut position, &mut energy, outside, 2, config), 0);
    assert_eq!(position, current);
    assert_eq!(energy.reserve, 10);
}
```

Run after preparation:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test movement movement_charges_only_an_affordable_actual_step -- --exact
```

**Expected:** one red test while the helper is unfinished, then one green test.
The literals check position and energy independently; a correct return value
alone would not prove the effect happened.

Worked implementation, with `Energy`, `Position`, `WorldConfig` in the existing
`lessons.rs` imports:

<!-- example: movement-helper -->
```rust
pub fn move_one_cell(
    position: &mut Position,
    energy: &mut Energy,
    target: Position,
    units_per_cell: u32,
    config: WorldConfig,
) -> u32 {
    let in_bounds = |p: Position| {
        (0..config.width() as i32).contains(&p.x)
            && (0..config.height() as i32).contains(&p.y)
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

    let distance = position.x.abs_diff(next.x)
        .checked_add(position.y.abs_diff(next.y))
        .expect("step distance overflow");
    let cost = distance.checked_mul(units_per_cell)
        .expect("movement cost overflow");
    if energy.reserve < cost {
        return 0;
    }

    *position = next;
    energy.reserve -= cost;
    distance
}
```

`let mut next = *position` copies the current cell into a tentative destination.
It does not move the animal yet. `*position = next` writes through the mutable
reference only after affordability succeeds. The proposed step stays between
two valid grid coordinates, so it stays in bounds. `checked_mul` makes overflow
an explicit error instead of silently producing a cheap travel cost.
See the optional [checked multiplication reference](https://doc.rust-lang.org/std/primitive.u32.html#method.checked_mul).

**Send:**

> The Chapter 4 step test is green. Review bounds, affordability and the mutation order. Help me wire the real target into move_to_food and verify a complete tick before adding eating.

## Checkpoint B — move toward the accepted target

I will prepare this as a separate pairing checkpoint with a real installed
fixture and one next edit. We will review one accepted step before adding the
missing-target and affordability boundaries.

Together we fill in `move_to_food`: only an eligible seeking grazer with a live,
nonempty food target attempts a step. Re-resolve the stable target ID now; a
previous choice is not proof that the target still exists. The agent handles
disjoint Bevy queries and chains maintenance → choice → movement → complete tick.

One Bevy pitfall belongs to that plumbing: animal `&mut Position` and patch
`&Position` queries can conflict even with different `With` filters. The agent
will add an explicit exclusion such as `Without<Creature>` for patches; different
component names alone do not guarantee disjoint queries.

The installed test starts a **seeking** hare at 60 (between the existing hunger
thresholds), passive rate 1, travel rate 2, and nearby nonempty grass. One tick
must move exactly one cell and leave 57. A low-reserve case starting at 2 leaves
1 after maintenance and must stay still. A no-food case pays only maintenance.
If a different valid patch is selected, travel toward that replacement can cost
energy; a stale target itself must never create a charge.

Add focused edge cases while reviewing: y movement after x aligns, all four
directions, exact affordability, and valid edge cells. The helper example above
is an initial checkpoint, not exhaustive coverage of the installed system.

**Browser:** Step a seeking hare toward nearby grass, checking old/new coordinates
and reserve. Pause and camera motion change neither. On the target cell it stops
paying travel, but maintenance continues. Biomass stays unchanged: eating is the
next chapter. Zero-energy animals still exist.

Review and stop once the installed test and browser agree.

## Optional: a journey that ends where it started

Suppose a later action moves from (2, 2) to (3, 2), then back to (2, 2). The
final displacement is zero, but the distance traveled is two cells. At 2 units
per cell, travel should cost 4, before counting any passive maintenance.
That is why a future multi-segment action must sum accepted segment lengths.

For the current one-cell action, old and new positions are enough. Record that
actual outcome and we can later explain a travel total without asking the camera
or a policy evaluation how far the animal went. [The tick walkthrough](context/a-tick-through-moss.md)
shows where authoritative state and its presentation part ways.

[Guide home](README.md) · Previous: [food choice](03-food-choice.md) · Next after review: [eating](05-eating.md)
