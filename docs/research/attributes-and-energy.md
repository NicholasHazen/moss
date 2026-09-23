# Practical attribute and energy design

[Research index](README.md) · [Attributes context](../tutorial/context/attributes-and-defaults.md) · [Future cost lessons](../tutorial/path/06-owned-costs.md)

**Reviewed:** September 21, 2026. This is research supporting a proposed design,
not an implemented feature or a Moss performance result. Two bounded research
helpers investigated ECS storage and applied attribute systems; the lead
compared their findings with simulation profiling reports.

For the broader question of large property sets outside games, continue with
[property bags and defaults](property-bags-and-defaults.md). It compares UI,
scene-description and configuration systems, including sparse authored values,
provenance and typed consumer views.

Fern and another hare should be able to pay different maintenance costs without
becoming different species. Later, cold weather might temporarily increase both
costs. These requirements raise three separate questions: where defaults live,
who owns an individual's baseline, and how temporary changes affect it.

The evidence supports keeping those responsibilities separate. My recommendation
remains shared species templates, small owned components for individual values,
and explicit temporary-effect rules when we need them. There is no evidence here
that this is universally the fastest layout.

## On this page

- [What other projects actually do](#what-other-projects-actually-do)
- [The choice that changes behavior](#the-choice-that-changes-behavior)
- [How this fits Moss](#how-this-fits-moss)
- [The next proof](#the-next-proof)
- [Evidence limits and performance](#evidence-limits-and-performance)

## What other projects actually do

### Flecs makes copying and sharing separate policies

Flecs v4.1 copies prefab components into instances by default. Its optional
inheritance policy instead keeps a shared value that queries can resolve through
the base prefab via `IsA`. An instance can acquire an owned override. Both are supported
contracts, rather than one being an ECS mistake. This is engine documentation,
not a comparative benchmark. [Flecs component traits](https://www.flecs.dev/flecs/ComponentTraits.html).

For Moss, the useful lesson is to choose the contract explicitly. Flecs supplies
the inheritance machinery; ordinary Bevy components do not acquire that behavior
automatically.

### Unity separates instance data from shared definitions

Unity Entities 1.3.15 clones component values when instantiating an entity. It
also provides immutable blob assets for structured data referenced by components.
Copying a small component can therefore coexist with sharing a large definition;
copying a handle does not duplicate its underlying asset.
[Instantiation](https://docs.unity3d.com/Packages/com.unity.entities@1.3/api/Unity.Entities.EntityManager.Instantiate.html),
[blob assets](https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/blob-assets-concept.html).

Its separate *shared component* mechanism groups entities by equal values.
Unity warns that many distinct values fragment storage into mostly empty chunks,
and changing those values moves entities between chunks. This warning is specific
to Unity's mechanism. It does not mean that different numeric values in an
ordinary Bevy component create different archetypes.
[Shared-component guidance](https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/components-shared-optimize.html).

### Veloren provides an inspectable Rust game example

At commit `2339dcd45104cedb4d7461a96c648dd9d7658c5f` (September 20, 2026), Veloren's
energy component owns current energy, a base maximum and a modified maximum.
Initialization derives values from the creature's body. Methods enforce bounds,
including reducing current energy when its maximum falls.
[Pinned energy component](https://github.com/veloren/veloren/blob/2339dcd45104cedb4d7461a96c648dd9d7658c5f/common/src/comp/energy.rs#L17-L24).

Its buff system resets temporary stat modifiers and reapplies active effects.
Stacking and processing order have explicit rules. Energy-change events are
separate from those temporary modifiers. This gives us a concrete alternative
to repeatedly mutating a baseline and attempting to reverse those mutations
when an effect expires.
[Pinned buff system](https://github.com/veloren/veloren/blob/2339dcd45104cedb4d7461a96c648dd9d7658c5f/common/systems/src/buff.rs#L453-L500).

Veloren uses Rust and Specs ECS, not Bevy. Its implementation demonstrates the
separation in a real game; it does not establish the right component sizes,
effect rules or performance for Moss.

### Unreal formalizes baseline versus effective value

Unreal Engine 5.6's Gameplay Ability System distinguishes an attribute's base
value from its current value under active effects. A permanent improvement can
change the base while a temporary penalty remains active. Attribute Sets group
typed attributes, and bounds require explicit handling.
[Gameplay attributes](https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-attributes-and-attribute-sets-for-the-gameplay-ability-system-in-unreal-engine?application_version=5.6).

We can use that distinction in a plain Rust function. Moss has no current need
for the surrounding ability framework. Also, an effective maintenance rate and
the energy remaining in a reserve are different quantities; naming both
“current energy” would hide the distinction we are trying to preserve.

### Factorio shows why layout is only part of scalability

In its August 18, 2017 engineering report, Factorio describes moving infrequently
used data out of entities. The invasive change saved some space but produced no
measurable performance improvement. Subsequent optimizations needed workload
measurements. [Friday Facts #204](https://www.factorio.com/blog/post/fff-204).

A July 26, 2024 report describes avoiding unnecessary updates for idle roboports.
In one playtest save, time spent on that subsystem fell from about 1 ms to
0.025 ms per tick. That is a particular subsystem and save, not a general game
speedup. [Friday Facts #421](https://www.factorio.com/blog/post/fff-421).

For Moss, this suggests measuring which work dominates before redesigning
attribute access. It does not justify skipping maintenance, tying biology to
visibility, or changing executed-tick outcomes. Factorio is a simulation
engineering reference here, not evidence about Bevy's storage implementation.

## The choice that changes behavior

Consider an animal created when its species default is 1. What should happen if
the default later becomes 3?

| Representation | Meaning for an existing animal |
| --- | --- |
| Shared definition plus optional override | An animal without an override reads 3. An absolute override of 2 still reads 2. |
| Owned baseline initialized at spawn | The animal keeps its original value until an explicit operation changes it. |
| Cached result of a shared lookup | The animal should read 3 after invalidation and recomputation; failing to refresh is a bug. |

The first approach is reasonable when live changes should propagate and
exceptions are sparse. The second fits individual variation and makes ownership
direct. The third is an optimization with an invalidation obligation, not another
name for ownership.

Our proposed `AnimalEnergyCosts` uses the second contract. Species defaults stay
fixed within a run. Authored overrides initialize individual values, including
zero. Reset rebuilds the scenario and reapplies those overrides. A later live
retuning feature must specify whether it changes defaults, existing individuals,
or both.

## How this fits Moss

| Data | Proposed home |
| --- | --- |
| Species defaults and substantial shared definitions | A catalog or resource, addressed through the existing species identity or a future stable key. |
| Small individual baseline costs | `AnimalEnergyCosts` on each animal. Initially, only maintenance units per tick. |
| Energy remaining and capacity | The existing `Energy` component. |
| Temporary conditions | Concrete simulation inputs; effective values computed where needed. Add stored derived values only with a reason. |

Suppose Fern's baseline is 1 and the other hare's override is 2. An illustrative
cold multiplier of 2 would produce effective costs of 2 and 4. When cold ends,
the costs return to 1 and 2. Neither baseline changed. This is an example of
ownership, not a selected weather rule or a feature to implement now.

As attributes grow, group them by the systems that read them and the rules that
change them. Maintenance should read costs and energy; it should not need a
giant record of perception, reproduction and appearance. Large shared data can
stay shared while small values needed every tick live on individuals. This is
our design inference from the examples, not a mandate to copy every attribute.

For Bevy 0.18.1, typed queries declare which component data a system accesses.
Its documentation describes query iteration costs and warns that optional
components can broaden the matching set. This supports focused queries; it
does not prove a local component is always faster than a small species lookup.
[Pinned query documentation](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html#performance).

Action costs need a separate distinction: a cost coefficient belongs with the
animal's attributes, while distance actually traveled belongs to the completed
movement calculation. Charge from the accepted movement outcome. Merely adding
a distance field or selecting an activity does not establish that travel occurred.
This remains the existing later movement lesson.

## The next proof

The research does not expand today's [movement task](../tutorial/today-v2.md).
When individual variation becomes active, [session 06](../tutorial/path/06-owned-costs.md)
starts with resolving a species value and an authored override; the agent prepares
the component and test. [Session 07](../tutorial/path/07-querying-costs.md) then
changes which data maintenance reads. [Chapter 1D](../tutorial/01d-energy-profiles.md)
is their integration companion. This supersedes the earlier declaration-only
assignment and fixed test count.

After the agent prepares spawn wiring, the decisive regression will use two
hares starting at 60, with costs 1 and 2. Three ticks must leave 57 and 54 while
the species default stays unchanged. A spawn invariant should ensure every
animal carries its costs; otherwise a query requiring that component can quietly
exclude an incomplete animal. Explicit-zero and reset cases complete the
ownership checks, followed by inspection in the browser.

When the first temporary effect is actually selected, add a small test proving
that applying and removing it preserves the individual's baseline. Multiple
effects then require explicit composition, rounding and expiration rules. No
generic modifier engine is needed to begin.

## Evidence limits and performance

The engine references document supported behavior. Veloren supplies inspectable
implementation. Factorio supplies measured experience from different workloads.
None compares candidate attribute layouts in Moss, and none establishes a
browser population limit. The cited engine versions are reference points, not
proposals to change Moss's dependencies.

Before a performance-driven redesign, compare identical populations and tick
outcomes in the same release browser build. Measure simulation time separately
from rendering, plus memory and time in individual systems. Only compare shared
lookup, owned values or derived caches if attribute access is a meaningful
cost. A cache must also include its refresh work in that comparison.

No simulation code, tests, dependencies or browser behavior changed during this
research. No benchmark or new native/WASM/browser run was performed.

[Research index](README.md) · [Next edit](../tutorial/01d-energy-profiles.md)
