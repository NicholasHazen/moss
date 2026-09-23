# 09. Give Meadow a way to recover

[Path home](README.md) · [Previous: a fair comparison](08-a-fair-comparison.md) · [Next: growth meets meals](10-growth-meets-meals.md)

**Future session · 20–30 minutes.** Your edit is a bounded `grow_biomass`
helper, planned in `crates/moss-sim/src/lessons.rs`. Before activation, the agent
prepares its stub and focused test, plus typed patch growth settings with a
capacity of 100 biomass units and a rate of 1 unit per tick. Neither those
settings nor scheduled growth exists in the current browser. Today's movement
exercise remains unchanged.

## Reconnect with the world

Fern can take something away from Meadow. We now want Meadow to recover a little
between visits. Recall how a meal worked: compute the amount permitted by every
limit, then apply exactly that amount to both participants. Growth reuses the
same pattern with one participant and one source: the environment supplies new
biomass, while the patch's capacity limits how much it can retain.

Meadow is a stand of grass represented by one entity. Increasing its biomass
does not create neighboring patches, alter its footprint, or change its species.
Those would be separate rules. A cellular automaton, which updates cells using
their neighbors, could eventually describe spread. Local recovery needs no
neighbor lookup, so we can learn this rule without also learning a grid engine.

## Work out what actually fits

Suppose Meadow holds 99 units and its capacity is 100. A request for
3 can retain only 1, producing 100. On the next tick the same request produces
nothing. This focused boundary uses a larger request than the default rate of 1
so it can expose partial capacity; the configured request and actual outcome
differ. Returning that outcome lets the inspector and later accounting report
what happened without reconstructing it from a configured rate.

This is the same distinction you used for travel: a cost per cell is a setting;
distance actually moved determines the charge. Here capacity and growth rate
are settings, while actual growth is a result. Keeping these meanings separate
makes later lighting changes smaller. Darkness will change the requested rate,
and the capacity rule will continue to protect the patch.

Rust's `&mut FoodPatch` gives the function temporary permission to change the
caller's patch. The helper owns neither the entity nor the world. Its small
signature makes the rule testable with an ordinary value, exactly as you tested
energy transfer before putting it inside an ECS system.

## Make one bounded change

Implement `grow_biomass` using remaining capacity to bound the request. The
agent will put the live test command beside that helper when preparing this
session. Its first check starts Meadow at 99 with capacity 100 and requests 3:
expect a return value of 1 and biomass 100. Calling again must return 0. If
biomass stays at 100 but the call reports 3, check whether you returned the
requested amount instead of the amount retained.

The **optional worked reference** below includes the complete helper and its
test. It uses the existing `FoodPatch`; it is not another module to paste into
production code. [Checking your work](README.md#checking-your-work) explains
how the live checkpoint differs from running this printed answer.

<!-- runnable: session-09 -->
```rust
use moss_sim::FoodPatch;

fn grow_biomass(patch: &mut FoodPatch, capacity: u32, requested: u32) -> u32 {
    assert!(patch.biomass <= capacity, "biomass exceeds capacity");
    let actual = requested.min(capacity - patch.biomass);
    patch.biomass = patch.biomass.checked_add(actual).expect("biomass overflow");
    actual
}

#[test]
fn session_09_growth_reports_only_what_fits() {
    let mut meadow = FoodPatch {
        name: "Meadow",
        biomass: 99,
    };
    assert_eq!(grow_biomass(&mut meadow, 100, 3), 1);
    assert_eq!(meadow.biomass, 100);
    assert_eq!(grow_biomass(&mut meadow, 100, 3), 0);
    assert_eq!(meadow.biomass, 100);

    meadow.biomass = 95;
    assert_eq!(grow_biomass(&mut meadow, 100, 3), 3);
    assert_eq!(meadow.biomass, 98);

    meadow.biomass = 0;
    assert_eq!(grow_biomass(&mut meadow, 100, 1), 1);
    assert_eq!(meadow.biomass, 1);
    assert_eq!(grow_biomass(&mut meadow, 100, 0), 0);
    assert_eq!(meadow.biomass, 1);
}
```

The assertion at entry distinguishes an invalid starting patch from an ordinary
full patch. Silently clamping an already invalid value would hide a setup bug.
The agent must validate or construct growth settings consistently before the
system calls this helper. Intentional saturation and error concealment can look
similar in arithmetic; their domain meanings decide which is appropriate.

## Keep a useful stopping point

An empty patch regrows because zero currently means cropped grass, not extinct
grass. That is a model choice, and the empty-patch assertion preserves it. Plant
death or seed availability can change the rule later through an explicit edit.
For now, being able to explain that one assertion is enough context to resume.

Stop after the helper test passes and ask: **“Review my bounded growth helper,
especially the reported amount and empty-patch case. Keep the system unscheduled
until the next integration checkpoint.”** The next session lets Fern eat during
the same tick that Meadow grows. We will use a tiny energy account to make the
order visible instead of trusting that two correct helpers automatically compose.
