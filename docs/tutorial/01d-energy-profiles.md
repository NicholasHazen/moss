# 1D. Give each animal its own energy costs

[Guide home](README.md) · Current route: [today](today.md) · Related: [populations](02-populations.md)

**Future: revisit individual variation after movement, a finite meal and food
choice make the difference visible.** This chapter is not a prerequisite for
[today's route](today.md). The proposed component and its runtime use are not
implemented. Before activation, the agent reconciles the examples and acceptance
checks with the current code and test baseline. [Verification](authoring/verification.md)
separates historical worked-example evidence from current readiness.

Fern and another hare can belong to the same species but have different
metabolism. Suppose the species default costs 1 unit per tick, while the second
hare is configured to spend 2. In [session 07's no-food fixture](path/07-querying-costs.md),
three ticks from 60 leave 57 and 54: neither animal can travel or eat, so the
comparison isolates maintenance. Species alone cannot express that difference.

We will keep species defaults as shared configuration, then use those defaults
and any authored individual overrides to initialize each animal's costs. The
animal owns the result. Maintenance will read that component directly, alongside
its energy reserve. This gives frequently used numeric attributes a home on the
entity without making every system repeat a default-and-override lookup.

This direction draws on [practical ECS and game examples](../research/attributes-and-energy.md).
That optional reading explains the alternatives; it is not a prerequisite for
the small edit below or proof that this layout is universally fastest.

The short-session route divides this work between [resolving an individual's
cost](path/06-owned-costs.md) and [reading it in the maintenance query](path/07-querying-costs.md).
Those pages own the paired steps and complete checked references. This chapter
keeps the component and integration context together for existing bookmarks.

**Before session 06 begins:** the agent prepares the component, exports,
and a focused resolver test. Your first edit is the body of `resolve_costs` in
`energy.rs`: use an explicit override when present, otherwise the configured
species value. That is the ownership decision we want to practice. Declaring
the component and wiring every spawn site are preparation, not prerequisites
you must complete before making that decision.

## On this page

- [Group the settings](#group-the-settings)
- [Update the test configuration](#update-the-test-configuration)
- [What scales from here](#what-scales-from-here)
- [Review and stop](#review-and-stop)

## Group the settings

**Session 06 integration reference — future.** The agent prepares
this complete declaration in [energy.rs](../../crates/moss-sim/src/energy.rs),
beside `Energy` and `SpeciesEnergyRules`. It uses the existing `Component` import.
You will receive this type as the input and output of the small resolver in
[session 06](path/06-owned-costs.md).

<!-- example: individual-energy-component -->
```rust
/// This animal's baseline costs, initialized from defaults and spawn overrides.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimalEnergyCosts {
    pub maintenance_units_per_tick: u32,
}
```

`#[derive(Component)]` makes this type eligible to live on an entity. It does
not attach it to existing animals; the spawn code must do that explicitly.
`Copy` lets the initializer copy these small values from a species template.
Once copied onto Fern, they belong to Fern. The species configuration supplies
the starting value explicitly. This type needs no separate Rust `Default`
implementation: a convenient fallback of 1 could obscure a configured Fox rate
of 2. `None` selects the supplied species value; `Some(0)` selects a real zero.

The agent preserves existing exports in [lib.rs](../../crates/moss-sim/src/lib.rs)
while exposing the component and prepared resolver to the integration test.
This is the **future complete energy re-export**, after both names exist:

<!-- example: individual-energy-export -->
```rust
pub use energy::{AnimalEnergyCosts, Energy, MovementRules, SpeciesEnergyRules, resolve_costs};
```

The declaration lives beside the other energy data; the re-export makes
`moss_sim::AnimalEnergyCosts` available to tests and the browser. This is one
type with a public path, not a second declaration. The
[module explanation](context/rust-at-point-of-use.md#where-a-declaration-lives)
connects `mod`, `pub` and `pub use`.

The agent supplies the exact focused command in session 06 when it becomes active.
It must call your live `resolve_costs`, not only the guide's printed answer.
The literal cases are species rate 1 with `None` → 1, `Some(2)` → 2, and
`Some(0)` → 0. Changing the template afterward leaves already-created costs
unchanged. The [session 06 reference](path/06-owned-costs.md) demonstrates that
ownership result with complete code.

These checks do not yet prove every animal carries the component. If a compiler
error says `AnimalEnergyCosts` is defined twice, keep one declaration and check
whether an earlier version of 1D was already added. Preserve `MovementRules`
when updating exports so the addition does not break the movement API.

## Update the test configuration

**Session 07 integration reference — future.** The agent will group
species defaults into the same small value type and initialize it on every
animal in `fixture.rs`. Grass receives no animal cost component. Any helper
that spawns an animal must preserve this invariant; a missing component would
otherwise silently exclude an animal from the new maintenance query.

The reviewed resolver supplies the component at construction. No second override
component needs to be searched by every rule. The initializer must pass the
configured species value through, rather than substituting a global constant.

The agent will prepare a regression with two hares, one at the default rate of
1 and one with an individual rate of 2. The test should first prove both cost
components exist with the expected values, then expect reserves of 57 and 54
after three installed ticks in a no-food scenario. That setup excludes travel
and meals while preserving the real schedule, so these reserves measure only
maintenance. The old species-based loop will still produce 57/57 for those
hares, giving the next paired edit a meaningful red checkpoint. These tests are
preparation requirements, not APIs or results already present.

After that preparation is reviewed, the paired change will request the animal's
costs directly. This is the **future query-type excerpt**, not an edit to make yet:

```rust
Query<(&AnimalEnergyCosts, &mut Energy), With<Creature>>
```

The subtraction stays saturating, and zero still causes no death. The loop will
read `costs.maintenance_units_per_tick`; it will no longer need `Species` or the
shared resource. The agent will reconcile the exact test and complete function
with the prepared code before making that step active.

Future acceptance includes unchanged species-default results, independent
same-species costs, an explicit zero override, and reset rebuilding the authored
costs. A temporary runtime edit to one animal disappears on reset; an override
stored in the scenario is applied again. The inspector should show the actual
component value and label its source only when that provenance is recorded.

## What scales from here

Species definitions are templates. Each animal's costs are authoritative
individual data for the current run, rather than a cache that must track every
change to a template. Species defaults remain fixed during a run under Moss's
existing rule. Retuning existing animals later would be an explicit simulation
operation with a policy for preserving individual differences.

As attributes grow, keep small typed components grouped by the systems that
read and change them. Cost settings, mutable energy reserves and unrelated
sensory data have different uses. We do not need a giant attribute struct or a
string-keyed map queried inside every tick. Individual numeric values do not
require different component types for different animals.

Frequently read individual values belong in components. Larger shared data,
such as descriptions or animation definitions, can stay behind species or asset
IDs. This is a design direction, not a benchmark result: copying more data also
uses more memory, and real population tests must guide later tuning.

Temporary weather or status effects should leave the baseline intact. Add an
explicit calculation or a derived value when the first such rule exists,
with defined ordering, rounding and bounds. There is no universal modifier or
cache invalidation framework to build in this checkpoint.

The [attributes and defaults context](context/attributes-and-defaults.md)
explains the ownership and scaling tradeoffs in more detail.

**Optional check:** both hares can share `Species::Hare` and still store different
`AnimalEnergyCosts` values. This does not create separate ECS archetypes: an
archetype describes the set of component types, not the values inside them.

## Review and stop

Use [session 06's resolver checkpoint](path/06-owned-costs.md) and then
[session 07's query checkpoint](path/07-querying-costs.md) for the edit and review
request. This companion does not activate a separate A/B sequence. The agent
keeps [NOW.md](../../NOW.md) on one step through reset, inspector and browser
acceptance. Population scaffolding supplies the comparison scene; none of this
is a prerequisite for today's movement and meal.

[Guide home](README.md) · Current route: [today](today.md) · Related: [populations](02-populations.md)
