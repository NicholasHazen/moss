<p class="eyebrow">Chapter 4 · Individual defaults and controlled comparisons</p>

<a id="4-two-hares-one-fair-comparison"></a>

# Two hares, one fair comparison

Suppose Fern and another hare start with 60 energy. Three ticks later, one has
57 and the other has 54. Have we made the second hare more expensive to sustain?
Perhaps. It might also have walked farther, missed a meal, or started with a
different reserve. The final numbers give us something to explain, but they do
not yet identify the cause.

We will make one difference deliberate: Fern pays one maintenance unit per
tick, while the other hare pays two. Both remain hares. That modest change gives
the questions about attributes a concrete home. Where should the default live?
What does an override mean? Does every tick search for a value through a chain
of fallbacks? If an animal eventually has speed, level and temporary effects,
does everything belong in one bag of numbers?

Start with the behavior we need. Each hare must own a rate that maintenance can
read, and a comparison must show that rate making a difference. We can decide
the representation by following those two obligations through the code.

**Worked references · future live behavior.** The current maintenance system
still reads shared species rates. Authored populations and individual cost
components are proposed later steps; this chapter's examples run independently
of the live schedule. The local repository's `NOW.md` continues to select the
active edit.

## On this page

- [Summarize several individuals](#a-population-gives-the-numbers-some-company)
- [Turn defaults into owned values](#a-default-is-an-instruction-for-making-an-individual)
- [Make maintenance read those values](#make-the-query-read-the-new-owner)
- [Choose useful component boundaries](#how-many-components-should-these-attributes-become)
- [Compare the same journey](#give-the-difference-a-fair-journey)

Read the short rule excerpts and traces for the argument. Expand a complete
answer when you want its imports, supporting types and test fixtures; each
reference has its own command beside it.

For one path from an authored cost to its visible effect, start with
[defaults and owned values](#a-default-is-an-instruction-for-making-an-individual),
continue through [the maintenance query](#make-the-query-read-the-new-owner),
then follow [the two-hare journey](#give-the-difference-a-fair-journey).
The population summary and discussions of component boundaries, modifiers and
scale are adjacent investigations you can return to after that account makes sense.

## A population gives the numbers some company

One animal makes a rule easy to follow. Several animals make its consequences
worth comparing. The proposed population scene supplies six hares, two foxes
and four grass patches with authored IDs and positions. It does not require
reproduction or random placement. Recreating those starting conditions gives
each later experiment a repeatable place to begin.

Fern is an individual nickname; Hare is her species; Grazer is her ecological
role. A species summary groups hares even when their names, targets and reserves
differ. Bevy's *archetype* answers another question: which component types does
an entity carry? Two hares with different energy values can have the same
archetype. Adding a `FoodTarget` changes a component set without changing the
animal's species.

A summary should help us find a story without replacing its participants.
Consider two groups:

| Individual reserves | What a mean of 50 conceals |
| --- | --- |
| 50 and 50 | Both animals have the same reserve. |
| 0 and 100 | One animal has no reserve; the other is full. |

Those groups can have different affordable actions on the next tick. Reporting
count, minimum and maximum alongside the mean exposes that difference. We can
then inspect the individual animals to understand it. The summary reads their
authoritative values; changing a displayed mean must never become another way
to change their energy.

An empty group has a different answer again. It has no mean, minimum or maximum.
Displaying zero would make “there are no hares” resemble “the hares have no
energy.” `Option` lets the function preserve that distinction, just as it
preserved the absence of an eligible food target.

### Checkpoint: a summary with members and limits

The complete reference from
[session 05](../../../docs/tutorial/path/05-many-individuals.md) accepts a
borrowed slice of reserves. Species filtering happens before this function;
the function cannot tell whether its numbers came from hares or foxes. Its local
`EnergySummary` is reference data, not an existing live observation API.

Follow `[20, 60]` first. The total is 80, the count is two, the minimum remains
20, and the maximum becomes 60. Then look at `[20, 21]`: a mean of 20.5 is a
useful case because integer division would conceal the half unit.

**Reading excerpt — `summarize_energy` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
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
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 05
```

<details class="worked-reference">
<summary>Complete population summary answer and test (56 lines)</summary>

<!-- moss-example: session-05 -->
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

</details>

`split_first()` returns a borrowed first element and the rest of the slice.
The `?` returns `None` immediately if there is no first element. In the pattern
`(&first, rest)`, `&first` copies the referenced `u32` into a local value;
`rest` remains a borrowed slice. Neither operation removes a member from the
input.

Starting minimum and maximum from an actual member avoids inventing initial
extremes. The total uses `u64`, giving the sum more room than one `u32` reserve,
and still checks addition. Converting before division preserves fractional
means for these small populations. This display calculation is not a guarantee
of exact floating-point summaries at arbitrary scale.

The proposed scenario begins with all six hares at 60, so its initial hare
summary should be count 6, minimum 60, maximum 60 and mean 60. The four patches
begin with 80 biomass each, for 320 total. A patch is a stand of plants, not a
count of individual grass blades. Those starting values are acceptance targets
for future scenario integration, not a population already present in the live
fixture.

After some foraging, inspecting two hares can explain two particular reserves.
It cannot verify a six-hare mean. Integration must gather all six values,
confirm the species filter, and compare the derived result with the display.
A correct formula applied to the wrong members remains a wrong summary.

## A default is an instruction for making an individual

The live code already separates individual reserves from shared rates.
[`Energy`](../../../crates/moss-sim/src/energy.rs) belongs to each animal, while
`SpeciesEnergyRules` is a shared resource.
[`spend_energy`](../../../crates/moss-sim/src/lessons.rs) reads an animal's
`Species`, looks up that species' rate, and subtracts it from the animal's
reserve. That is why one hare can eat without replenishing every hare, while
the current shared hare rate applies to every hare.

For two members of one species to have different upkeep, the rate needs a
more specific owner. The proposed rule resolves a species default and an
optional authored override when an animal is created. It stores the resulting
maintenance rate in an `AnimalEnergyCosts` component on that animal.

| Moment | Information needed |
| --- | --- |
| Construct the animal's cost | Its configured species default and its optional authored override. |
| Apply maintenance later | Its owned cost component and its current energy. |

This answers the question about repeated lookups. A maintenance tick does not
need to walk an override/default chain under this design. Creation already
resolved the instruction into a concrete value. The query reads that value
beside `Energy` and applies the familiar subtraction.

That choice also defines what *default* means. A species template supplies
starting instructions; it is not a live parent whose fields are consulted
whenever an animal needs a number. If we wanted a change to the template to
immediately alter every non-overridden animal, we would need a different
propagation contract. The two contracts answer different design questions;
neither follows automatically from using ECS.

Moss's planned individual-cost policy keeps defaults fixed during a run.
Reset rebuilds animals from the selected scenario's authored inputs. Runtime
changes to an individual disappear on reconstruction, while authored overrides
are applied again. Retuning a running population would be an explicit later
operation, with a decision about which individuals it changes.

## Missing, zero and equal are three different stories

An authored override uses `Option<u32>`. The meaning of the wrapper is part of
the model:

| Authored input with species default 1 | Constructed maintenance rate |
| --- | --- |
| `None`: no individual override | 1, from the species default. |
| `Some(2)`: explicitly use two | 2. |
| `Some(0)`: explicitly use zero | 0. |
| `Some(1)`: explicitly use one | 1, from the override. |

Zero is a present value. A test such as “use the override if it is greater than
zero” would erase that intent and charge an animal that was explicitly given
free upkeep. `unwrap_or(default)` instead asks whether a value exists, then
takes it as supplied.

The last row is subtler. `None` and `Some(1)` produce the same number under a
default of one. They still give different instructions. If a later scenario
changes the default to nine, the unchanged inputs construct nine and one.
An explicit one remains an explicit one even when an earlier default happened
to match it.

![At creation, a species default and an optional authored override produce an owned cost component. Maintenance later reads that resolved component directly.](../../../docs/tutorial/path/visuals/default-to-instance.svg)

[Open the defaults and owned values diagram at full size](../../../docs/tutorial/path/visuals/default-to-instance.svg).

The arrow from template to individual happens during construction. There is
no arrow reaching back from every future maintenance subtraction. This is why
the original `Option` belongs in retained authoring data: reconstruction needs
the instruction, not only its previous result.

### At the workbench: reconstructing owned costs

Give three authored hares the inputs `None`, `Some(1)` and `Some(0)`. With an
active hare template of one, they begin with owned costs one, one and zero.
Change the draft template to nine and watch those owned values stay put. The
draft is a proposed configuration; **Start configured run** submits it and
constructs a new set of individuals.

**Reset** has a different job: it reconstructs the current active configuration.
To see that distinction, leave nine in the draft, apply a runtime cost of five
to Fern, then Reset. Fern returns to one, while the unsubmitted draft remains
nine. Starting the configured run now constructs nine, one and zero. This is
the chapter's selected future policy, illustrated independently of live Moss.

| Action in this sequence | Configuration and owned costs afterward |
| --- | --- |
| Begin | Active 1; draft 1. Fern / ID 7 / ID 8 own **1 / 1 / 0**. |
| Edit the draft to nine | Active 1; draft **9**. Owned costs stay **1 / 1 / 0**. |
| Apply runtime cost five to Fern | Active 1; draft 9. Owned costs become **5 / 1 / 0**. |
| Reset the current run | Active 1; draft 9. Owned costs return to **1 / 1 / 0**. |
| Start configured run | Active **9**; draft 9. New individuals own **9 / 1 / 0**. |

<section class="lab" data-lab="defaults" aria-label="Authored inputs and owned maintenance costs">
<div class="lab-heading"><span class="lab-kind">Independent teaching model · future policy</span><strong>A template makes three owned values</strong></div>
<p>This JavaScript illustration does not execute Rust or live Moss. All three individuals are hares. The cards retain authored inputs so they can label initialization as Default-derived or Explicit override, even when the numbers match. Costs use whole energy units per tick; the controls accept 0–12 for this illustration.</p>
<div class="lab-controls">
<label>Draft hare template <input data-input="draft-template" type="number" min="0" max="12" step="1" value="1" disabled> energy units per tick</label>
<label>Proposed runtime cost for Fern <input data-input="runtime-cost" type="number" min="0" max="12" step="1" value="5" disabled> energy units per tick</label>
</div>
<div class="lab-state" data-output="configuration"></div>
<div class="lab-actions"><button type="button" data-action="start-configured" disabled>Start configured run</button><button type="button" data-action="reset" disabled>Reset current run</button><button type="button" data-action="edit-owned" disabled>Apply runtime cost to Fern</button></div>
<p>Each large number below is the individual's owned current cost. Its initialization label describes construction; a later runtime edit is shown separately.</p>
<div class="lab-state" data-output="individuals"></div>
<p class="lab-explanation" data-output="explanation" role="status" aria-live="polite" aria-atomic="true">Enable JavaScript to change the draft and reconstruct these individuals. The table above supplies the same sequence without interaction. Initially Fern (ID 1) uses None and owns 1; ID 7 uses Some(1) and owns 1; ID 8 uses Some(0) and owns 0.</p>
</section>

### Checkpoint: resolve the input once

The exact reference from
[session 06](../../../docs/tutorial/path/06-owned-costs.md) defines a small
component and a resolver. Read the returned struct as an owned result. Then
watch the test change its local template from one to nine: the already
constructed values remain unchanged.

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 06
```

<!-- moss-example: session-06 -->
```rust
use bevy_ecs::prelude::Component;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimalEnergyCosts {
    pub maintenance_units_per_tick: u32,
}

fn resolve_costs(
    species_default: AnimalEnergyCosts,
    maintenance_override: Option<u32>,
) -> AnimalEnergyCosts {
    AnimalEnergyCosts {
        maintenance_units_per_tick: maintenance_override
            .unwrap_or(species_default.maintenance_units_per_tick),
    }
}

#[test]
fn session_06_an_override_is_resolved_at_creation() {
    let mut hare_default = AnimalEnergyCosts {
        maintenance_units_per_tick: 1,
    };
    let fern = resolve_costs(hare_default, None);
    let explicit_one = resolve_costs(hare_default, Some(1));
    let other_hare = resolve_costs(hare_default, Some(2));
    let free_upkeep = resolve_costs(hare_default, Some(0));
    assert_eq!(fern.maintenance_units_per_tick, 1);
    assert_eq!(explicit_one, fern); // Equal results can have different authored inputs.
    assert_eq!(other_hare.maintenance_units_per_tick, 2);
    assert_eq!(free_upkeep.maintenance_units_per_tick, 0);

    hare_default.maintenance_units_per_tick = 9;
    assert_eq!(fern.maintenance_units_per_tick, 1);
    assert_eq!(other_hare.maintenance_units_per_tick, 2);
    assert_eq!(
        resolve_costs(hare_default, None).maintenance_units_per_tick,
        9
    );
    assert_eq!(
        resolve_costs(hare_default, Some(1)).maintenance_units_per_tick,
        1
    );
}
```

`Copy` makes passing this small default value an independent value copy. The
resolved `fern` contains its own number, not a reference into `hare_default`.
The template mutation therefore affects a later construction, not an existing
result. The example demonstrates ownership; it does not run Moss's Reset or
introduce a browser control for changing defaults.

`#[derive(Component)]` makes the type usable as a Bevy component. It does not
attach a value to an entity. That is another step, performed by construction
code. A component can be correctly declared and correctly initialized while
still having no effect on the world because no animal carries it or no system
reads it.

The helper receives an already selected species default. It has no species
argument and therefore cannot choose the right template itself. Future spawn
wiring must pass a fox's configured rate for a fox and a hare's rate for a
hare. An initializer test protects resolution; construction tests protect
the connection from species to the initializer and then to the entity.

### Knowing the value is not knowing its origin

Suppose the inspector reads Fern's resolved rate and sees one. Can it say
“inherited from the species default”? Not from that number alone. The explicit
`Some(1)` case is indistinguishable after resolution.

This is a provenance question: which authored instruction produced the value?
Maintenance needs only the resolved rate. An inspector explaining its origin
needs the retained instruction or a recorded origin associated with that
animal. If that information was not recorded, the correct description is
unknown. It should not guess from equality with the current template.

The distinction lets us keep runtime data small without losing the authoring
model. Scenario inputs can preserve optional overrides while a frequently read
component holds a plain number. These representations serve different jobs;
they do not have to be the same struct. Adding origin display is separate
plumbing, not a hidden field required to subtract upkeep.

## Make the query read the new owner

Now give Fern an owned cost of one and the other hare an owned cost of two.
Nothing changes until maintenance reads those values. Continuing to look up
`Species::Hare` in the shared resource would charge both animals the same rate,
regardless of the components we attached.

The proposed query is
`Query<(&AnimalEnergyCosts, &mut Energy), With<Creature>>`. Read it as three
requirements on one entity: it must carry costs, energy and `Creature`.
Each matching row supplies read access to costs and write access to energy.
`With<Creature>` checks membership without returning a third value.

The arithmetic remains familiar. Starting from 60, a cost of one leaves 59;
a cost of two leaves 58. Three passes give 57 and 54. The query changes the
source of the rate, while `saturating_sub` still spends up to the remaining
reserve and stops at zero. It still does not define a death rule.

An entity missing required query data is skipped. That behavior is useful
when absence means “this capability does not apply,” but it can also conceal
a construction mistake. Every animal in this model owes maintenance, so
forgetting to attach costs must not grant it free upkeep.

### Checkpoint: both the right values and the right members

The exact [session 07](../../../docs/tutorial/path/07-querying-costs.md)
reference constructs small Bevy worlds and installs only its local maintenance
system. Its local component repeats the preceding concept so the reference can
run independently. The edition's `energy.rs` does not yet define
`AnimalEnergyCosts`. When the owned-cost lesson becomes active, prepare that
component there once and make live systems use it; do not copy this reference's
local definition into a second competing type.

The first test uses valid animals and checks attachment as well as reserves.
The second deliberately constructs three different component combinations.
Use that second test to distinguish a query doing its job from spawn code
failing to meet the world's construction rule.

**Reading excerpt — `spend_energy` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
fn spend_energy(mut animals: Query<(&AnimalEnergyCosts, &mut Energy), With<Creature>>) {
    for (costs, mut energy) in &mut animals {
        energy.reserve = energy
            .reserve
            .saturating_sub(costs.maintenance_units_per_tick);
    }
}
```

**Reference check — expected: two named tests.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 07
```

<details class="worked-reference">
<summary>Complete owned-cost query answer and tests (111 lines)</summary>

<!-- moss-example: session-07 -->
```rust
use bevy_ecs::{prelude::*, schedule::ExecutorKind};
use moss_sim::{Creature, Energy, Species};

#[derive(Component)]
struct AnimalEnergyCosts {
    maintenance_units_per_tick: u32,
}

fn spend_energy(mut animals: Query<(&AnimalEnergyCosts, &mut Energy), With<Creature>>) {
    for (costs, mut energy) in &mut animals {
        energy.reserve = energy
            .reserve
            .saturating_sub(costs.maintenance_units_per_tick);
    }
}

#[test]
fn session_07_same_species_can_have_different_costs() {
    let mut world = World::new();
    let first = world
        .spawn((
            Creature { name: "Fern" },
            Species::Hare,
            Energy {
                reserve: 60,
                capacity: 100,
            },
            AnimalEnergyCosts {
                maintenance_units_per_tick: 1,
            },
        ))
        .id();
    let second = world
        .spawn((
            Creature {
                name: "another hare",
            },
            Species::Hare,
            Energy {
                reserve: 60,
                capacity: 100,
            },
            AnimalEnergyCosts {
                maintenance_units_per_tick: 2,
            },
        ))
        .id();
    let grass = world.spawn(Species::Grass).id();

    let animal_count = world
        .query_filtered::<Entity, With<Creature>>()
        .iter(&world)
        .count();
    let cost_count = world
        .query_filtered::<&AnimalEnergyCosts, With<Creature>>()
        .iter(&world)
        .count();
    assert_eq!(animal_count, 2);
    assert_eq!(cost_count, 2);
    assert!(world.get::<AnimalEnergyCosts>(grass).is_none());

    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(spend_energy);
    for _ in 0..3 {
        schedule.run(&mut world);
    }
    assert_eq!(world.get::<Energy>(first).unwrap().reserve, 57);
    assert_eq!(world.get::<Energy>(second).unwrap().reserve, 54);

    world.get_mut::<Energy>(second).unwrap().reserve = 1;
    schedule.run(&mut world);
    assert_eq!(world.get::<Energy>(second).unwrap().reserve, 0);
    assert!(world.get::<Creature>(second).is_some());
}

#[test]
fn session_07_only_matching_entities_pay() {
    let mut world = World::new();
    let energy = Energy {
        reserve: 60,
        capacity: 100,
    };
    let eligible = world
        .spawn((
            Creature { name: "eligible" },
            energy,
            AnimalEnergyCosts {
                maintenance_units_per_tick: 1,
            },
        ))
        .id();
    let missing_cost = world.spawn((Creature { name: "no cost" }, energy)).id();
    let not_an_animal = world
        .spawn((
            energy,
            AnimalEnergyCosts {
                maintenance_units_per_tick: 1,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(spend_energy);
    schedule.run(&mut world);

    assert_eq!(world.get::<Energy>(eligible).unwrap().reserve, 59);
    assert_eq!(world.get::<Energy>(missing_cost).unwrap().reserve, 60);
    assert_eq!(world.get::<Energy>(not_an_animal).unwrap().reserve, 60);
}
```

</details>

The first test runs three passes through the local schedule. It inspects each
specific entity, so one missed animal cannot hide inside a correct total.
Afterward it gives the more expensive hare only one remaining unit. The next
pass clamps its reserve to zero, and the final assertion confirms the creature
still exists. Changing the rate's owner has not changed the meaning of zero.

The membership test deserves its three actors:

| Component combination | Observation after one maintenance pass |
| --- | --- |
| `Creature`, costs and energy | The positive control reaches 59, showing the system ran. |
| `Creature` and energy, without costs | It stays at 60 because required query data is missing. This synthetic animal is malformed for the planned production model. |
| Costs and energy, without `Creature` | It stays at 60 because the animal filter excludes it. |

The last control has both data components. If the filter disappeared, it would
start paying upkeep and the test would fail. A bare grass entity would not
serve the same purpose: lacking energy and costs already excludes it, even
without `With<Creature>`. The choice of control determines which mistake a
green test can rule out.

In the loop's `(costs, mut energy)`, the immutable cost access keeps the
baseline read-only. Bevy provides writable energy through its `Mut<Energy>`
wrapper; the mutable binding lets the field assignment reach the component.
This is Fern's existing reserve, not a local copy discarded after the loop.
The versioned [Bevy ECS 0.18.1 query documentation](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html)
explains the required data and filter access used here.

When this becomes a live checkpoint, a no-food scenario isolates upkeep while
leaving the real schedule installed. If both hares finish at 57, inspect whether
maintenance still reads the species resource. If one stays at 60, inspect
attachment and query membership before changing the subtraction.

[Continue the two-hare experiment](#give-the-difference-a-fair-journey) to see
those costs combine with travel, or stay here to examine the component boundary.

## How many components should these attributes become?

The owned-cost example answers who owns a number. It does not require every
attribute to occupy its own component, and it does not require all attributes
to share one component. Group fields according to the rules that read and
change them, their lifetime, and whether their presence has a useful meaning.

`Energy { reserve, capacity }` already groups two fields. Meals need them
together to calculate room, and both describe the same reserve. Splitting
them merely because one changes more often would add a boundary without yet
solving a problem. Maintenance costs describe the rate at which that reserve
is spent, so keeping them separately owned makes the baseline/state
distinction visible.

Now imagine an additional movement-pace rule. The existing
[attribute comparison](../../../docs/tutorial/context/attributes-and-defaults.md#where-would-one-more-attribute-live)
uses a positive `step_interval_ticks`: how many simulation ticks separate
steps. This is a future design example, not the current one-cell-per-tick rule.
It gives us a specific reason to compare two layouts.

If upkeep and pace live inside one hypothetical `AnimalProfile` component,
maintenance reads that component while a pace adjustment writes that component.
Ordinary query access is declared for the component, not just whichever field
the body happens to touch. When both systems can reach the same animals, those
declared accesses overlap. Putting a nested `Locomotion` struct inside that
profile does not make it a separately attached component.

Alternatively, upkeep can read `AnimalEnergyCosts` and a pace adjustment can
write a separately attached `Locomotion`. Those particular accesses no longer
overlap. This also permits an entity without locomotion to owe upkeep while
being excluded from a locomotion query. The absence has a model meaning;
absence of required upkeep costs would still be a construction defect.

Both layouts can store individual values. Splitting fields changes an access
or presence boundary, not their ownership. Moss's schedule remains
single-threaded, so this comparison is not evidence of a speedup. Systems
that both mutate `Energy` still share that access regardless of how their
other fields are grouped.

A “bag of attributes” can describe several different arrangements: a concrete
struct with named fields, several typed components on an entity, or a dynamic
map of keys to values. The present rules need the first two. A maintenance
query can state its required types and iterate costs beside energy without
asking a generic lookup service to interpret an attribute name. A future
data-driven authoring format could still resolve its inputs into these concrete
runtime values. The editor's representation need not dictate the tick loop's
representation.

### A speed, a level and a temporary effect

Here is a bounded design example to make the distinction useful beyond upkeep.
It proposes no new Moss API or selected movement policy. Suppose an individual
owns a baseline interval of **4 ticks per step**. For this illustration, a
particular level adjustment halves that interval, and a temporary slowing
effect doubles the adjusted interval.

| Kind of value | Value in this example and its job |
| --- | --- |
| Baseline | 4 ticks per step, owned by the individual. It remains available when an effect ends. |
| Effective value | `4 ÷ 2 × 2 = 4` ticks per step while the level adjustment and slowing effect apply. This is what the proposed pace rule would read. |
| Current state | 1 tick elapsed since the last accepted step. It describes progress, not the underlying capability. |

Remove the slowing effect and recompute: the effective interval becomes
`4 ÷ 2 = 2`, while the baseline stays four. A smaller interval permits more
frequent steps under such a model. Applying the formula again next tick still
produces two; repeatedly dividing the stored baseline would instead produce
four, two, one and eventually an unintended zero.

The current elapsed state has its own question: what happens when the required
interval changes midway through waiting? Keeping or rescaling progress would
be different policies. Naming state separately exposes that choice instead
of hiding it inside a speed getter. This numerical example avoids rounding;
a general rule would also need positive bounds and explicit rounding when
division does not produce a whole tick.

The same separation helps temporary maintenance effects. Combine the baseline
with applicable conditions at a defined point; do not overwrite the baseline
every tick and try to undo the accumulated changes later. If an effective
value is eventually cached, removing an effect or changing a global condition
must refresh it before a consumer runs. A cache introduces a dependency rule;
it does not make that rule disappear.

## What this layout does—and does not—tell us about scale

The concrete query gives us an inspectable path from owned costs to reserves.
It does not prove that copying a number is always faster than looking up a
shared definition. The relevant comparison depends on the population,
component layout, systems and update frequency we actually run.

Small, frequently used individual values are reasonable candidates for owned
components. A large, rarely read species description can stay shared behind a
key or handle. Those choices can coexist. We do not need to copy every species
definition onto every animal in order to give upkeep an individual owner.

When scale becomes a question, measure simulation tick time separately from
rendering, along with memory, allocations and the expensive systems. Compare
the same entities, accepted inputs, executed ticks and outcomes before and
after a representation change. Different numeric values within the same
component types do not themselves create more archetypes; attaching different
sets of components does. Neither statement is a benchmark of Moss's eventual
world.

The [applied attribute research](../../../docs/research/attributes-and-energy.md)
and [property/default research](../../../docs/research/property-bags-and-defaults.md)
are optional comparisons when an actual requirement makes another design
interesting. Today's experiment has a smaller question: can two hares pay
different upkeep for an explicit reason we can inspect?

## Give the difference a fair journey

The no-food experiment isolates maintenance. Next let both animals perform
the same movement, so we can see the individual rate combine with an existing
action. Both start at `(10, 10)`, reserve 60, seeking Meadow at `(16, 13)`.
Fern pays upkeep one; the other hare pays two. Travel remains the shared
**2 energy units per accepted cell**.

The initial Seeking activity matters. Under the earlier threshold rule, an
idle hare at reserve 60 would stay idle, while one already Seeking continues.
Forgetting that input could turn the intended two-walker comparison into
standing still, without any error in the cost component.

After three x steps, both should reach `(13, 10)`. Neither has reached food,
so Meadow's biomass should remain 80. These are the expected values for the
future controlled scenario:

| Completed ticks | Shared position; Fern / other hare reserves |
| --- | --- |
| 0 | `(10, 10)`; 60 / 60 |
| 1 | `(11, 10)`; 57 / 56 |
| 2 | `(12, 10)`; 54 / 52 |
| 3 | `(13, 10)`; 51 / 48 |

Each animal spends six on travel. Fern also spends three on upkeep; the
other hare spends six. The three-unit gap now has one controlled cause: a
one-unit difference on each of three maintenance ticks.

The observations support one another. With the specified start and at most
one cardinal step on each of three ticks, `(13, 10)` establishes three cells
of accepted travel. Unchanged biomass rules out a meal in this fixture.
Equal endpoints alone would not establish equal distance in a journey long
enough to include a detour, so keep the input conditions in the explanation.

### Checkpoint: account for the same accepted distance

This exact [session 08](../../../docs/tutorial/path/08-a-fair-comparison.md)
reference starts with an already supplied movement trace. Its `Account` is
local example data, not another component to install. Read `[1, 1, 1]` as
one cell actually accepted on each of three ticks. The function does not
choose or perform those steps.

**Reading excerpt — `account_trace` from the complete isolated reference below.**
The imports, local types and test fixtures remain in the expandable answer.

```rust
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
```

**Reference check — expected: one named test.** Run from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 08
```

<details class="worked-reference">
<summary>Complete journey accounting answer and test (65 lines)</summary>

<!-- moss-example: session-08 -->
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

</details>

Maintenance records the amount actually available to deduct. Travel is
calculated from accepted distance, then checked for affordability. The
assertion rejects an impossible supplied trace rather than quietly granting
movement on credit. This is an accounting model of observations, not a
replacement for the movement helper's acceptance rule.

The equality back to 60 reconciles each starting reserve with its remaining
reserve and both expenditures. It would not, by itself, prove equal journeys;
the separate travel totals and the supplied traces provide that context.
The live experiment must produce its positions through the real movement
system and check the unchanged patch as well as reserves.

**A nearby case:** keep three executed ticks but supply `[1, 0, 1]` for both
hares. What remains if one tick has no accepted travel?

**Answer:** both spend four on travel. Fern spends three on upkeep and finishes
at 53; the other spends six and finishes at 50. The gap is still three because
the upkeep difference still applied on three ticks. The zero-distance tick
removes a travel charge, not a tick of maintenance. These values follow the
reference's accounting assumptions; they do not explain why movement was
absent on that tick.

## Run the reference, then check the live connection

The four complete answers above are unchanged copies of their short-session
references. Each has its exact command beside it. Expect one named reference
test per command except session 07, which has two. The runner extracts the
existing guide's complete answer into an isolated workspace. It does not check whether a live learner edit is finished
or install these proposed systems. The
[reference evidence](../../../docs/tutorial/path/verification.md#what-each-reference-establishes)
records what those examples establish and where their scope ends.

For actual integration, the agent prepares one selected checkpoint at a time:
summary calculation, initialization, maintenance access, or the controlled
comparison. Its named live symbol, test and stopping point belong in `NOW.md`.
Spawn and Reset checks must establish that every animal receives the intended
owned cost and that authored overrides survive reconstruction. The inspector
must display the rate maintenance actually uses, not merely its old template.

After the comparison is installed and reviewed, its browser check is short:
Reset the comparison scene, Step three times and inspect both hares separately
by ID. Their markers will overlap at `(13, 10)`, while reserves should be 51
and 48. Check Meadow at 80 biomass before judging the account. These remain
expected post-integration observations in this edition.

We can now explain an individual number all the way through: an authored
instruction chose it, construction gave it an owner, a matching query read it,
and a controlled run exposed its consequence. The two hares do not need a
different species or a universal attribute system to live differently. Their
next constraint is shared: every successful meal still spends down Meadow's
finite supply. Giving that supply a source will let us ask what the world
can support, as well as what each animal spends.
