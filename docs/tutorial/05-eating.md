# 5. Transfer a finite meal

[Guide home](README.md) · Current session: [movement v2](today-v2.md) · Short session: [a bounded meal](path/01-bounded-meal.md)

**Future: today's stretch after reviewed movement.** The agent must first prepare
`eat_from_patch`, its test and the same-cell adapter. Those APIs are not ready to
call in the live project. The worked answer below is available for learning;
[verification](authoring/verification.md) records its isolated evidence separately
from pending installed-system and browser checks.

Fern can reach Meadow, but contact alone has not made a meal. Energy keeps
falling while the patch keeps its biomass. The next rule transfers some of that
finite food into Fern's reserve. A full animal takes nothing; a nearly empty
patch supplies only what remains.

**Start after movement review:** the agent prepares the helper and test, then
you implement only `eat_from_patch` and run checkpoint A. Keep the authored
`FoodTarget`; no hunger thresholds, autonomous food choice or activity system
is needed for this first meal. Reaching and consuming finite food is the session's
stretch result, not a requirement for the movement session to count as complete.

This chapter is the integration companion for two short sessions. Checkpoint A
is [session 01's bounded helper](path/01-bounded-meal.md); checkpoint B connects
it to the world, as [session 02](path/02-a-shared-meal.md) explains. They cover
the same outcomes at different session lengths. If this stretch completes both,
keep the implementation and its tests and resume at
[finding food](path/03-finding-food.md).

## On this page

- [The units and the boundary](#the-units-and-the-boundary)
- [A: a bounded transfer](#checkpoint-a--a-bounded-transfer)
- [B: two mouths, one patch](#checkpoint-b--two-mouths-one-patch)
- [Browser checkpoint and stopping point](#browser-checkpoint-and-stopping-point)
- [Optional: the meal and the net change](#optional-the-meal-and-the-net-change)

## The units and the boundary

Use an explicit first conversion of **one biomass unit to one energy unit**,
with an authored bite limit of **4 biomass units per eating action**. These are
small demonstration settings. Biomass and energy remain distinct quantities,
even when the conversion happens to be 1:1. A shared feeding setting can supply
the bite limit without changing the maintenance configuration today.

At reserve 98 and capacity 100, a patch with 5 biomass can give only **2** useful
energy. Even though the bite limit is 4, Fern ends at 100 and the patch ends at 3.
There are three limits on the transfer: room in the animal, food in the patch and
the bite size. The smallest wins.

There is no additional eating cost in this first model. Maintenance still runs
each tick, and travel still charges for actual movement. The meal changes both
energy and biomass in one operation so the next eater sees the real remainder.

## Checkpoint A — a bounded transfer

**Preparation is still required.** The agent adds an `eat_from_patch` stub to
[`lessons.rs`](../../crates/moss-sim/src/lessons.rs), imports `FoodPatch` there,
and creates `crates/moss-sim/tests/eating.rs`. The helper assumes its caller has
already established same-cell contact with eligible food. It takes mutable
borrows of the animal's energy and the patch, and returns biomass consumed.

The following is the **complete future test**, to be installed by the agent
when this checkpoint becomes active. It includes all imports for `eating.rs`.

<!-- example: eating-test -->
```rust
use moss_sim::{Energy, FoodPatch, lessons::eat_from_patch};

#[test]
fn meals_respect_capacity_bite_size_and_shared_food() {
    let mut energy = Energy {
        reserve: 98,
        capacity: 100,
    };
    let mut patch = FoodPatch {
        name: "test grass",
        biomass: 5,
    };

    assert_eq!(eat_from_patch(&mut energy, &mut patch, 4), 2);
    assert_eq!(energy.reserve, 100);
    assert_eq!(patch.biomass, 3);
    assert_eq!(eat_from_patch(&mut energy, &mut patch, 4), 0);
    assert_eq!(patch.biomass, 3); // A full animal removes no food.

    energy.reserve = 0; // Arrange a hungry animal for the bite-limit cases.
    assert_eq!(eat_from_patch(&mut energy, &mut patch, 0), 0);
    assert_eq!(energy.reserve, 0);
    assert_eq!(patch.biomass, 3);
    assert_eq!(eat_from_patch(&mut energy, &mut patch, 2), 2);
    assert_eq!(energy.reserve, 2);
    assert_eq!(patch.biomass, 1);

    let mut first = Energy {
        reserve: 0,
        capacity: 100,
    };
    let mut second = Energy {
        reserve: 0,
        capacity: 100,
    };
    patch.biomass = 5;
    assert_eq!(eat_from_patch(&mut first, &mut patch, 4), 4);
    assert_eq!(eat_from_patch(&mut second, &mut patch, 4), 1);
    assert_eq!(first.reserve, 4);
    assert_eq!(second.reserve, 1);
    assert_eq!(patch.biomass, 0);
    assert_eq!(eat_from_patch(&mut second, &mut patch, 4), 0);
    assert_eq!(second.reserve, 1);
}
```

**Run after preparation, from the repository root:**

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test eating meals_respect_capacity_bite_size_and_shared_food -- --exact
```

**Expected:** exactly one test fails at the unfinished helper, then reports
`1 passed; 0 failed` after implementation. There is no maintenance in this test:
it isolates the transfer. The second half passes the same patch to two eaters,
so their total gain must equal its loss. Zero matching tests or a missing test
target means preparation is incomplete.

The explicit reserve reset arranges another test input; it is not a simulation
action. With room for 100 and biomass 3, a bite limit of 2 must now determine the
transfer. A function that always assumes the usual bite of 4 would take 3 and
fail this case. The zero-bite call checks that an explicit zero remains zero.

**Complete future worked replacement** for `eat_from_patch` in `lessons.rs`,
with `Energy` and `FoodPatch` in that file's imports:

<!-- example: eating-helper -->
```rust
pub fn eat_from_patch(
    energy: &mut Energy,
    patch: &mut FoodPatch,
    bite_limit: u32,
) -> u32 {
    let room = energy.capacity.saturating_sub(energy.reserve);
    let consumed = room.min(patch.biomass).min(bite_limit);
    energy.reserve = energy.reserve.checked_add(consumed)
        .expect("meal energy overflow");
    patch.biomass -= consumed;
    consumed
}
```

This reuses the same pattern as movement: work out an allowed change, then apply
it to the borrowed values. Here `.min(...)` expresses each upper bound directly.
`room` is zero for a full animal, so the transfer is zero without a separate
full-animal branch. `consumed` is also bounded by current biomass, which makes
the later subtraction safe.

The room calculation uses saturating subtraction because free capacity cannot
be negative. If malformed input already exceeds capacity, the helper awards no
food; it does not silently repair that reserve. The checked addition reports an
unexpected overflow rather than wrapping the energy value.

**If the result differs:** a patch ending at 1 instead of 3 in the first example
means the code removed the nominal bite instead of the actual transfer. Use the
same `consumed` value for both changes. Returning 2 without changing both values
is not enough to satisfy the test.

**Observed evidence:** see [verification](authoring/verification.md) for any
isolated worked-answer execution. The helper, installed meal system and browser
behavior remain future live work until this chapter is activated and reviewed.

**Send for review:**

> The Chapter 5 transfer test is green. Review the units and boundaries, then
> finish the same-cell adapter, competition regression and browser verification.
> Stop before food choice, growth or starvation.

## Checkpoint B — two mouths, one patch

The agent prepares and verifies the ECS adapter after the helper review. It
requires a grazer, its live nonempty grass target and exact same-cell contact.
An animal a cell away cannot eat that patch, and Flint's hunter role does not
permit grazing. Capacity alone bounds eating for this first authored-target
model; hunger thresholds and deciding to leave a patch are later behaviors.

Shared food needs an explicit winner when requests compete. The agent prepares
iteration by ascending `SimId` and rechecks the patch before each transfer.
The first hare can take 4 from a patch with 5; the second then sees 1. Both
cannot receive 4 based on the original snapshot. Query iteration order and
queued removal are not reservations of food.

The helper test checks sequential arithmetic, but **not ECS ordering**. The
installed regression must create two grazers on one patch with biomass 5,
reserves 0, bite limit 4 and maintenance rates explicitly set to 0 for this
experiment. After one tick the lower-ID eater has 4, the other has 1, and the
patch has 0. Reversing spawn order with the same IDs must preserve that outcome.
The agent owns this test plumbing; it is not an extra implementation assignment.

An empty patch remains in the world with biomass 0. It cannot supply another
meal, and it does not regrow during this chapter. A successful meal records
participants and the actual transfer. A requested bite of 4 is not an actual
meal of 4 if only 1 was available.

The agent also verifies a complete tick that starts Fern on Meadow's cell with
reserve 33 and biomass 80. Maintenance leaves 32; same-cell movement costs 0;
the meal adds 4. **Expected: reserve 36, biomass 76.** That test distinguishes
the helper's transfer from the complete schedule's net energy change.

## Browser checkpoint and stopping point

The agent installs maintenance → movement → eating → complete tick only after
reviewing the helper and adapter. For a direct meal observation, prepare the
same-cell state above, Step once, then inspect both participants. Fern's reserve
must become 36 and Meadow's biomass 76, with an actual meal of 4 recorded.
This is an acceptance target; it has not been observed in the live browser yet.

For the original route from Reset, eating now runs on the arrival tick too.
After nine movement steps, Fern reaches Meadow and takes her first meal in that
same tick: reserve becomes **37**, rather than the movement-only chapter's 33,
and biomass becomes **76**. That difference is expected because a new rule has
joined the schedule; camera actions still have no biological effect.

Stop when the native checks, browser observation and review agree. There is now
a visible consequence to reaching food, and its supply really decreases. The
next paired step is to replace the authored destination with a small
[food-selection rule](path/03-finding-food.md), followed by
[when to seek](path/04-when-to-seek.md). Together they give Fern a reason to
choose a destination, continue toward it or stop.
Plant renewal, day/night, individual attributes and populations can follow in
separate visible increments; none is required to explain this meal.

## Optional: the meal and the net change

The complete same-cell example starts at 33, spends 1 maintenance unit and gains
4 food units. Its reserve rises by 3 overall. An event saying “ate 3” would be
wrong: the meal supplied 4, while another rule spent 1. Recording each actual
outcome lets us later explain a creature's reserve without inventing a cause
from the final total.

The expected state, reserve 36 and biomass 76, resolves the opening problem:
contact helps Fern because a bounded transfer now occurs. The optional
[ecosystem context](context/from-meals-to-ecosystems.md) follows the later question
of where replacement food might come from.

[Guide home](README.md) · Current session: [movement v2](today-v2.md) · After review: [finding food](path/03-finding-food.md)
