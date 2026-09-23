# 13. Decide when an empty reserve becomes terminal

[Path home](README.md) · [Previous: light controls growth](12-light-controls-growth.md) · [Next: removal and memory](14-removal-and-memory.md)

**Future policy session · 20–30 minutes.** This page recommends a rule for paired
review; it does not record an accepted or installed starvation policy. Animals
currently remain alive at zero. If you accept the proposal, your edit is the
small `is_starved_after_meals` predicate, planned in
`crates/moss-sim/src/lessons.rs`. The agent prepares its test and keeps death
resolution unscheduled until the next checkpoint.

## Return to a familiar subtraction

Your maintenance rule reduces energy and stops at zero. That arithmetic says
how much reserve remains; it never answered whether the creature still exists.
The distinction was useful while learning movement and eating because an empty
reserve did not silently introduce a new lifecycle. Now finite food and darkness
make the unanswered question visible: should Fern survive a tick that ends
with an empty reserve?

The recommendation is **resolve starvation at the end of the tick, after any
allowed meal, when reserve is still zero**. Maintenance still comes first, so
it drains up to the available reserve. A creature already able to eat may recover
before the terminal check. This creates a last-chance meal without providing free
movement or allowing a dead creature to act afterward.

This is a game-model decision, not a fact about real animals. An earlier check
could instead make maintenance reaching zero immediately terminal. That choice
would remove the last-chance meal and produce different populations. We should
choose deliberately, record it, and keep a literal test rather than let the
placement of a system accidentally decide for us.

There is a second consequence to accept explicitly: clamping forgives any part
of upkeep the reserve cannot cover. With reserve 1 and a requested cost of 2,
maintenance deducts only 1. Eating one unit afterward restores reserve 1, so the
proposed predicate permits survival. Repeat that on a patch growing one unit
each tick and the animal survives while food arrives—even though the configured
cost says 2. In this first model, that number is a bounded drain, not a fully
enforced nutritional requirement.

Keep this simple no-debt proposal visible during the policy review. If upkeep
must be paid in full, revise the composed rule before activating starvation;
`reserve == 0` after a meal cannot enforce that requirement by itself. We do not
need an energy-debt component just to expose the choice. One worked trace can
tell us whether the intended model and the proposed rule agree.

## Follow one tick all the way through

Put Fern on Meadow with reserve 1, a maintenance cost of 1, and 4 edible
biomass. Use executing tick 120: night has begun, so no new grass grows.
Maintenance leaves zero. Fern cannot afford a charged move, but no move is
needed because she is already in contact. Under the proposed policy, the meal
is still permitted. It brings reserve to 4, and the terminal predicate is false.

Now reset that small setup with no stored biomass, still at night. Maintenance
leaves zero and there is nothing to eat, so the predicate reports starvation.
The night condition matters. At dawn, session 12's growth rule replenishes an
empty patch before choice and eating. Fern can consume that new unit and finish
with reserve 1. An empty patch at the start of a tick does not prove that the
whole tick will provide no food.

| Start: Fern has 1 reserve | End after growth, maintenance and meal |
| --- | --- |
| Night, Meadow has 4 | Reserve 4; survives on stored food. |
| Night, Meadow has 0 | Reserve 0; starvation is proposed. |
| Dawn, Meadow has 0 | Reserve 1; survives on this tick's growth. |

All three outcomes use the same final predicate. Earlier rules produce its
input, so understanding `reserve == 0` requires understanding *when* it runs.
In the future installed fixture Fern is seeking; an empty patch must become eligible
after dawn growth, just as you tested in session 10.

Rest is separate. We may later track fatigue and let rest reduce it, but rest
does not manufacture food energy. Keeping those meanings distinct prevents an
empty creature from surviving indefinitely through an unrelated recovery stat.
Similarly, this recommendation does not make energy into health or define
injuries, resurrection, predation or corpse biomass.

## Encode the proposed boundary

Review the proposed timing before changing live behavior. If it matches your
intent, implement `is_starved_after_meals` and use the prepared live predicate
test. Reserve 0 is terminal; reserve 1 is not. Keep removal for the next
checkpoint. The agent also prepares the three composed cases above: stored
night food and new dawn growth rescue Fern, while an empty night does not.
If stored food fails to rescue her, check whether the predicate ran before
the meal. If only the dawn case fails, inspect whether choice saw the patch
before growth or rejected a zero-energy eater.

The **optional worked reference** below shows the policy and all three cases
in ordinary Rust, then checks the underpaid-upkeep consequence across one full
daylight interval. Its local tick helper demonstrates ordering rather than
running Moss's schedule; see [checking your work](README.md#checking-your-work).

<!-- runnable: session-13 -->
```rust
use moss_sim::{Energy, FoodPatch};

fn is_starved_after_meals(energy: &Energy) -> bool {
    energy.reserve == 0
}

fn contact_tick(
    energy: &mut Energy,
    food: &mut FoodPatch,
    executing_tick: u64,
    maintenance_units_per_tick: u32,
) -> bool {
    // Reuse the preceding lessons' order: light and growth precede the meal.
    let daylight = executing_tick % 240 < 120;
    let requested_growth = if daylight { 1_u32 } else { 0 };
    assert!(food.biomass <= 100);
    let actual_growth = requested_growth.min(100 - food.biomass);
    food.biomass = food
        .biomass
        .checked_add(actual_growth)
        .expect("biomass overflow");

    energy.reserve = energy.reserve.saturating_sub(maintenance_units_per_tick);
    let eaten = 4_u32
        .min(food.biomass)
        .min(energy.capacity.saturating_sub(energy.reserve));
    food.biomass -= eaten;
    energy.reserve = energy.reserve.checked_add(eaten).expect("energy overflow");
    is_starved_after_meals(energy)
}

#[test]
fn session_13_stored_food_and_dawn_growth_can_prevent_starvation() {
    let mut fern = Energy {
        reserve: 1,
        capacity: 100,
    };
    let mut meadow = FoodPatch {
        name: "Meadow",
        biomass: 4,
    };
    assert!(!contact_tick(&mut fern, &mut meadow, 120, 1));
    assert_eq!((fern.reserve, meadow.biomass), (4, 0));

    fern.reserve = 1;
    assert!(contact_tick(&mut fern, &mut meadow, 120, 1));
    assert_eq!((fern.reserve, meadow.biomass), (0, 0));

    fern.reserve = 1;
    assert!(!contact_tick(&mut fern, &mut meadow, 240, 1));
    assert_eq!((fern.reserve, meadow.biomass), (1, 0));
    assert!(!is_starved_after_meals(&fern));

    // Start a separate trace: a requested drain of 2, with only 1 available.
    fern.reserve = 1;
    meadow.biomass = 0;
    for executing_tick in 240..360 {
        assert!(!contact_tick(&mut fern, &mut meadow, executing_tick, 2));
        assert_eq!((fern.reserve, meadow.biomass), (1, 0));
    }
    assert!(contact_tick(&mut fern, &mut meadow, 360, 2));
    assert_eq!((fern.reserve, meadow.biomass), (0, 0));
}
```

The local composition assumes contact instead of running choice or movement.
The installed cases must establish those permissions before a meal can rescue
Fern; the terminal predicate does not grant an otherwise forbidden action.

## Stop at the decision, not an accidental death system

The predicate returns a `bool`; it neither despawns an entity nor records an
event. That small boundary lets you understand the policy before confronting
structural ECS mutation. There is no need to make the choice permanent today:
the test describes the selected behavior and makes a later revision explicit.

Ask: **“Review the proposed after-meal starvation policy and my predicate.
Record my acceptance before preparing the removal checkpoint.”** If you prefer
earlier terminal resolution or a full-payment requirement, revise this example
and its expectations together.
Next we connect an accepted terminal outcome to removal and one retained record,
so Fern cannot act again yet its disappearance remains explainable.
