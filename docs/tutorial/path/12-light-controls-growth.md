# 12. Let daylight feed the plants

[Path home](README.md) · [Previous: a day from ticks](11-a-day-from-ticks.md) · [Next: deciding starvation](13-deciding-starvation.md)

**Future session · 20–30 minutes.** Your edit is a `requested_growth` helper,
planned in `crates/moss-sim/src/environment.rs`: choose the production request
from effective light. The agent prepares its signature, a boundary regression
in `crates/moss-sim/tests/environment.rs`, and the wiring that supplies the
executing tick's light. It exposes effective light and actual growth in
inspection. These are future APIs, not fields already present in the browser
snapshot; the biological conditional remains your edit.

## Reuse the limit you already trust

Last session gave each executed tick a day or night classification. Earlier,
`grow_biomass` accepted a requested amount and returned what fit in Meadow.
The connection needs only one new decision: request 1 biomass unit during day
and 0 during night. The plant's capacity rule remains the same on both sides
of dusk.

This separation is useful design practice. The environment answers “how much
production is requested here?” and the patch answers “how much can I retain?”
Neither needs to know how the browser paints the sky. Later clouds can alter
the environmental input without requiring a different eating function or a
different capacity check. We are giving a concrete future change somewhere to
fit, without building a general weather system in advance.

Darkness also does not undo existing grass. Meadow can hold biomass throughout
the night, and Fern may eat that stock. Sunlight is the source of new plant
production; it is not a condition attached to every bite. Keeping stock and
production separate will matter when a population survives a long night using
what grew earlier.

## Examine the boundary instead of a long animation

Set Meadow to 98 biomass. At executing tick 119, daylight produces 1 and leaves
99. Tick 120 is night and leaves 99. Tick 239 is still night. Tick 240 begins
day again, produces 1, and reaches capacity 100. These four values isolate the
boundary more clearly than watching a minute of animation and guessing when
growth stopped.

The normal schedule still grows plants before creatures act. Thus biomass
produced at dawn can be eaten during that same tick. The reference below
demonstrates the production boundary and one subsequent transfer. It leaves
maintenance and movement out deliberately; the installed regression prepared
for your edit must exercise the actual environment and growth systems too.

## Protect the causal connection

Implement `requested_growth`, then run its prepared live boundary regression.
Daylight requests the configured rate; night requests 0. The installed check
also starts the animal at reserve 20 with 4 stored biomass at night and expects
a meal to leave reserve 24 and biomass 0. Zero maintenance and same-cell contact
isolate light from costs. If night erases existing food, inspect whether the rule
assigned the request to biomass instead of adding the bounded actual growth.

The **optional worked reference** below gives the conditional and expected
arithmetic with local helpers. It is complete but does not exercise the live
schedule; [checking your work](README.md#checking-your-work) explains which
checkpoint to use for your edit.

<!-- runnable: session-12 -->
```rust
use moss_sim::{Energy, FoodPatch};

fn requested_growth(daylight: bool, full_light_rate: u32) -> u32 {
    if daylight { full_light_rate } else { 0 }
}

fn produce_for_tick(patch: &mut FoodPatch, executing_tick: u64) -> u32 {
    let daylight = executing_tick % 240 < 120;
    let requested = requested_growth(daylight, 1);
    assert!(patch.biomass <= 100);
    let actual = requested.min(100 - patch.biomass);
    patch.biomass = patch.biomass.checked_add(actual).expect("biomass overflow");
    actual
}

fn eat_stored_food(energy: &mut Energy, patch: &mut FoodPatch) -> u32 {
    let eaten = 4_u32
        .min(patch.biomass)
        .min(energy.capacity.saturating_sub(energy.reserve));
    patch.biomass -= eaten;
    energy.reserve = energy.reserve.checked_add(eaten).expect("energy overflow");
    eaten
}

#[test]
fn session_12_light_changes_production_without_erasing_food() {
    assert_eq!(requested_growth(true, 3), 3);
    assert_eq!(requested_growth(false, 3), 0);
    assert_eq!(requested_growth(true, 0), 0);

    let mut meadow = FoodPatch {
        name: "Meadow",
        biomass: 98,
    };
    assert_eq!(produce_for_tick(&mut meadow, 119), 1);
    assert_eq!(meadow.biomass, 99);
    assert_eq!(produce_for_tick(&mut meadow, 120), 0);
    assert_eq!(produce_for_tick(&mut meadow, 239), 0);
    assert_eq!(meadow.biomass, 99);
    assert_eq!(produce_for_tick(&mut meadow, 240), 1);
    assert_eq!(meadow.biomass, 100);

    let mut fern = Energy {
        reserve: 20,
        capacity: 100,
    };
    let eaten = eat_stored_food(&mut fern, &mut meadow);
    assert_eq!((eaten, fern.reserve, meadow.biomass), (4, 24, 96));

    assert_eq!(produce_for_tick(&mut meadow, 241), 1);
    assert_eq!(meadow.biomass, 97);

    let mut night_food = FoodPatch {
        name: "night stock",
        biomass: 4,
    };
    fern.reserve = 20;
    assert_eq!(produce_for_tick(&mut night_food, 120), 0);
    assert_eq!(eat_stored_food(&mut fern, &mut night_food), 4);
    assert_eq!((fern.reserve, night_food.biomass), (24, 0));
}
```

The night calls preserve the existing 99 units, while the separate night meal
uses stored food. These cases protect different connections: light controls
production, and biomass controls what can be eaten. If playback speed changes
the outcome for the same executed ticks, inspect for frame time entering the
simulation path.

<a id="finish-with-an-inspectable-dawn"></a>

## Follow the same patch into night

For browser review, the agent prepares a seeking hare already on an empty
patch, at completed tick 118. Start its reserve at 20, maintenance at 1,
daylight growth at 1 and bite limit at 4. Same-cell contact means no travel;
this observation has no death rule. These are **expected results**, not a run
already observed:

![At tick 119, one unit grows and is eaten, leaving an empty patch and reserve 20. At night tick 120, neither flow occurs and maintenance leaves reserve 19.](visuals/stock-and-flow.svg)

Watch two consecutive ticks. At 119, growth adds 1 and the meal transfers 1;
maintenance spends 1, so reserve remains 20 and biomass ends at 0. At 120,
this empty patch has neither new growth nor stored food to supply a meal;
reserve becomes 19 and biomass remains 0.
The same empty picture can conceal different activity. Read actual growth and
meals beside the final stores to see the connection. Dawn permits new growth
again; whether biomass accumulates depends on capacity and consumption.

The fixture avoids clicking through a whole day to reach this transition.
A tint helps locate dusk; simulation values explain it. Pause should freeze
both the phase and these exchanges, including while the tab is hidden.

Ask: **“Review my light-to-growth helper and compare the prepared browser
transition with the native regression.”** Stop once that single connection is clear.
Weather remains a later experiment: a deterministic cloudy interval could reduce
light, while fractional production would need retained remainders to avoid
rounding every small amount to zero. Next we address an immediate consequence
of finite supply: what should happen when Fern finishes a tick without energy?
