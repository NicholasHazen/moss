# 05. Read the population without losing the individuals

[Path home](README.md) · Previous: [when to seek](04-when-to-seek.md) · Next: [owned costs](06-owned-costs.md)

**Future checkpoint · 20–30 minutes after autonomous foraging is reviewed.**
The agent prepares the authored population scenario, deterministic reset, browser
selector and species summary plumbing. Your edit is one small summary function
at a prepared landmark in a simulation-owned observation module. Its exact live
path is recorded before activation; no summary API exists in the current code.

Watching Fern tells us whether one animal can feed. Watching six hares lets us
ask whether the available food offers similar opportunities to different
individuals. The new scene will contain six hares, two foxes and four grass
patches, with authored positions and IDs. These counts create an experiment;
they do not promise a sustainable ecosystem.

## Remember what a group means

Fern is a nickname, Hare is a species, and Grazer is a food-web role. A species
summary groups by `Species`, regardless of nickname. ECS archetype means a set
of component types and does not tell us what species a creature belongs to.
For example, adding `FoodTarget` can change an entity's archetype without turning
a hare into another kind of animal.

The agent keeps the original three-entity diagnostic scene available. Selecting
the population scene starts a fresh run; Reset restores its authored IDs,
positions and starting values. We do not need births or random placement to
obtain several animals. Keeping placement repeatable gives us a firm baseline
for the individual-cost experiment in the next few sessions.

At reset every animal has reserve 60 and capacity 100. The hare summary should
therefore say count 6, minimum 60, maximum 60 and mean 60. Four patches at biomass
80 contribute total biomass 320. Patch count describes local stands of plants,
not the number of individual grass blades. Those are different units of observation.

## Summarize values that already exist

Suppose the mean reserve is 50. That could describe two comfortable hares at
`[50, 50]`, or one empty and one full at `[0, 100]`. The same average tells two
different stories. Keeping minimum and maximum beside it lets us see that
difference without pretending a summary replaces the individuals.

The adapter supplies the reserves for one species. Your function computes their
summary without modifying those reserves. This is a read-only projection of the
authoritative world, much like the renderer derives a transform from `Position`.
A summary never becomes a second place that owns animal energy.

Your one edit is the prepared `summarize_energy` body. On `[20, 60]`, start with
the first value: total, minimum and maximum are all 20. Visiting 60 changes the
total to 80 and maximum to 60, while minimum stays 20. Two members give a mean
of `80 / 2 = 40`. An empty slice needs a separate answer because it has no first
member and no meaningful mean.

## Check the view against its members

Run the live summary test prepared alongside `summarize_energy`; the agent records
its file and exact command before the session. The unfinished body should fail,
then your edit should pass the mixed-values and empty-input cases. If `[20, 60]`
reports 20 or 60 as its mean, inspect the accumulator and divisor rather than
changing the fixture.

## A complete summary example

The **optional worked answer** below defines the return type and test locally.
The agent prepares their live equivalents; you supply only the summary calculation.
Notice that this function needs a borrowed slice of numbers, not mutable access
to an ECS world.

<!-- runnable: session-05 -->
```rust
#[derive(Debug, PartialEq)]
struct EnergySummary {
    count: usize,
    minimum: u32,
    maximum: u32,
    mean: f64,
}

fn summarize_energy(reserves: &[u32]) -> Option<EnergySummary> {
    let (&first, rest) = reserves.split_first()?;
    let mut total = u64::from(first);
    let mut minimum = first;
    let mut maximum = first;
    for &reserve in rest {
        total = total
            .checked_add(u64::from(reserve))
            .expect("population energy total overflow");
        minimum = minimum.min(reserve);
        maximum = maximum.max(reserve);
    }
    Some(EnergySummary {
        count: reserves.len(),
        minimum,
        maximum,
        mean: total as f64 / reserves.len() as f64,
    })
}

#[test]
fn session_05_a_mean_has_members_and_limits() {
    assert_eq!(
        summarize_energy(&[20, 60]),
        Some(EnergySummary {
            count: 2,
            minimum: 20,
            maximum: 60,
            mean: 40.0,
        })
    );
    assert_eq!(
        summarize_energy(&[60; 6]),
        Some(EnergySummary {
            count: 6,
            minimum: 60,
            maximum: 60,
            mean: 60.0,
        })
    );
    assert_eq!(summarize_energy(&[]), None);
    assert_eq!(summarize_energy(&[20, 21]).unwrap().mean, 20.5);
    let unequal = summarize_energy(&[0, 100]).unwrap();
    let equal = summarize_energy(&[50, 50]).unwrap();
    assert_eq!(unequal.mean, equal.mean);
    assert_eq!((unequal.minimum, unequal.maximum), (0, 100));
    assert_eq!((equal.minimum, equal.maximum), (50, 50));
}
```

For `[20, 60]`, `split_first` returns borrowed pieces: a reference to `20` and a
slice containing `60`. In `let (&first, rest)`, the `&first` pattern copies that
first `u32` into a local number; `rest` remains a borrowed slice. Neither operation
removes an element from the caller's data. The `?` unwraps the pair when present,
or returns `None` from this summary immediately when the input is empty.

This is a compact use of the absence we encountered when no food qualified. Here
an empty group has no mean, minimum or maximum; displaying zero would invent a
measurement and make “no hares” resemble “hares with no energy.”

The first member supplies valid initial minimum and maximum values. We accumulate
in `u64` to provide more room than an individual `u32` reserve, still checking
addition. Division uses floating point so a mean such as 20.5 is not truncated.
These small diagnostic populations fit comfortably; this display calculation is
not a promise of exact floating-point summaries at arbitrary scale.

**Optional:** run the complete answer; expect one passing reference test.

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 05
```

[What this checks](README.md#checking-your-work) · [Recorded evidence](verification.md)

After integration, select the population scene and inspect two hares before and
after several Steps. Travel and eating may produce different reserves even when
maintenance rates match. Those two inspections give the summary a concrete
meaning; they cannot verify a six-hare mean. The agent also gathers all six hare
reserves, checks their count and sum, and compares the derived minimum, maximum
and mean with the display. Foxes and grass patches are checked separately so a
correct formula cannot hide incorrect species filtering. Do not expect every
food-containing scene to follow a maintenance-only prediction. Reset should
restore the literal starting counts and values above.

Send: **“Session 05's summary is green. Review the empty-group case and compare
the browser summary with individuals; keep the diagnostic scenario available.”**
The summary gives unequal outcomes somewhere visible to appear. Next we will
make one controlled difference belong to an individual rather than to its species.
