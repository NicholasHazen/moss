<p class="eyebrow">Chapter 2 · Bounded transfers and shared resources</p>

<a id="2-a-meal-has-two-sides"></a>

# A meal has two sides

At the end of the proposed movement-only journey, Fern stands on Meadow's cell
with 33 energy. Meadow still holds its original 80 biomass. Arrival has cost
Fern something, but it has given her nothing. Another Step would spend another
maintenance unit while she stands on an untouched meal.

We can give arrival a consequence without changing the journey. Some biomass
will leave Meadow and become energy in Fern. The interesting word is *some*:
Fern has a capacity, Meadow has a finite supply, and one eating action has a
bite limit. Those three limits must agree on one transfer. Later, another hare
will ask for a share of the same patch.

The first part of this chapter follows that transfer through two mutable
borrows. The second keeps the helper unchanged and gives competing eaters an
explicit order. Each part has its own stopping point. If the meal stretch after
movement already covers either result, keep the reviewed work and use the
examples to reconnect with it.

**Worked reference · future live behavior.** This chapter starts after the
movement journey has been reviewed. In this edition's live baseline, movement
is still the unfinished exercise and eating is not installed. Follow
the local repository's `NOW.md` for the current edit; the complete examples here run
in isolation without changing that world.

## On this page

- [Bound one meal](#how-much-of-the-meal-can-happen)
- [Resolve competing eaters](#another-hare-reaches-the-same-patch)
- [Separate the meal from the tick’s net change](#a-meal-is-not-the-ticks-net-gain)
- [Return the meal to the journey](#what-changes-when-the-meal-joins-the-journey)

## How much of the meal can happen?

Set the long journey aside for a moment. Give Fern reserve **98** and capacity
**100**, and give Meadow **5 biomass**. Our first meal model permits a bite of
at most **4 biomass units per eating action**, converting one biomass unit into
one energy unit.

Fern has room for only two. Awarding four would overfill her. Removing four
from Meadow but awarding only two would respect Fern's capacity while making
two biomass disappear. The patch and the animal need to describe the same
event, so first decide what can actually transfer.

| Limit on this meal | Amount allowed |
| --- | --- |
| Room in Fern: capacity 100 minus reserve 98 | 2 energy units |
| Food remaining in Meadow | 5 biomass units |
| Maximum bite | 4 biomass units |

Under the chosen 1:1 conversion, the smallest bound allows a transfer of **2**.
Fern ends at 100; Meadow keeps 3. The bite limit was a request ceiling, not an
instruction to subtract four regardless of the outcome.

The units still matter even though their numerical conversion is one. Biomass
describes available plant material; reserve describes stored animal energy.
This demonstration rule connects them. It is not a biological claim that one
unit of any food must always yield one energy unit. Keeping both names now
makes the current assumption visible without designing another food system.

Movement gave us a useful way to think: work out an allowed change before
committing it. A meal uses the same pattern. It differs in one deliberate
respect: movement rejected a step that Fern could not afford, while a meal
can take a smaller amount than the bite limit. Rust permits both policies;
the rule tells us which outcome to calculate.

## Two borrows reach two existing values

The helper needs writable access to Fern's `Energy` and Meadow's `FoodPatch`.
It does not need either whole entity, a browser, or a simulation tick. Those
existing types live in [energy.rs](../../../crates/moss-sim/src/energy.rs) and
[components.rs](../../../crates/moss-sim/src/components.rs).

The signature in the complete reference below contains `energy: &mut Energy`
and `patch: &mut FoodPatch`. Each `&mut` grants temporary exclusive access to
the caller's value. In the test, `&mut fern` and `&mut meadow` supply that access.
The helper can write their fields; it does not take ownership of Fern or Meadow
and return replacements.

![Fern's energy and Meadow's patch remain owned by the caller. During one call, mutable borrows let the helper transfer two units, leaving reserve 100 and biomass 3.](../../../docs/tutorial/path/visuals/meal-borrow.svg)

[Open the mutable borrowing diagram at full size](../../../docs/tutorial/path/visuals/meal-borrow.svg).

Follow the arrows back to the caller. After the call, the assertions inspect
the same `fern` and `meadow` values with their changed fields. The returned
number, 2, supplies additional information: it says how much was consumed.
It does not perform the mutations by itself. A function that returns 2 but
leaves both values untouched would tell a plausible story without making it
happen.

The helper has no position, target or ecological role argument. That absence
defines its boundary. It can calculate a valid transfer, but it cannot decide
whether a hare is beside food or a fox is permitted to graze. The later ECS
adapter must establish an eligible interaction before calling it. Keeping that
boundary small lets us inspect the arithmetic without constructing a whole
world first.

## The complete bounded transfer

This is the exact isolated reference from
[session 01](../../../docs/tutorial/path/01-bounded-meal.md), including its
test. Its local `eat_from_patch` is a worked answer, not a function already
installed in the live crate. The future live exercise prepares that helper in
`lessons.rs`; it does not ask you to paste this entire test module into that
file.

Read the first call with the 98/100, 5-biomass example beside it. Then follow
the second call without resetting anything: Fern is now full, so there is no
room for another transfer.

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 01
```

<!-- moss-example: session-01 -->
```rust
use moss_sim::{Energy, FoodPatch};

fn eat_from_patch(energy: &mut Energy, patch: &mut FoodPatch, bite_limit: u32) -> u32 {
    let room = energy.capacity.saturating_sub(energy.reserve);
    let consumed = room.min(patch.biomass).min(bite_limit);
    energy.reserve = energy
        .reserve
        .checked_add(consumed)
        .expect("meal energy overflow");
    patch.biomass -= consumed;
    consumed
}

#[test]
fn session_01_a_meal_changes_both_participants() {
    let mut fern = Energy {
        reserve: 98,
        capacity: 100,
    };
    let mut meadow = FoodPatch {
        name: "Meadow",
        biomass: 5,
    };

    assert_eq!(eat_from_patch(&mut fern, &mut meadow, 4), 2);
    assert_eq!(fern.reserve, 100);
    assert_eq!(meadow.biomass, 3);
    assert_eq!(eat_from_patch(&mut fern, &mut meadow, 4), 0);
    assert_eq!(meadow.biomass, 3);

    fern.reserve = 0;
    assert_eq!(eat_from_patch(&mut fern, &mut meadow, 0), 0);
    assert_eq!(fern.reserve, 0);
    assert_eq!(meadow.biomass, 3);

    assert_eq!(eat_from_patch(&mut fern, &mut meadow, 2), 2);
    assert_eq!(fern.reserve, 2);
    assert_eq!(meadow.biomass, 1);

    assert_eq!(eat_from_patch(&mut fern, &mut meadow, 4), 1);
    assert_eq!(fern.reserve, 3);
    assert_eq!(meadow.biomass, 0);
    assert_eq!(eat_from_patch(&mut fern, &mut meadow, 4), 0);
    assert_eq!(fern.reserve, 3);
    assert_eq!(meadow.biomass, 0);
}
```

`room.min(patch.biomass).min(bite_limit)` tightens one upper bound twice.
In the first case, room is 2, so neither five available biomass nor a bite
limit of four can enlarge the transfer. The resulting `consumed` is used for
both mutations. That shared amount is the small but consequential decision
in this function.

The room calculation uses `saturating_sub` because free capacity cannot be
negative. If a malformed animal already has reserve above capacity, it gets
zero room and no additional food; this helper does not repair its existing
reserve. The later subtraction is safe because `consumed` cannot exceed the
patch's current biomass. `checked_add(...).expect(...)` makes an unexpected
energy overflow fail explicitly instead of wrapping the value.

There is no special full-animal branch. When room is zero, `consumed` is zero;
adding and subtracting it change neither participant. The final `consumed`
expression returns that actual amount. A rejected or empty meal therefore has
an ordinary result: zero.

### Let each limit decide a case

The first nearly full animal does not, by itself, prove all three limits work.
A broken helper that considered only capacity would also transfer two from
the initial patch. The test changes its inputs so the other bounds have a turn.

Setting `fern.reserve = 0` is fixture preparation, not a new biological rule.
Meadow still has three biomass. With a bite limit of zero, nothing transfers.
With a bite limit of two, Fern receives two and Meadow keeps one. A following
request for four can take only that last one. At each call, look at the current
values rather than assuming the test starts over.

These cases distinguish three plausible mistakes: ignoring a configured zero,
hard-coding the usual bite of four, and awarding food that the patch no longer
contains. The assertions check the return value and both participants because
each can be wrong independently. If Fern reaches 100 but Meadow falls from five
to one, the likely mistake is subtracting the requested bite while awarding
the bounded transfer.

**A nearby case:** keep reserve 98, capacity 100 and bite limit 4, but give
Meadow only one biomass. What limits the meal now?

**Answer:** supply wins. The helper returns 1, Fern reaches 99, and Meadow
reaches 0. Fern's remaining room does not create a second unit of food. On
another call, the empty patch yields zero. This changed case also explains why
checking only the animal's capacity would be insufficient.

### A useful first stopping point

The reference command beside the [complete bounded transfer](#the-complete-bounded-transfer)
expects one named test to pass. The runner extracts the existing guide's exact
answer into a temporary workspace; it does not test an unfinished
live helper or install eating. The
[reference evidence](../../../docs/tutorial/path/verification.md) records its
executed checks separately from future integration.

When this checkpoint becomes the active task, the agent first prepares the
live helper, imports and test described in the
[eating companion](../../../docs/tutorial/05-eating.md#checkpoint-a--a-bounded-transfer).
Your edit is then only the helper body, and your checkpoint is the prepared
live test. Stop for review once the transfer is understood. Competition can
be a separate sitting; it will reuse this helper unchanged.

## Another hare reaches the same patch

One meal involves two values. Two meals raise a new question: which animal
sees Meadow first? With five biomass and two hungry hares each able to take
four, both cannot receive a full bite.

The proposed rule is simple: eligible eaters resolve in ascending `SimId`
order. Fern, ID 1, goes before the second hare, ID 7, even if the second hare
appears first in the input list. Stable IDs make that choice independent of
the incidental order in which data happens to arrive.

| Resolution turn | Result |
| --- | --- |
| ID 1 sees five biomass | It receives four; one remains. |
| ID 7 sees one biomass | It receives one; zero remains. |

This is repeatable allocation, not a claim of fairness. A lower-ID animal can
win repeatedly. That bias is visible enough to investigate later. For now,
an explicit rule makes the result explainable and testable; accidental query
order would conceal the same policy decision inside storage behavior.

Sorting is only half the solution. Each eater must also read the patch after
the preceding transfer. A request to eat does not reserve the biomass it saw
when the request was made.

### At the workbench: the last bite

Keep Meadow at five biomass and reverse the supplied input order. Resolve
the two eaters one at a time, watching the remainder passed from the first
turn to the second. Then try four biomass: one hare gets the whole supply.
Changing the starting biomass or input order begins a fresh example with both
reserves back at zero.
The five-biomass table above gives the same account without interaction.

<section class="lab" data-lab="meal" aria-label="Two eligible eaters sharing Meadow">
<div class="lab-heading"><span class="lab-kind">Independent teaching model</span><strong>One patch, two eligible eaters</strong></div>
<p>This JavaScript illustration does not execute Rust or the live simulation. Fern (ID 1) and the second hare (ID 7) are already eligible to eat. Each starts with reserve 0, capacity 100 energy units and a bite limit of 4 biomass. The conversion is 1 energy unit per biomass unit.</p>
<div class="lab-controls">
<label>Starting Meadow biomass <input data-input="biomass" type="number" min="0" max="12" step="1" value="5" disabled> biomass units</label>
</div>
<div class="lab-state" data-output="order"></div>
<p data-output="remaining">Meadow begins with 5 biomass units.</p>
<div class="lab-state" data-output="transfers"></div>
<div class="lab-actions"><button type="button" data-action="reverse" aria-pressed="false" disabled>Reverse input order</button><button type="button" data-action="resolve" disabled>Resolve ID 1</button><button type="button" data-action="reset" disabled>Reset example</button></div>
<p class="lab-explanation" data-output="explanation" role="status" aria-live="polite" aria-atomic="true">Enable JavaScript to resolve each turn interactively. The static table above shows ID 1 receiving 4, ID 7 receiving 1, and no biomass remaining. Reset restores that starting supply and input order 1 then 7.</p>
</section>

### Safe access can still tell the wrong story

Imagine copying Meadow's starting supply, five, into a local number. Use that
copy to calculate two awards of four. Then update each animal in turn and clamp
the patch's subtraction at zero. Every mutation could happen through a valid,
short mutable borrow, in a single thread. The animals would still gain eight
from a supply of five.

![Two transfers using the current patch award four and then one, consuming five total. Two awards calculated from the initial supply instead grant eight, even if the patch's final subtraction clamps at zero.](../../../docs/tutorial/path/visuals/shared-food.svg)

[Open the shared food diagram at full size](../../../docs/tutorial/path/visuals/shared-food.svg).

Rust's borrowing rules do not express the invariant “these two meals together
must fit the supply.” They control access to values. Our transfer helper and
resolution order express the resource rule. Sorting stale awards would not
repair the account; the second call must use the real remainder.

The live adapter will also have work to do before a request joins this ordered
set. The animal must be an eligible grazer, its target must resolve to live
nonempty grass, and their simulation cells must match. Sprite overlap cannot
establish contact. Flint's `Energy` component does not make his hunter role
eligible to eat grass.

The next reference begins after those checks. Its slice contains already
eligible eaters, so it isolates resolution rather than pretending to exercise
an ECS adapter.

## Resolve the meals, then record what happened

This is the exact complete reference from
[session 02](../../../docs/tutorial/path/02-a-shared-meal.md). It includes a
local copy of the same helper so the example runs independently. `Meal` and
`resolve_meals` belong to this reference; they are not additional APIs to add
to the current live project.

Read `resolve_meals` first. It orders the slice, then calls the transfer helper
once for each eater using the same mutable patch. The returned amount decides
whether an actual meal belongs in the result vector. The longer test beneath
it makes the ordering and records observable.

**Reading excerpt — `resolve_meals` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
fn resolve_meals(eaters: &mut [(SimId, Energy)], patch: &mut FoodPatch) -> Vec<Meal> {
    eaters.sort_by_key(|(id, _)| *id);
    let mut meals = Vec::new();
    for (id, energy) in eaters {
        let consumed = eat_from_patch(energy, patch, 4);
        if consumed > 0 {
            meals.push(Meal {
                eater: *id,
                consumed,
            });
        }
    }
    meals
}
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 02
```

<details class="worked-reference">
<summary>Complete meal resolution answer and test (109 lines)</summary>

<!-- moss-example: session-02 -->
```rust
use moss_sim::{Energy, FoodPatch, SimId};

#[derive(Debug, PartialEq, Eq)]
struct Meal {
    eater: SimId,
    consumed: u32,
}

fn eat_from_patch(energy: &mut Energy, patch: &mut FoodPatch, bite_limit: u32) -> u32 {
    let room = energy.capacity.saturating_sub(energy.reserve);
    let consumed = room.min(patch.biomass).min(bite_limit);
    energy.reserve = energy
        .reserve
        .checked_add(consumed)
        .expect("meal energy overflow");
    patch.biomass -= consumed;
    consumed
}

fn resolve_meals(eaters: &mut [(SimId, Energy)], patch: &mut FoodPatch) -> Vec<Meal> {
    eaters.sort_by_key(|(id, _)| *id);
    let mut meals = Vec::new();
    for (id, energy) in eaters {
        let consumed = eat_from_patch(energy, patch, 4);
        if consumed > 0 {
            meals.push(Meal {
                eater: *id,
                consumed,
            });
        }
    }
    meals
}

#[test]
fn session_02_order_is_a_rule_not_an_accident() {
    for ids in [[SimId(1), SimId(7)], [SimId(7), SimId(1)]] {
        let mut eaters = ids.map(|id| {
            (
                id,
                Energy {
                    reserve: 0,
                    capacity: 100,
                },
            )
        });
        let mut patch = FoodPatch {
            name: "Meadow",
            biomass: 5,
        };
        let meals = resolve_meals(&mut eaters, &mut patch);
        assert_eq!(eaters[0].0, SimId(1));
        assert_eq!(eaters[0].1.reserve, 4);
        assert_eq!(eaters[1].0, SimId(7));
        assert_eq!(eaters[1].1.reserve, 1);
        assert_eq!(patch.biomass, 0);
        assert_eq!(
            meals,
            vec![
                Meal {
                    eater: SimId(1),
                    consumed: 4
                },
                Meal {
                    eater: SimId(7),
                    consumed: 1
                },
            ]
        );
    }

    let hungry = Energy {
        reserve: 0,
        capacity: 100,
    };
    let mut eaters = [(SimId(7), hungry), (SimId(1), hungry)];
    let mut patch = FoodPatch {
        name: "Meadow",
        biomass: 4,
    };
    let meals = resolve_meals(&mut eaters, &mut patch);
    assert_eq!(eaters[0].1.reserve, 4);
    assert_eq!(eaters[1].1.reserve, 0);
    assert_eq!(patch.biomass, 0);
    assert_eq!(
        meals,
        vec![Meal {
            eater: SimId(1),
            consumed: 4,
        }]
    );

    let mut fern = [(
        SimId(1),
        Energy {
            reserve: 33,
            capacity: 100,
        },
    )];
    let mut meadow = FoodPatch {
        name: "Meadow",
        biomass: 80,
    };
    fern[0].1.reserve = fern[0].1.reserve.saturating_sub(1); // Maintenance.
    let meals = resolve_meals(&mut fern, &mut meadow); // Already at the target.
    assert_eq!(fern[0].1.reserve, 36);
    assert_eq!(meadow.biomass, 76);
    assert_eq!(meals[0].consumed, 4);
}
```

</details>

The parameter `&mut [(SimId, Energy)]` borrows a slice: a sequence of pairs whose
length is supplied by the caller. It permits the resolver to reorder the pairs
and update their energy. The function does not need to own the array used in
the test, and it does not need a special two-animal version.

Iterating this mutable slice yields a mutable reference to each pair. Rust's
pattern matching lets `(id, energy)` name the fields through that reference:
here `id` is `&mut SimId` and `energy` is `&mut Energy`. Passing `energy` to
`eat_from_patch` therefore lends the animal's existing reserve to the helper.
The record uses `*id` to copy the small identity value out; it does not keep a
borrow into the slice after the meal.

In `sort_by_key(|(id, _)| *id)`, the method lends the closure a pair and the
tuple pattern gives its ID a name. `*id` copies the small `SimId` value to use
as a sorting key; `_` ignores the energy while choosing that key. `SimId`'s
ordering is defined on its stored numeric value. The array's later `map`
closure has a different input: it receives each ID by value and constructs
the corresponding pair. The closure bars look the same, but the calling
method determines what arrives between them.

During the resolution loop, each temporary borrow used by `eat_from_patch`
finishes before the next call. Meadow's mutation persists. ID 7 therefore
receives a reference to a patch containing one, not another copy of the
original five. The `hungry` value in the next case is copied into two
independent `Energy` values because that type implements `Copy`; changing
one reserve does not change the other. Copying initial state is useful here.
Using a copied observation as permission to spend shared food would be a
different operation.

### Reversing the inputs makes the policy visible

The test starts once with IDs `[1, 7]` and once with `[7, 1]`. Each setup gives
both animals room to eat and reconstructs the five-biomass patch. Both must
produce the same two meals and the same zero remainder. The resolver also
sorts the array in place, so the following index-based assertions first check
which ID occupies each position.

That last detail matters when moving to ECS. A production regression should
look up animals by their stable IDs rather than assume a query returns them
in the same order as this sorted local array. The future installed test also
sets maintenance to zero and starts both animals on the patch. Those choices
let reserves 4 and 1 reveal the meal allocations without unrelated upkeep or
travel changing the numbers.

Reversing spawn order is a useful challenge, but it is not sufficient evidence
by itself that an ECS system has an explicit ordering rule. The resulting
query order might happen to be the same in both fixtures. Review must also
find the sorting step in the installed resolver and ensure each transfer
reads the current patch. A green example is strongest when we know which
incorrect explanation it rejects.

The four-biomass case asks a different question. Both hares enter resolution,
but ID 1 consumes everything. ID 7 receives zero and generates no `Meal`
record. Eligibility and intention do not guarantee an outcome. Recording an
attempt as a second meal would make the history claim something the transfer
never did.

## A meal is not the tick's net gain

The last reference case brings us back to Fern alone at Meadow. She begins
with reserve 33 and the patch holds 80. The example explicitly applies one
maintenance subtraction, then resolves a meal without travel:

| Change | Fern's reserve |
| --- | --- |
| Start already at Meadow | 33 |
| Maintenance spends one | 32 |
| Meal supplies four | 36 |

Fern finishes three energy units above her starting reserve, but her meal
supplied four. Calling the meal “three” would assign maintenance's cost to the
wrong event. The returned transfer amount remains 4; Meadow's loss of four
agrees with it.

This reference uses an explicit subtraction to arrange the tick's accounting.
It does not run the installed maintenance system or prove that the live
schedule orders maintenance before eating. Those are separate integration
checks. Its narrower lesson is still useful: an outcome should record the
amount the action actually produced, not infer that amount from the final
reserve after other actions have contributed.

The command beside [the ordering reference](#resolve-the-meals-then-record-what-happened)
expects one named test to pass. It checks the existing guide's complete answer
in a temporary workspace. The
[evidence summary](../../../docs/tutorial/path/verification.md#what-each-reference-establishes)
states the limit: these are already eligible eaters, with local `Meal` records;
the example does not establish ECS eligibility, journal wiring or browser
behavior.

## What changes when the meal joins the journey?

After the live helper is reviewed, the agent prepares the same-cell adapter,
explicit competition resolution, actual-outcome records and installed tests.
The intended first meal schedule is **maintenance → movement → eating →
complete tick**. It lets arrival and a meal happen in the same tick. Food
choice, regrowth and starvation are separate later rules.

The browser observation returns to the original nine-cell journey. Movement
alone left Fern at Meadow with reserve 33. With eating added after movement,
the arrival tick should instead finish at **37**, and Meadow should hold
**76 biomass**. These are expected values after activation, not observations
of today's unfinished live build.

The same-cell case above begins at 33 and spends another maintenance unit
before eating, so it finishes at 36. The arrival case's 33 already includes
that ninth tick's maintenance and movement; adding its meal gives 37. Naming
the starting point explains the apparent one-unit disagreement.

For the future browser check, Reset, select Fern, and Step through arrival.
Inspect both her reserve and Meadow's biomass, then inspect the recorded
meal. A change in one value alone cannot show that the transfer is correct.
The agent reconciles that observation with the helper and installed tests;
your selected checkpoint remains the one named in `NOW.md`.

Fern's destination now has a possible benefit as well as a travel cost.
Meadow's dwindling remainder explains why the next hare may get less, and
the returned amount lets the record say exactly what happened. We can keep
that transfer intact while asking the next question: who chooses the
destination when the fixture stops choosing it for Fern?
