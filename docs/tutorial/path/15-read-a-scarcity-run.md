# 15. Explain a population that cannot all survive

[Path home](README.md) · [Previous: removal and memory](14-removal-and-memory.md) · [Next: return and explain](16-return-and-explain.md)

**Future session · 20–30 minutes.** Your edit is a small population-account
check, planned in `crates/moss-sim/tests/scarcity.rs`, followed by one prepared
scenario observation. The agent supplies a deterministic fixture, bounded
samples and run counters before this session. It uses six hares and one patch
to isolate scarcity; this is a separate experiment from the general population
fixture with six hares, two foxes and four patches.

## Start with a prediction you can explain

Each hare now has an individual maintenance cost. Meals transfer finite biomass,
daylight supplies new biomass, and an accepted starvation rule may remove an
animal. Those are familiar local rules. Today we ask what they imply together,
before interpreting a population curve as success or failure.

With six hares each spending 1 energy unit per tick, maintenance alone requests
6 units per tick. One patch producing at most 1 biomass unit per tick cannot
replace that demand indefinitely under the chosen 1:1 conversion, even with
continuous daylight and perfect access. Night and travel can make the deficit
worse. Initial reserves and stored biomass can delay the consequence, and
deaths eventually reduce demand; they do not erase the original mismatch.

This is a useful prediction, but it does not specify who dies or on which tick.
Distance, target choice, stable conflict order, capacity and stored food affect
those outcomes. A species-wide total can identify an impossible long-term
budget in this unit-cost fixture without explaining an individual's entire
story. Each survivor has at least 1 reserve, so it can pay the next unit of upkeep.
For larger requested costs, clamping can deduct less than the nominal demand;
[the previous policy example](13-deciding-starvation.md#return-to-a-familiar-subtraction)
shows why that larger total alone does not prove starvation. Compare actual
deductions when explaining a run. We need both the model's rules and its outcomes.

## Reconcile counts before judging the model

Recall the last session's bounded history: a whole-run death count may outlive
the individual event details. To reconcile population size, use counters whose
coverage begins at run initialization. Counting the visible rows in a truncated
event list would undercount deaths and create an apparent simulation bug.
Adding the total number of evictions cannot repair that count: the missing
entries may describe meals or other outcomes. One retained death plus three
evicted events leaves the death total unknown unless a separate counter covers
those outcomes. Use the prepared whole-run counter and the independently
observed living population for this account.

For this experiment there are no births, additions or manual removals. Living
hares should therefore equal seeded hares minus recorded deaths. Keep births
as an explicit zero in the example so initialization cannot quietly become a
birth event. When reproduction arrives, this same account will gain a real
nonzero term rather than a new meaning for an old field.

The following samples are **illustrative inputs**, not observations from a
running Moss scenario. Their purpose is to teach the account without inventing
a claim about when the future ecosystem will collapse. The proposed browser
run supplies the actual timing later.

## Add one check that catches an impossible account

Implement the prepared `population_account_agrees` helper or its equivalent
literal assertions, then run the live account check supplied with it. Six
seeded, no births, three deaths and three living agrees. Six seeded, one death
and three living does not. More deaths than the available population must also
fail, even if clamping would make the arithmetic resemble an empty world.
If a live sample disagrees, first inspect when living entities were counted
and whether the death counter covers the whole run.

The **optional worked reference** below contains the complete check and test.
The agent supplies the live sample type, so this example needs no new statistics
framework. [Checking your work](README.md#checking-your-work) distinguishes the
prepared live checkpoint from these illustrative inputs.

<!-- runnable: session-15 -->
```rust
struct PopulationSample {
    seeded: u32,
    births: u32,
    deaths: u32,
    living: u32,
}

fn population_account_agrees(sample: &PopulationSample) -> bool {
    sample
        .seeded
        .checked_add(sample.births)
        .and_then(|total| total.checked_sub(sample.deaths))
        == Some(sample.living)
}

#[test]
fn session_15_population_account_rejects_missing_or_impossible_deaths() {
    for (deaths, living) in [(0, 6), (1, 5), (3, 3), (6, 0)] {
        let sample = PopulationSample {
            seeded: 6,
            births: 0,
            deaths,
            living,
        };
        assert!(population_account_agrees(&sample));
    }
    let missing_death = PopulationSample {
        seeded: 6,
        births: 0,
        deaths: 1,
        living: 3,
    };
    assert!(!population_account_agrees(&missing_death));
    let impossible = PopulationSample {
        seeded: 6,
        births: 0,
        deaths: 7,
        living: 0,
    };
    assert!(!population_account_agrees(&impossible));

    let overflowing = PopulationSample {
        seeded: u32::MAX,
        births: 1,
        deaths: 0,
        living: 0,
    };
    assert!(!population_account_agrees(&overflowing));
}
```

`checked_sub` returns `None` when deaths exceed the available population. That
absence carries an invalid-account result rather than clamping it into a
plausible zero. Compare this with reserve maintenance, where clamping at zero
is the intended model. The same arithmetic technique should not be used for
two quantities merely because both happen to be unsigned integers.

`checked_add` also returns an `Option`. The `and_then` closure receives the sum
only when that sum exists; otherwise it preserves `None` without subtracting.
Earlier, `map` transformed a present candidate into an ID. Here the next operation
can itself fail: `checked_sub` already returns an `Option`. `and_then` keeps that
result directly, so `Some(6)` followed by subtracting seven becomes `None`.
Using `map` would add a layer and produce `Some(None)` instead.
Comparing the result with `Some(sample.living)` accepts only a valid calculation
that matches the observation. This extends the earlier use of `Option` for no
food: here absence means the account could not be computed within its bounds.
The last case checks that an impossible counter does not wrap into a plausible
empty population. Its nonzero birth count tests the helper's arithmetic only;
this session's actual scenario still has no reproduction.

A reconciled equation does not prove that a sample was collected correctly.
The installed check must count living hares directly from the ECS world,
independently of the death counter. Deriving both from `seeded - deaths` would
make them agree even if an entity survived an alleged removal. Preserve the
assertion when investigating a mismatch; missing history does not mean zero
events occurred.

## Read one run, then choose what its evidence means

Before your session, the agent prepares one paused scarcity scenario and finds
a short fixed tick window containing a meal or death. The handoff names that
window and records its starting population, individual maintenance costs,
production limit and light phase. These values let you check that the prediction
about demand actually applies to the loaded scenario.

Run that window and compare the final live count with seeded creatures minus
whole-run deaths. Follow one actual meal or death record back to its creature;
keep its time coverage beside the population sample's coverage. If counts
reconcile but many hares die, the implementation may be correct and the scenario
harsh. Whether the resulting competition is interesting is a separate question
from either correctness or sustainable balance. The useful result today is a
small observation you can explain, not a chart whose shape happens to look good.

One explained run is a useful stopping point. If you want to investigate the
model further, make one optional controlled repeat: keep the authored animals,
positions, starting stores, light cycle and observation window identical, but
increase the accessible patch's daylight production rate from 1 to 2. Compare
actual recorded growth, meals and living counts. The request doubled; capacity,
access and appetite still limit what reaches an animal. No particular survival
time or balanced population is promised. This changes a scenario input while
preserving the accounting invariant: living still equals seeded minus deaths,
even if the observed number of deaths changes.

Ask: **“Review my population account and this scarcity observation. Help separate
a broken rule from a harsh setting and an uninteresting result.”** The next
consolidation returns to identical conditions to check repeatability; the
optional comparison above deliberately changes one condition to investigate
its effect. Both depend on knowing which claim the comparison is making.
