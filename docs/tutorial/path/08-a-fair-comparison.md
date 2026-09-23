# 08. Change one cost and explain the difference

[Path home](README.md) · Previous: [querying costs](07-querying-costs.md) · Next: [growing grass](09-growing-grass.md)

**Future checkpoint · 20–30 minutes after individual costs work.** The agent
prepares a comparison scenario with two hares at the same cell, reserve 60,
the same distant food opportunity and `Seeking` activity. One has maintenance 1,
the other 2. Your edit adds one three-tick comparison assertion to the prepared
scenario test. Travel remains the shared 2 units per cell; no new attribute API
is needed for this experiment.

The no-food test established that one hare pays more upkeep. It did not show
how that difference combines with the behavior we built earlier. This time both
animals walk toward the same patch. If they take equal steps and do not yet
reach food, the extra energy loss should still have one identifiable cause.

## Keep the other causes equal

Recall why the previous test removed food: we were separating maintenance from
travel and eating. We can now allow travel while controlling its contribution.
Both hares begin at `(10, 10)` and seek Meadow at `(16, 13)`. After three steps,
both should be at `(13, 10)`, still six cells short of the patch. Sharing a cell
is allowed by the current model; sprite overlap is not a collision rule.

The starting `Seeking` state matters. At reserve 60, an idle hare would stay
idle under our hysteresis rule, while a seeking hare continues. A comparison
that forgets this state could accidentally compare walking with standing still
and blame the difference on upkeep. Session 04's remembered activity is now an
experimental input, not merely an inspector label.

Each hare travels three cells at 2 units per cell, so each pays 6 for travel.
Fern pays another 3 in maintenance and ends at **51**. The other hare pays 6
in maintenance and ends at **48**. Their reserve gap is 3 because the upkeep
difference was 1 on each of three executed ticks. Neither eats yet, and Meadow
should still have 80 biomass.

| Completed ticks | Both positions; Fern / other hare reserves |
| --- | --- |
| 0 | `(10, 10)`; `60 / 60` |
| 1 | `(11, 10)`; `57 / 56` |
| 2 | `(12, 10)`; `54 / 52` |
| 3 | `(13, 10)`; `51 / 48` |

Each row subtracts 3 from Fern and 4 from the other hare: upkeep plus the same
2-unit step. This is the expected trace for the prepared experiment. It tells us
what to look for before we open a test or the browser.

## Write the accounting before interpreting the result

Your one live edit extends the prepared scenario test after its third tick.
Using its existing stable-ID lookups, assert both positions are `(13, 10)`, Fern's
reserve is 51, the other hare's reserve is 48, and Meadow's biomass is still 80.
The agent supplies the scenario and lookup code. Those observations matter as
a group: with the same start and at most one cardinal step on each of these
three ticks, reaching `(13, 10)` establishes three cells of travel for each hare.
Unchanged biomass rules out a meal, and the reserves reveal the upkeep difference.
Equal endpoints alone would not account for a journey containing a detour.

## Make one comparison, then stop

Complete those assertions in the prepared live test and run its focused command,
recorded by the agent before the session. This test advances the installed
simulation three times; the independent accounting reference does not run it.
If it reports 57/54, inspect whether the starting activity was `Idle`. Those are
the maintenance-only results, so changing the rates would hide the wrong cause.
If biomass fell, the supposed pre-arrival setup was not the one tested.

## Follow the accounting in Rust

The table above is enough to derive those assertions. The **optional worked
reference** below accounts for an already-supplied movement trace; it is available
if you want to follow the arithmetic in Rust. `Account` is local example data,
not a new component or a helper you need to install. The live simulation must
produce the journey through its existing movement rule.

<!-- runnable: session-08 -->
```rust
#[derive(Debug, PartialEq, Eq)]
struct Account {
    reserve: u32,
    maintenance_spent: u32,
    travel_spent: u32,
}

fn account_trace(maintenance_units_per_tick: u32, accepted_cells: &[u32]) -> Account {
    let mut result = Account {
        reserve: 60,
        maintenance_spent: 0,
        travel_spent: 0,
    };
    for &distance in accepted_cells {
        let actual_maintenance = result.reserve.min(maintenance_units_per_tick);
        result.reserve -= actual_maintenance;
        result.maintenance_spent = result
            .maintenance_spent
            .checked_add(actual_maintenance)
            .expect("maintenance total overflow");
        let travel = distance.checked_mul(2).expect("travel cost overflow");
        assert!(
            travel <= result.reserve,
            "trace contains an unaffordable step"
        );
        result.reserve -= travel;
        result.travel_spent = result
            .travel_spent
            .checked_add(travel)
            .expect("travel total overflow");
    }
    result
}

#[test]
fn session_08_equal_journeys_reveal_the_upkeep_difference() {
    let fern = account_trace(1, &[1, 1, 1]);
    let other = account_trace(2, &[1, 1, 1]);
    assert_eq!(
        fern,
        Account {
            reserve: 51,
            maintenance_spent: 3,
            travel_spent: 6
        }
    );
    assert_eq!(
        other,
        Account {
            reserve: 48,
            maintenance_spent: 6,
            travel_spent: 6
        }
    );
    assert_eq!(fern.reserve - other.reserve, 3);
    assert_eq!(
        fern.reserve + fern.maintenance_spent + fern.travel_spent,
        60
    );
    assert_eq!(
        other.reserve + other.maintenance_spent + other.travel_spent,
        60
    );
    assert_eq!(account_trace(1, &[0, 0, 0]).reserve, 57);
}
```

The trace records actual distance, so a zero entry still pays maintenance while
paying no travel. The explicit affordability assertion prevents the reference
from accepting an impossible movement history. This is an accounting check on
supplied observations, not permission to move on credit.

An energy total alone could hide a different combination of movement and food.
The live test checks the causes we intended to hold constant as well as the
final reserves. That is why its position and biomass assertions are part of the
experiment, even though the parameter we changed was maintenance.

**Optional:** run the complete answer; expect one passing reference test.

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 08
```

[What this checks](README.md#checking-your-work) · [Recorded evidence](verification.md)

After integration, Reset the comparison scene, Step three times and inspect
each animal. There is no need to let the run continue until food competition
introduces another cause. A narrow, explainable result is the point of this session.
It establishes that individual settings matter without claiming which animal
will thrive across every habitat or how fast the storage layout performs.

Send: **“Session 08's comparison is green. Review the controlled inputs and
verify 51/48 with equal travel before preparing plant renewal.”**
We now have individual differences worth observing. The next limit is visible
in Meadow: every meal removes food, so the world still spends down a finite stock.
We will give the patch a bounded way to replace that stock before adding sunlight.
