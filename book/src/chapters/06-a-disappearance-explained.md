<p class="eyebrow">Chapter 6 · Lifecycle, removal and evidence</p>

<a id="6-a-disappearance-the-world-can-explain"></a>

# A disappearance the world can explain

Fern reaches zero during maintenance. Should she disappear now? There is food
under her feet, and eating has not had its turn. If the terminal check runs
before that meal, she dies beside food she could have consumed. If it runs
afterward, she may finish the tick with a replenished reserve.

Both orders are implementable. They describe different worlds.

Until now, saturating subtraction has answered an arithmetic question: how much
reserve remains after a bounded drain? It has not answered the lifecycle
question: when does an empty reserve end an animal's participation? To make a
disappearance explainable, we must choose that boundary, resolve it once, and
retain evidence whose meaning survives the removal.

**Proposed policy and worked references.** Animals in the current live
simulation remain present at zero energy. This chapter investigates a future
after-meal starvation rule, then the removal and evidence it would require.
The complete examples run in isolation; they do not install that policy.

## On this page

- [Choose the terminal boundary](#give-the-last-meal-a-precise-place)
- [Remove once and retain the identity](#removing-the-entity-is-another-operation)
- [Read evidence within its coverage](#a-retained-event-is-evidence-with-a-boundary)
- [Reconcile the population](#the-population-needs-its-own-account)
- [Compare relevant state across runs](#compare-two-worlds-without-comparing-their-camera-positions)

Read the short rule excerpts and traces for the argument. Expand a complete
answer when you want its imports, supporting types and test fixtures; each
reference has its own command beside it.

## Give the last meal a precise place

The policy explored here is **resolve starvation after any allowed meal, when
the animal's reserve is still zero**. Maintenance comes earlier and spends up
to the available reserve. A meal that the animal is already eligible to take
can replenish it before the terminal check.

This permits a last-chance meal. It does not grant an unaffordable move, change
a hunter into a grazer, or resurrect an animal already removed by a different
terminal interaction. Eligibility still belongs to the action rules. The
starvation predicate examines the state they leave behind.

The timing is a model choice rather than a statement about real physiology.
An alternative could make reaching zero during maintenance immediately terminal.
That rule would prohibit the rescue. A test should preserve whichever boundary
is selected; the incidental position of a system in a schedule should not
select it silently.

Put Fern directly on Meadow with reserve **1**, capacity **100**, maintenance
cost **1**, and **4 stored biomass**. Use executing tick **120**, when the
proposed light cycle has entered night. No new grass grows. Maintenance takes
reserve to zero, but no movement is needed to eat. The meal supplies four;
the final reserve is four, so the after-meal predicate says she survives.

Now reconstruct that starting situation with no stored food. The night still
produces nothing. Maintenance leaves zero, the meal transfers zero, and the
terminal predicate is true. Move the same empty-patch setup to dawn at executing
tick **240**, however, and growth can supply one before the meal. Fern consumes
that new unit and survives with one.

| Starting situation: Fern has 1 reserve and is already on Meadow | Result under the proposed composition |
| --- | --- |
| Night, 4 stored biomass | No growth; maintenance leaves 0; eating leaves reserve 4 and biomass 0. |
| Night, no stored biomass | No growth or meal; reserve remains 0, so starvation is proposed. |
| Dawn, no stored biomass | Growth supplies 1; maintenance leaves 0; eating leaves reserve 1 and biomass 0. |

An empty patch at the start of a tick is therefore not enough information to
predict starvation. We also need to know whether production occurs before
choice and eating. Conversely, night removes new light-driven production;
it does not erase food already stored in a patch.

### Checkpoint: the predicate sees the completed meal

The complete reference below includes a local `contact_tick` to make that
order visible. It assumes the animal is already in contact with eligible food.
It does not run autonomous choice, movement, Moss's installed schedule or an
ECS removal system. `is_starved_after_meals` returns a Boolean; it neither
despawns the animal nor writes an event.

Read the first three cases against the table. Each assigns Fern a fresh
reserve of one, while the patch values follow the preceding result. The final
loop begins a separate trace whose maintenance request is larger.

**Reading excerpt — `is_starved_after_meals`, `contact_tick` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
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
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 13
```

<details class="worked-reference">
<summary>Complete after-meal starvation answer and test (63 lines)</summary>

<!-- moss-example: session-13 -->
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

</details>

The executing tick selects the light phase. The sequence inside `contact_tick`
then grows food, deducts maintenance, transfers a bounded meal and finally
checks for zero. That order supplies the meaning of the tiny predicate. A
function body containing `energy.reserve == 0` cannot tell us by itself which
earlier opportunities the animal was allowed.

The test's dawn case also places a requirement on future integration. If a
patch begins empty, growth must make it visible to choice before the animal's
meal is resolved. A correct final predicate cannot rescue an animal that the
adapter incorrectly excludes from eating at zero. The isolated reference
starts beyond those eligibility decisions, so installed checks must establish
them separately.

### A cost can request more than the animal pays

The final trace exposes another consequence of the proposed rules. Fern begins
with reserve one, but maintenance requests two. Saturating subtraction removes
only the one she has. Daylight grows one biomass, which she eats, returning her
reserve to one. She survives the tick.

The same cycle repeats over executing ticks 240 through 359. Each requests two
maintenance units, actually deducts one, and receives one from food. At tick
360, night supplies no replacement; reserve finishes at zero and the proposed
predicate becomes true.

This is a no-debt model. The unpayable portion of maintenance is not retained
as an obligation, and the predicate cannot recover that missing information
by looking at the final reserve. A configured rate of two is a bounded drain
under these rules, not a guarantee that two units must be obtained every tick
to survive. If the intended biology requires full payment, revise the composed
rule before adopting starvation. Adding `reserve == 0` at the end does not
enforce full payment on its own.

Outcome records must preserve that distinction. A request for two is not
evidence that two were spent. Likewise, a reserve that rises by three after
maintenance and a meal may have received a meal of four. Record the actual
deduction or transfer at the operation that knows it, rather than reconstructing
an imagined event from the final balance.

**A nearby case:** return to the night setup with reserve one and four stored
biomass, but put Fern one cell away instead of on Meadow. Travel costs two
energy units per accepted cell. Does the stored food still rescue her?

**Answer:** under the stated movement and contact rules, it does not.
Maintenance leaves zero, the charged step is unaffordable, and same-cell
contact is absent. No meal occurs; the proposed after-meal predicate is true,
and Meadow keeps its four biomass. This is an expected consequence of the
composed rules, not a case executed by `contact_tick`, which assumes contact.
Available supply and accessible supply can tell different stories.

## Removing the entity is another operation

Once the policy says a creature is terminal, the world must commit that result.
An energy check reads a component. Despawning changes which entities and
components exist. Keeping those operations in two phases both clarifies the
Rust borrows and gives the outcome one place to be recorded.

The existing [reset implementation](../../../crates/moss-sim/src/simulation.rs)
already uses the relevant shape: collect entity handles, finish the query, then
despawn the collected entities. The terminal resolver additionally keeps each
animal's stable `SimId`, because the live handle and the historical identity
serve different purposes.

A Bevy `Entity` handle lets us address a live entity in this world. Copying that
handle produces an owned handle value; it does not transfer ownership of the
animal out of the ECS. A copied `SimId` identifies the participant in the
simulation's records after its components are gone. When a record leaves the
current run, its identity also needs the run number.

### Checkpoint: collect values that can outlive the query

The complete reference uses actual Bevy query and removal operations. It selects
zero-reserve creatures and copies only their handles and stable IDs, then
removes them in ascending stable-ID order. The returned vector reports the IDs
whose removal actually succeeded. It is input for a future journal caller,
not another history store.

**Reading excerpt — `remove_starved` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
fn remove_starved(world: &mut World) -> Vec<SimId> {
    // Phase 1: inspect components and copy only the handles needed afterward.
    let mut terminal: Vec<_> = {
        let mut query = world.query_filtered::<(Entity, &SimId, &Energy), With<Creature>>();
        query
            .iter(world)
            .filter(|(_, _, energy)| energy.reserve == 0)
            .map(|(entity, id, _)| (entity, *id))
            .collect()
    };
    terminal.sort_by_key(|(_, id)| *id);

    // Phase 2: no query borrow remains, so changing the world's structure is safe.
    let mut removed = Vec::new();
    for (entity, id) in terminal {
        if world.despawn(entity) {
            removed.push(id);
        }
    }
    removed
}
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 14
```

<details class="worked-reference">
<summary>Complete terminal removal answer and test (48 lines)</summary>

<!-- moss-example: session-14 -->
```rust
use bevy_ecs::prelude::*;
use moss_sim::{Creature, Energy, SimId};

fn remove_starved(world: &mut World) -> Vec<SimId> {
    // Phase 1: inspect components and copy only the handles needed afterward.
    let mut terminal: Vec<_> = {
        let mut query = world.query_filtered::<(Entity, &SimId, &Energy), With<Creature>>();
        query
            .iter(world)
            .filter(|(_, _, energy)| energy.reserve == 0)
            .map(|(entity, id, _)| (entity, *id))
            .collect()
    };
    terminal.sort_by_key(|(_, id)| *id);

    // Phase 2: no query borrow remains, so changing the world's structure is safe.
    let mut removed = Vec::new();
    for (entity, id) in terminal {
        if world.despawn(entity) {
            removed.push(id);
        }
    }
    removed
}

#[test]
fn session_14_removal_reports_each_terminal_id_once() {
    let mut world = World::new();
    for (id, reserve) in [(2, 0), (9, 5), (1, 0)] {
        world.spawn((
            SimId(id),
            Creature { name: "test hare" },
            Energy {
                reserve,
                capacity: 100,
            },
        ));
    }
    assert_eq!(remove_starved(&mut world), vec![SimId(1), SimId(2)]);
    assert!(remove_starved(&mut world).is_empty());

    let remaining: Vec<_> = world
        .query::<(&SimId, &Energy)>()
        .iter(&world)
        .map(|(id, energy)| (*id, energy.reserve))
        .collect();
    assert_eq!(remaining, vec![(SimId(9), 5)]);
}
```

</details>

The resolver keeps handle and ID values when it crosses from inspecting
components to removing entities:

| Phase | What the resolver holds |
| --- | --- |
| Inspect | Each query row supplies an entity handle and borrowed `SimId` and `Energy` components. The filter reads the reserve. |
| Keep candidates | The mapping yields owned `(Entity, SimId)` pairs; `collect()` gathers them into a vector with no references into component storage. |
| Commit removal | Query iteration has finished. The loop uses the owned handles to despawn and reports an ID only after removal succeeds. |

`collect()` materializes those yielded values. It does not automatically
turn borrowed components into copies. The `*id` in the mapping performs the
copy that matters. Here `Vec<_>` can be inferred as `Vec<(Entity, SimId)>`.
By the time the removal loop begins, the query iteration has finished and
the list contains no component borrows that must survive structural mutation.

This is a reason to use a temporary collection in this operation, not a reason
to allocate a list for every ECS query. Maintenance can update each matched
reserve during ordinary iteration because it is not removing the component
storage being traversed.

The test inserts terminal IDs 2 and 1 around a surviving ID 9. Sorting makes the
reported order `[1, 2]`. Calling the resolver again returns an empty vector;
the terminal creatures are no longer present to remove. The surviving control
remains at reserve five. Without that control, a resolver that deleted every
creature could satisfy the superficial expectation that the terminal ones
disappeared.

### One resolver, one committed outcome

`World::despawn` reports whether it removed an entity. The reference appends an
ID only after success, so its return value reports actual removals rather than
the earlier candidate list. A journal caller must record those results once.
A second caller must not independently count the same transition as another
death.

The live integration has a stronger obligation than the reference: after the
death resolves, no later action may use the creature. Immediate exclusive-world
removal makes it unavailable to later queries. If a future implementation queues
removal, it must first make the creature ineligible and ensure later resolvers
honor that state. A queued despawn alone is not a reservation. This distinction
will matter when two hunters can claim one prey, even in a single-threaded
simulation.

Timing also applies to the record. During execution of tick 120, the completed
counter still describes tick 119 until completion runs. A death resolved now
belongs to executing tick `completed + 1`, with checked arithmetic, and should
be stamped 120. Initialization records remain at tick zero. Future integration
must verify the event stamp, the completed counter afterward, and the absence
of another death or later action by the removed creature on subsequent ticks.

The removal reference does not run those journal or full-schedule checks.
Its narrower contribution is visible and useful: owned candidates permit
structural mutation, stable sorting gives a reproducible report, successful
removal is reported once, and a living control survives.

## A retained event is evidence with a boundary

The current [journal](../../../crates/moss-sim/src/journal.rs) retains **32**
entries and reports how many have been evicted. Its event kinds currently
describe run initialization and authored placement. Death records, whole-run
death counters and historical creature summaries remain future integration.
Reading the planned record examples below must not make that missing collection
look like a live feature.

An event can link participants to one actual outcome. A future meal record,
for example, can associate the eater and patch with the amount transferred.
It should not create contradictory copies of the interaction in two biographies.
An inspector can derive each participant's view from that shared evidence.

The [observability design](../../../docs/design/observability.md) separates four
kinds of information because they answer different questions:

| Information | What it can establish |
| --- | --- |
| Current state | Fern's reserve or target now. |
| An actor's retained observation | What that actor previously perceived, if that memory is modeled. |
| A semantic outcome record | A particular meal or terminal outcome actually resolved. |
| A population measurement | A count or distribution sampled at a named time and scope. |

A population sample cannot reconstruct every event between samples. A final
reserve cannot reveal every input an earlier choice function read. A nearby
timestamp is not proof that one event caused another. Use the relation or
decision input actually recorded; otherwise leave the explanation bounded.

### “First retained” is different from “complete since”

Suppose 35 outcomes are recorded on tick 120. A 32-entry journal drops the first
three while retaining 32 later entries from that same tick. Its oldest retained
tick is still 120.

```text
Tick 120:  three earlier records evicted | 32 later records retained
```

“First retained event: tick 120” describes the surviving evidence. “Complete
since tick 120” would overstate it. A timestamp alone cannot distinguish events
inside that boundary tick. Sequence identifiers could make a future retained
suffix more precise, but they are not fields in the current journal.

Collection has a separate boundary from retention. The present journal can
retain every initialization entry, with zero evictions, while maintenance
changes both animals from 60 to 57. Maintenance changes are not collected there.
Zero evictions means no recorded details were dropped; it does not mean every
change has a detail.

The distinction becomes consequential when reading deaths. A whole-run counter
initialized at run start may cover every resolved death even after some event
details have been evicted. If that counter is absent, one visible death record
and three evicted events do not tell us the death total. Those three missing
events might have been meals, placements or deaths. Adding the eviction count
to visible death rows would invent a result.

Use “zero” when the relevant collection covers the scope and recorded none.
Use “unknown” or “uncollected” when it does not. A small honest record can
explain more than a full-looking timeline whose gaps are hidden.

![One future death is reflected in a living count of five and an independent whole-run death count of one. Fern's detail remains in a 32-entry journal, while three earlier records from the same tick have been evicted. The retained detail is partial even though the independent counter covers the run.](../assets/a-life-and-its-evidence.svg)

[Open the history and population diagram at full size](../assets/a-life-and-its-evidence.svg).

*One illustrative future outcome, three views. The current journal does not yet
collect deaths or this counter. If both are added, losing a detail must not erase
the aggregate fact; the living count should still be measured from the world.*

## The population needs its own account

Now consider a future scarcity experiment with six hares and one patch, rather
than the earlier general population scene. Each hare costs one maintenance unit
per tick. The patch can produce at most one biomass per daylight tick, converted
1:1 into animal energy.

While all six survive under the proposed end-of-tick rule, each ends a tick
with at least one reserve and can pay the next unit of maintenance. Their
combined maintenance spends six per tick. Even continuous daylight and perfect
access could supply at most one new unit from that patch. Night and travel can
worsen the deficit. Initial animal reserves and stored biomass delay its
consequences; deaths eventually reduce demand. They do not make the original
six-animal budget sustainable indefinitely.

That argument deliberately uses a unit maintenance cost. The earlier cost-two,
reserve-one trace showed that larger requested rates can exceed actual drains.
Adding those nominal requests would not by itself prove how much nutrition the
no-debt model requires. Explain the actual run using deductions and transfers
that occurred.

The budget also does not name the first animal to die or its death tick.
Distance, target choice, capacity, stored food and conflict priority affect
which individuals obtain the limited supply. A population total identifies
a constraint; individual evidence explains how that constraint was experienced.

Before interpreting a population curve, reconcile who exists. This scenario
has no births, manual additions or other removals, so its account is:

```text
living hares = seeded hares - resolved deaths
```

Initialization is not reproduction. Keeping a separate births term, explicitly
zero here, gives later offspring their own meaning. If future interventions add
or remove animals, those operations need their own terms too.

### Checkpoint: reject an impossible ledger

The complete reference below receives illustrative counts. They are not samples
from a simulated scarcity run and do not predict a collapse schedule. The
calculation asks whether the supplied account is possible and agrees with the
independently observed living count.

**Reading excerpt — `population_account_agrees` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
fn population_account_agrees(sample: &PopulationSample) -> bool {
    sample
        .seeded
        .checked_add(sample.births)
        .and_then(|total| total.checked_sub(sample.deaths))
        == Some(sample.living)
}
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 15
```

<details class="worked-reference">
<summary>Complete population accounting answer and test (49 lines)</summary>

<!-- moss-example: session-15 -->
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

</details>

Six seeded, no births and three deaths can agree with three living animals.
Six seeded and seven deaths is impossible under the stated scope, even if the
reported living count is zero. Clamping the subtraction would hide that error
by making an impossible account resemble an empty world.

`checked_add` and `checked_sub` instead return `Option` results. After a valid
sum, `and_then` supplies it to the subtraction and keeps that subtraction's
own optional result. A sum overflow or excessive deaths yields `None`, which
cannot equal `Some(sample.living)`. The last test checks overflow using a
nonzero birth count to challenge the arithmetic; it does not introduce
reproduction into the actual scarcity scenario.

This revisits the meaning of absence. No eligible food was one reason to
return `None`; an arithmetic account that cannot be represented within its
bounds is another. The surrounding function gives the value its meaning.
Using `map` for this particular second calculation would retain another layer,
because `checked_sub` already returns an `Option`; `and_then` combines the
steps without nesting their wrappers.

The equation's inputs also need independence. If we compute both the death
count and “living” as two versions of `seeded - deaths`, they will agree even
when an allegedly removed creature remains in the world. Count living hares
directly from the ECS after the relevant outcomes commit. Use a death counter
covering the same run and sample boundary. A truncated detail list cannot
substitute for that counter.

### A correct account can describe a harsh world

For an installed scarcity observation, begin with a short fixed window and
record the starting population, stores, costs, production limit and light phase.
Follow one actual meal or death while checking the population account at the
end. Keep event-detail coverage beside the measurement's scope.

If the account disagrees, investigate collection timing, missing outcomes or
an entity that survived its supposed removal. Preserve the invariant while
finding the cause. If the account agrees and several hares died, the scenario
may simply be severe. Whether the competition is interesting is a further
question. Correctness, sustainability and enjoyment need different evidence.

Doubling requested production would be a controlled change only if the other
starting conditions stayed fixed. Capacity can limit actual growth; distance
and appetite can limit consumption. A doubled request therefore does not promise
twice the meals or a particular survival time. The population account must still
reconcile whichever outcomes occur.

## Compare two worlds without comparing their camera positions

An explainable run raises another question: can the same starting state,
accepted inputs and executed ticks produce the same relevant result again?
First define what will be compared. ECS iteration order and browser drawing
order are not stable orders for an animal's story.

A canonical snapshot gathers selected simulation values and sorts them by
stable ID. Equivalent values then appear in the same order. Leave runtime
entity handles and view transforms out of a comparison intended to describe
biological state. Retain a field when changing it could matter to the claim,
even if it would not change a screenshot immediately.

An accepted food target is such a field. Fern at the same position with the
same reserve can have a different next instruction if her target changes from
patch 3 to patch 4. A picture of the current cell would miss that distinction.

### Checkpoint: equal values despite different insertion order

The reference constructs two small worlds with equivalent component values
inserted in opposite orders. It does not execute ticks. Its local
`AnimalSnapshot` is a read model, not a component and not a second owner of
animal state.

After establishing equality, the test also checks a complete literal expected
vector. Then it changes one reserve and one target in separate steps to show
that the comparison notices those changes.

**Reading excerpt — `snapshot` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
fn snapshot(world: &mut World) -> Vec<AnimalSnapshot> {
    let mut animals: Vec<_> = world
        .query_filtered::<(&SimId, &Position, &Energy, Option<&FoodTarget>), With<Creature>>()
        .iter(world)
        .map(|(id, position, energy, target)| AnimalSnapshot {
            id: *id,
            position: *position,
            reserve: energy.reserve,
            target: target.map(|target| target.0),
        })
        .collect();
    animals.sort_by_key(|animal| animal.id);
    animals
}
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 16
```

<details class="worked-reference">
<summary>Complete canonical snapshot answer and test (87 lines)</summary>

<!-- moss-example: session-16 -->
```rust
use bevy_ecs::prelude::*;
use moss_sim::{Creature, Energy, FoodTarget, Position, SimId};

#[derive(Debug, PartialEq, Eq)]
struct AnimalSnapshot {
    id: SimId,
    position: Position,
    reserve: u32,
    target: Option<SimId>,
}

fn example_world(reverse: bool) -> World {
    let mut world = World::new();
    let mut rows = [
        (SimId(1), Position { x: 10, y: 10 }, Some(SimId(3))),
        (SimId(2), Position { x: 21, y: 6 }, None),
    ];
    if reverse {
        rows.reverse();
    }
    for (id, position, target) in rows {
        let mut entity = world.spawn((
            id,
            position,
            Creature { name: "example" },
            Energy {
                reserve: 57,
                capacity: 100,
            },
        ));
        if let Some(id) = target {
            entity.insert(FoodTarget(id));
        }
    }
    world
}

fn snapshot(world: &mut World) -> Vec<AnimalSnapshot> {
    let mut animals: Vec<_> = world
        .query_filtered::<(&SimId, &Position, &Energy, Option<&FoodTarget>), With<Creature>>()
        .iter(world)
        .map(|(id, position, energy, target)| AnimalSnapshot {
            id: *id,
            position: *position,
            reserve: energy.reserve,
            target: target.map(|target| target.0),
        })
        .collect();
    animals.sort_by_key(|animal| animal.id);
    animals
}

#[test]
fn session_16_snapshots_ignore_order_but_detect_state_changes() {
    let mut first = example_world(false);
    let mut reordered = example_world(true);
    let expected = snapshot(&mut first);
    assert_eq!(expected, snapshot(&mut reordered));
    assert_eq!(
        expected,
        vec![
            AnimalSnapshot {
                id: SimId(1),
                position: Position { x: 10, y: 10 },
                reserve: 57,
                target: Some(SimId(3)),
            },
            AnimalSnapshot {
                id: SimId(2),
                position: Position { x: 21, y: 6 },
                reserve: 57,
                target: None,
            },
        ]
    );

    let fern = reordered
        .query_filtered::<Entity, With<FoodTarget>>()
        .single(&reordered)
        .unwrap();
    reordered.get_mut::<Energy>(fern).unwrap().reserve = 56;
    assert_ne!(expected, snapshot(&mut reordered));

    reordered.get_mut::<Energy>(fern).unwrap().reserve = 57;
    reordered.get_mut::<FoodTarget>(fern).unwrap().0 = SimId(4);
    assert_ne!(expected, snapshot(&mut reordered));
}
```

</details>

`Option<&FoodTarget>` is important: an animal without that component remains
in the query, with `None` for its target. Requiring `&FoodTarget` would silently
exclude the targetless animal. The mapping copies each small value into an
owned snapshot. `target.map(|target| target.0)` converts an optional borrow
into an optional copied stable ID, so no target reference needs to survive
after collection.

The mutable world parameter is needed to create query state here. The requested
component accesses are read-only; this snapshot function changes no animal
component. Owning its returned data allows the test to retain a “before” result
while subsequently mutating a world and constructing another snapshot.

The literal expected vector protects against two equally empty queries
comparing equal. The later unequal comparisons protect against a snapshot
that forgets one of the fields it claims to observe. Sorting only removes
incidental order differences; it cannot make an incomplete selection complete.

For example, this snapshot omits capacity. Two animals with equal reserve but
different capacities would compare equal here, although a later meal could
be limited differently. That is acceptable for the reference's stated checks.
It would not support a claim that all state relevant to future meals matched.

### What a repeated run would need to establish

A future installed comparison must include the completed behaviors' relevant
state: patch biomass, activity, individual costs, clock and light, as well as
positions, reserves, targets and outcomes. If pending work or randomness can
influence continuation, those inputs also belong in the claim. A seed alone
does not contain the entire simulation.

The model already provides a useful composed test case. Begin at completed
tick 239 with an empty patch and two seeking hares, each at reserve one,
maintenance one and travel cost two. Give neither an authored target. Put
Fern on the patch and the other hare one cell away. Under the proposed
after-meal policy, dawn tick 240 should grow one biomass, maintenance should
leave both at zero, and only Fern should eat without paying for movement.

The expected result is one unit grown, one eaten, Fern alive at reserve one,
patch biomass zero, and the other hare removed with one starvation record.
This is a future integration target, not an outcome established by the
snapshot reference. It needs growth, choice, action permissions, terminal
resolution and recording to agree. A different accepted terminal policy
would require a different expectation.

Reset adds another distinction. It advances the run number and recreates the
authored scene. Equivalent biological values can be compared across two such
runs while accounting for that expected metadata difference. Their historical
identities remain different: `(run number, SimId)` preserves which life an
event belonged to. The current reset also discards the previous journal; it
does not provide a cross-run archive.

A browser comparison can then vary panning, zooming and pauses while preserving
the same accepted simulation inputs and executed ticks. It must compare the
resulting state, not equal wall time or screenshots. That checks a connection
the native snapshot example never exercises: browser controls and rendering
must leave the biological outcome independent of the view.

Even a successful bounded comparison would not establish a general replay or
save/resume system. Resuming requires enough authoritative state and versioned
context to continue, including pending actions and random state when present.
A record that helps explain an event and a record that can restart its world
are different artifacts.

## Inspecting the four complete references

Each complete answer has its exact command beside it. These commands run the
canonical printed examples in temporary workspaces, using the pinned toolchain
and cached dependencies. Each expects one named reference test to pass.

The [first](#checkpoint-the-predicate-sees-the-completed-meal) checks an
assumed-contact composition; the [second](#checkpoint-collect-values-that-can-outlive-the-query)
performs real Bevy removal; the [third](#checkpoint-reject-an-impossible-ledger)
checks supplied population counts; the [fourth](#checkpoint-equal-values-despite-different-insertion-order)
compares selected snapshot fields. None installs the future lifecycle in the live
project. Installed schedule tests, journal integration and browser observations
remain separate checks.

Fern's absence can now have a precise explanation: an explicitly timed rule
found a terminal reserve, one resolver removed the animal, and the records
describe the outcome within their visible coverage. Where details are missing,
the explanation can say so. A later hunt, rest rule or birth will add different
decisions, but it will face the same practical question: what changed, when
did it become authoritative, and which evidence lets us tell?
