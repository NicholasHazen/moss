# Attributes, defaults and individual variation

[Context shelf](README.md) · [Later individual-cost chapter](../01d-energy-profiles.md)

**Proposed direction:** the live maintenance loop still reads shared species
rates. The individual component below is a later paired variation experiment,
not a running feature. Today's active edit is [movement](../today-v2.md).

Fern and a second hare start with 60 energy. A species template supplies a
maintenance rate of 1, but the second animal has an authored override of 2.
Initialization resolves those inputs into a cost component on each animal.
In [session 07's no-food fixture](../path/07-querying-costs.md), neither animal
can travel or eat, so three ticks should leave 57 and 54. Maintenance needs the
resolved costs, without interpreting where either value came from. When both
hares instead travel three cells, [session 08's comparison](../path/08-a-fair-comparison.md)
expects 51 and 48: each also paid the same 6 units for movement. The fixture
determines which observation actually isolates the individual cost.

## Ownership before optimization

| Data | Responsibility |
| --- | --- |
| Shared species definitions | Authored defaults and shared descriptive data. |
| Scenario or future inheritance inputs | Select an individual's starting values. |
| Individual components | Own the animal's numeric baseline and current state. |
| Simulation systems | Apply explicit rules using those values. |

This separates a template from an instance. The species template is not a live
parent whose fields every animal must inherit whenever they are read. An
individual's baseline can diverge. `Species` still identifies membership and
can locate shared information, but it need not mediate every numeric read.

The selected initial policy is an absolute authored override at spawn, falling
back to the species default only when the override is absent. A zero value is
a real override. Rates keep their named units; an override does not change what
a tick represents. Biological inheritance and mutation later become explicit
ways to initialize an offspring's individual values, rather than copying a
pointer to a parent's mutable state. No inheritance rule is selected today.

Defaults remain fixed during a run. Reset constructs the authored population
again, so recorded scenario overrides recur and temporary instance edits do not.
If live retuning is added, its scope must be explicit: updating a template alone
will not silently overwrite existing individuals.

Consider an override that happens to equal the default. These are two different
authoring instructions, even though the first constructed values agree:

| Authored input | New component value under each scenario default |
| --- | --- |
| `None`: use the species value | Default 1 produces 1; a later scenario with default 9 produces 9. |
| `Some(1)`: explicitly use 1 | Default 1 produces 1; a later scenario with default 9 still produces 1. |

This is a construction comparison, not live propagation. Existing animals keep
their owned baseline. Under the planned Reset policy, the unchanged default-1
scenario would reconstruct 1 for both animals; the table's changed default belongs
to a newly configured run. Keep the original `Option` in authored scenario data so
reconstruction can repeat the instruction. An inspector reading only a component
with value 1 cannot infer “default” or “override”: that origin must have been
recorded, or be shown as unknown. Maintenance itself needs only the resolved
number. [Session 06's reference](../path/06-owned-costs.md#trace-a-resolved-value)
checks the equal-result case without adding a provenance component or a new task.

## Many attributes do not require many scattered lookups

An ECS query names the component types a system needs. Bevy prepares access to
matching entities; a maintenance loop can iterate costs and energy together.
The default component storage is designed for iteration. This motivates the
design; it does not establish that copying profiles beats every shared lookup
on Moss's eventual workload. [Bevy components](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/component/trait.Component.html#choosing-a-storage-type),
[query performance](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html#performance).

Group fields according to how they are used and updated. Energy reserve changes
frequently; its baseline costs usually do not. Unrelated perception attributes
should not be dragged into the maintenance query. These boundaries can evolve
as actual systems reveal which data they need together.

Different values of the same component do not produce new archetypes. A set of
components can serve a slow hare, a fast hare and a fox. Avoid expressing every
numeric variation as a unique marker component; changing a numeric field is
enough. Adding or removing `FoodTarget`, however, carries a useful meaning:
it gives or withdraws an instruction, as [session 04](../path/04-when-to-seek.md)
shows. That changes the component set, but clarity earns its place here. Measure
the cost of structural changes before replacing that representation.

Copy small, frequently used individual values. Share large or infrequently used
definitions through stable keys or handles. A species catalog can eventually
load validated definitions from files without dictating the runtime layout.
Do not store borrowed references into a movable ECS resource on each entity.

## Where would one more attribute live?

Imagine a later rule giving Fern a movement pace, represented by a positive
`step_interval_ticks`: how many simulation ticks separate steps. This is a
hypothetical attribute, not today's one-step movement rule or a new assignment.
It gives us a concrete way to compare two possible layouts.

We could put upkeep and pace in one `AnimalProfile` component. Upkeep would
request `(&AnimalProfile, &mut Energy)`, then read the maintenance field. A
later pace adjustment would request `&mut AnimalProfile`, even if it changed
only `step_interval_ticks`. Ordinary Bevy queries declare access to components,
not individual struct fields. These two accesses overlap when they can reach
the same animals. [Bevy query access](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html#performance).

Alternatively, upkeep can read the proposed `AnimalEnergyCosts`, while pace
adjustment mutably borrows a separate hypothetical `Locomotion` component.
Those particular accesses no longer overlap. Both layouts still give each
animal owned values; splitting them changes the access boundary, not who owns
the numbers.

![A hypothetical grouped AnimalProfile puts maintenance and movement pace behind one component access boundary. Separate AnimalEnergyCosts and Locomotion let upkeep read costs while pace adjustment writes locomotion. Energy remains a separate reserve component in both layouts.](../visuals/component-boundaries.svg)

Moss's tick remains single-threaded. This comparison gives us clearer declared
responsibilities, not a demonstrated speedup or permission to change execution
order. Two systems that both mutate `Energy` still overlap even if their other
attributes are split. Merely nesting a `Locomotion` struct inside a profile does
not attach it as a separate ECS component; separate access requires separate
attachment to the entity.

Presence is another reason to split. A hypothetical animal with no locomotion
capability may still owe upkeep. Without `Locomotion`, it can match the maintenance
query while staying out of a query that requires locomotion. Missing `AnimalEnergyCosts`, however,
would violate our construction rule and accidentally skip upkeep. The same
absence has different meanings because those components have different jobs.
An optional field inside a profile is also possible, but its consumers must
inspect that field after fetching the profile. The representation should make
the intended capability or invariant clear.

Keep the opposite example in view: today's `Energy { reserve, capacity }` groups
two related values. Meals need both to preserve a bounded reserve, and they share
the same lifetime. There is no need to split them merely because one changes more
often. Start with fields that belong together; split when independent access or
presence explains a real rule. The next individual-cost lesson needs only upkeep,
so it does not predeclare this hypothetical locomotion component.

## Temporary effects and derived values

The individual baseline should survive a temporary effect ending. If cold
weather increases maintenance, the rule can combine baseline and current
conditions at a defined point in the tick. Define whether a particular effect
adds or multiplies, its order relative to other effects, rounding and clamping.
Never compound a multiplier into the baseline every tick and hope to undo it
later.

If resolving many effects becomes expensive, a derived effective-cost component
may be useful. That introduces a new obligation: refresh it when any dependency
changes, including global environment inputs and removed effects, before any
consumer runs. Bevy change detection alone does not define that dependency
policy. Start with an explicit, tested rule before introducing such a cache.

An attribute's meaning determines its update rules. Spendable currency would be
owned mutable state; level-dependent speed could be derived from an individual
baseline; damage dealt would be an outcome involving an attacker and target.
These can share conventions for defaults and inspection without sharing one
calculation or component. [The parked RPG discussion](../../design/parking-lot.md#ecs-attributes-with-possible-rpg-progression)
is an optional design example, not additional work for this chapter.

## What we would measure

The relevant workload is the population we actually intend to simulate in the
browser: tick time, memory, allocation, number of component layouts and the
cost of the systems doing the most work. Compare the same entities, ticks and
outcomes when testing a storage change. Keep render-frame timing separate.
No benchmark has established a universally fastest layout for Moss.

When we revisit individual costs, the correctness example is deliberately small: two hares, different costs,
unchanged species identity, and independent reserves. It establishes the
ownership rule before a larger population experiment tests performance.

For practical precedents, the [applied research note](../../research/attributes-and-energy.md)
compares Flecs, Unity, Veloren, Unreal and Factorio. Shared inheritance and owned
baselines are both valid contracts; choose the intended behavior before comparing
their performance. The [property-system research](../../research/property-bags-and-defaults.md)
looks at WPF, CSS, OpenUSD and configuration libraries. Authored overrides, resolved
values and runtime storage can use different representations; retain authored
inputs even when a runtime component contains only their result. These are
optional sources, not a reading assignment before the next edit.

[Context shelf](README.md) · [Chapter 1D](../01d-energy-profiles.md)
