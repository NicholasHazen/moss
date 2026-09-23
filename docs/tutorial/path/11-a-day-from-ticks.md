# 11. Make a day belong to the simulation

[Path home](README.md) · [Previous: growth meets meals](10-growth-meets-meals.md) · [Next: light controls growth](12-light-controls-growth.md)

**Future session · 20–30 minutes.** Your edit is a pure `is_daylight` function,
planned in `crates/moss-sim/src/environment.rs`. The agent creates that module
when this session becomes active, prepares its test, and handles the future
environment resource, re-export and inspector wiring. Today there is no live
day/night cycle. This lesson adds time classification before changing growth.

## Remember which clock changes the world

You have already made growth and meals happen in a particular tick order.
The next question is whether every tick receives the same light. Moss advances
because the browser requests complete simulation ticks. A rendered frame can
request no tick, one tick, or several; it is not itself a biological time unit.
That distinction now gives us a useful promise: pausing Moss also pauses its day.

Imagine leaving the tab during the night and returning later. If daylight came
from the computer's clock, the environment could advance while Fern stood still.
We would then have to invent rules for missed plant growth and energy costs.
Our current model avoids that problem by deriving phase only from executed
simulation ticks. Hidden-tab suspension does not create catch-up biology.

Use a 240-tick day for this demonstration. The first 120 phase values are day;
the next 120 are night. At the normal four ticks per second this is a convenient
one-minute viewing cycle. It is a playback choice, not an assertion about hare
metabolism or real sunlight. You can change the model's duration later without
making the rule depend on wall time.

## Wrap a growing counter into a small phase

The remainder operator `%` gives a repeating phase: `tick % 240`. Ticks 0 and
240 both have phase 0, while tick 239 has phase 239. The daylight test then asks
whether that phase is below 120. Thinking in two steps separates the repeating
clock from the rule that interprets its phase.

There is one boundary convention to remember. If the world has completed 119
ticks, the next executing tick is 120. We compute light for 120 before its
growth step, so that tick is already night. After committing it, the inspector
shows completed tick 120 and night. Tick 0 describes the initialized environment;
it does not grant an extra growth update before the first requested Step.

That convention makes the initial daylight interval slightly shorter: Steps
1–119 receive light, then 120–239 are night. The next daylight interval,
240–359, contains all 120 updates. There is no missing growth at initialization;
tick 0 describes state rather than an executed update. Keep this distinction
when predicting production from a run that starts at reset.

## Write the smallest clock rule

Implement only `is_daylight`, then use the live test command the agent
prepares with it. Expect ticks 119 and 240 to be daylight, and 120 and 239 to
be night. If 120 still reports day, inspect whether your comparison uses `<=`
instead of `<`. If the helper passes but the inspector changes a tick late,
check whether the environment received the completed tick or the executing one.

The **optional worked reference** below uses a fixed nonzero period so you can
focus on remainder and boundary conditions. A configurable day length would
need validation; accepting zero and hoping division works is not a later
feature. The reference is complete and independently testable, as described in
[checking your work](README.md#checking-your-work).

<!-- runnable: session-11 -->
```rust
const TICKS_PER_DAY: u64 = 240;

fn is_daylight(tick: u64) -> bool {
    tick % TICKS_PER_DAY < TICKS_PER_DAY / 2
}

#[test]
fn session_11_daylight_changes_at_exact_tick_boundaries() {
    assert!(is_daylight(0));
    assert!(is_daylight(119));
    assert!(!is_daylight(120));
    assert!(!is_daylight(239));
    assert!(is_daylight(240));
    assert!(is_daylight(359));
    assert!(!is_daylight(360));

    let completed_tick = 119_u64;
    let executing_tick = completed_tick.checked_add(1).expect("tick overflow");
    assert_eq!(executing_tick, 120);
    assert!(!is_daylight(executing_tick));
    assert!(is_daylight(completed_tick));

    assert_eq!((1..120).filter(|tick| is_daylight(*tick)).count(), 119);
    assert_eq!((240..360).filter(|tick| is_daylight(*tick)).count(), 120);
}
```

The function returns a value and changes nothing. In Rust terms it needs no
mutable borrow because there is no state to update. The environment system will
later call it with the executing tick and expose the result; the renderer may
read that result to tint the scene. A dark background must never become the
source of truth that decides whether grass grows.

## Leave the next connection visible

Once the helper is reviewed, the agent can wire and inspect the environment
without changing growth yet. A useful browser check pauses before dusk, waits
without advancing, then Steps across the boundary. Waiting should change
nothing; the Step should change the reported phase. That is a future browser
acceptance check, not evidence supplied by this function test.

Ask: **“Review my daylight boundaries and prepare the environment inspector.
Keep plant growth unchanged until the light-to-growth checkpoint.”** Stop when
the clock and displayed phase agree. Next we let the same phase select growth's
requested rate. You will reuse the capacity helper rather than write a second
plant rule for night.
