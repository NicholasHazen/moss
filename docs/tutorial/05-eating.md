# 5. Transfer a finite meal

[Guide home](README.md) · Previous: [movement](04-movement.md)

**Future chapter: review movement first.** Now the animal can reach food. The
next rule transfers actual biomass into actual energy, bounded by what exists
and what fits. It does not replenish the patch or remove animals at zero.

> Prepare Chapter 5 with same-cell meal fixtures, an authored bite limit and stable conflict ordering. Leave the finite transfer for me. Keep plant growth and death separate, and update the guide against our current code.

## The units and the boundary

Use an explicit first conversion of **one biomass unit to one energy unit**.
Use a proposed bite limit of 4 biomass units per eating action, introduced in
the species feeding settings when this action is prepared. Biomass and energy
remain distinct quantities even when their demonstration conversion is 1:1.

At reserve 98 / capacity 100, a patch with 5 biomass and bite limit 4 can give
only **2** useful energy. The animal ends at 100; the patch ends at 3. We will
not discard extra food just because the nominal bite is larger than the room.

There is no separate extra energy charge for eating in this first model. Add a
named action cost only when we deliberately choose that rule. Maintenance already
runs each tick; this does not make activity selection itself an energy expense.

## Checkpoint A — a bounded transfer

The agent prepares an `eat_from_patch` helper stub in `lessons.rs` and
`crates/moss-sim/tests/eating.rs`. This helper assumes the calling system has
established eligibility/contact. It changes the actual reserve and biomass and
returns biomass consumed.

<!-- example: eating-test -->
```rust
use moss_sim::{Energy, FoodPatch, lessons::eat_from_patch};

#[test]
fn meals_respect_capacity_bite_size_and_shared_food() {
    let mut energy = Energy { reserve: 98, capacity: 100 };
    let mut patch = FoodPatch { name: "test grass", biomass: 5 };

    assert_eq!(eat_from_patch(&mut energy, &mut patch, 4), 2);
    assert_eq!(energy.reserve, 100);
    assert_eq!(patch.biomass, 3);
    assert_eq!(eat_from_patch(&mut energy, &mut patch, 4), 0);
    assert_eq!(patch.biomass, 3); // A full animal removes no food.

    let mut first = Energy { reserve: 0, capacity: 100 };
    let mut second = Energy { reserve: 0, capacity: 100 };
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

Run after preparation:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test eating meals_respect_capacity_bite_size_and_shared_food -- --exact
```

**Expected:** one test, red before the transfer is implemented and green after.
There is no maintenance in this helper test. Its purpose is to isolate the
transfer boundary; the installed test below includes the tick's earlier work.

Worked implementation, with `FoodPatch` added to the existing `lessons.rs` imports:

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

Each `.min(...)` imposes one upper bound: free capacity, actual food, then bite
size. `saturating_sub` makes free capacity zero even if a malformed input already
exceeds capacity; the helper does not silently repair that reserve. Biomass
subtraction cannot underflow because `consumed` is bounded by current biomass.
Returning the actual amount gives the caller evidence for an outcome event.

**Send:**

> The Chapter 5 transfer test is green. Review the units and boundaries. Help me finish the same-cell eating system and deterministic competition test, then stop before growth or starvation.

## Checkpoint B — two mouths, one patch

This is a separate prepared pairing session: first one eligible eater, then
same-cell rejection, then two eaters. I will supply the installed fixtures and
one exact next edit at each review.

**Agent test preparation:** both maintenance regressions currently run the whole
schedule in a world containing food. Once eating is scheduled, regaining energy
would legitimately invalidate their zero-after-63-ticks expectation. Give those
tests a no-food setup, keeping the real `install` + `tick` path and their existing
60 → 57 (or 54) → 0 expectations. Do not weaken the assertions or disable the new
systems globally. Dedicated foraging tests exercise the food-containing world.

Together we finish `eat_food`: require a seeking grazer, its accepted live grass
target, and exact same-cell contact. The agent prepares iteration by ascending
`SimId`; we recheck and mutate the shared patch for each eater in that order.
Do not give both eaters a reward calculated from the original five units.
Do not rely on query order or deferred despawn to reserve food.

For the first model, the empty patch remains with biomass 0. Choice can ignore
it next tick; later renewal can replenish it. No growth is installed here.
Record only successful meals with participants and actual biomass/energy amounts;
a desired bite of 4 is not an actual meal of 4 when only 1 remains.

The helper's sequential calls prove shared-food arithmetic, but **not ECS
ordering**. The installed regression must create two hungry grazers on one patch
with 5 biomass, both at reserve 0, bite size 4, and passive rates explicitly set
to 0 for this conflict experiment. After one tick, the lower-ID eater has 4,
the other 1, and the patch 0. Reverse the spawn order while keeping IDs and
starting conditions fixed; the same individual must receive the first bite.

Also test a normal installed tick: a seeking hare starts at 60 on the patch,
maintenance = 1 and bite = 4. Choice preserves seeking at 59, same-cell movement
costs 0, and a meal from biomass 5 leaves **energy 63, biomass 1**. Being on the
cell does not permit an idle/satisfied animal or a fox to eat grass. A target
one cell away cannot be consumed by the eating system before contact.

## Browser checkpoint and stopping point

The agent will prepare a nearby-food diagnostic that the hare can reach before
it runs out of energy. Step to contact, inspect both animal and patch, and compare
the actual changes with the meal event. With two grazers, total food removed must
equal the total meal energy awarded under the 1:1 rule; maintenance and movement
still subtract their own costs. Check that a satisfied grazer stops consuming
after choice applies the stop threshold, and that an exhausted patch cannot feed
anyone again. The 98 → 100 capacity boundary is isolated in the helper test;
ordinary seeking stops well before a grazer reaches full capacity.

The completed order is maintenance → choice → movement → eating → complete tick,
with only implemented systems scheduled. Choice sees the preceding reserve;
after eating reaches the stop threshold, it stops seeking on the next tick if
the post-maintenance reserve still meets that threshold. This ordering belongs
in the integration test and inspector explanation.

After native checks, browser verification and review, stop. The recommended next
chapter will be **bounded grass renewal under constant light**, followed by
day/night as its own small change. We should first be able to explain one finite
meal without any growth hiding where the food came from.
