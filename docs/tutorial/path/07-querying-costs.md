# 07. Ask each animal for the cost it owns

[Path home](README.md) · Previous: [owned costs](06-owned-costs.md) · Next: [a fair comparison](08-a-fair-comparison.md)

**Future checkpoint · 20–30 minutes after the component and spawn wiring are
reviewed.** Before you begin, the agent attaches costs to every animal, preserves
species defaults, prepares a no-food installed scenario and proves the component
attachment invariant. Your edit replaces the maintenance query and its rate
lookup in `crates/moss-sim/src/lessons.rs::spend_energy`. The subtraction rule stays.

Fern and another hare now carry different cost values, but a component does
nothing merely by existing. The running maintenance system still has to read it.
This session connects the ownership decision to behavior: after three ticks,
the default-rate hare should have 57 energy and the higher-cost hare 54.

## The query describes access, not a class hierarchy

Recall the original loop: it requested `Species` and mutable `Energy`, then
looked up a shared rate. The new loop requests `AnimalEnergyCosts` and mutable
`Energy` directly. `With<Creature>` keeps eligibility explicit. The shape of the
query tells us both which entities match and which data the system can change.

Read `Query<(&AnimalEnergyCosts, &mut Energy), With<Creature>>` as a request:
“Find entities with all three components; give me read access to costs and write
access to energy.” The tuple lists the two values delivered to each iteration.
`With<Creature>` is only a filter, so the loop gets no third `Creature` value.

An immutable `&AnimalEnergyCosts` borrow reads the baseline. A mutable
`&mut Energy` borrow changes the reserve. The animal's species remains useful for
grouping and future behavior, but it no longer has to participate in this rate
lookup. Two hares can therefore share all the same component types while storing
different numbers. Different values alone do not create different ECS archetypes.

Imagine three test entities starting at reserve 60. Where present, the cost is 1.
The second reference test below gives this table an executable example:

| Components on an entity | Result after one pass |
| --- | --- |
| `Creature`, `AnimalEnergyCosts`, `Energy` | Yields `(costs, energy)` and pays its own upkeep: **59**. |
| `Creature`, `Energy`, but no costs | Skipped because required data is missing: **60**. |
| Costs and energy, but no `Creature` | Skipped because the animal filter is not satisfied: **60**. |

Missing components need not produce a compiler error. A query can match zero
entities and quietly do nothing. That is why the agent checks attachment as
well as arithmetic: an animal skipped by the query should not receive free upkeep.
The incomplete animal in this diagnostic is deliberately synthetic. A query
can correctly skip a malformed animal while its spawn code is still wrong for
Moss. The production fixture must attach the required costs to every animal.

## Change the input, preserve the rule

In the existing `spend_energy` function, remove the shared `Res<SpeciesEnergyRules>`
parameter and request costs where the query previously requested species. The
loop then reads `costs.maintenance_units_per_tick` directly. Keep the saturating
subtraction you already wrote; the change is where its argument comes from.
The agent prepares the imports and test before the session.

Trace the first pass: Fern starts at 60, her component says 1, so she ends at 59.
The second hare starts at 60, its component says 2, so it ends at 58. The loop has
one rule; different input data accounts for the different result. Three such
passes leave 57 and 54.

## Use isolation to explain the numbers

Run the installed maintenance regression using the exact command the agent
records before your edit. It must initially expose the old species-based result
of 57/57 for these two hares, then pass with 57/54 after your query changes. Its
no-food scenario matters: if one animal traveled or ate, 57 and 54 would no
longer measure upkeep alone. Removing food from the scenario lets the real
schedule remain installed while those other actions have no opportunity to occur.

If both hares end at 57 in the live regression, inspect whether maintenance still
reads the species resource. If one stays at 60, inspect its component attachment
before changing arithmetic.

## Read the complete system and test

The **optional worked reference** defines the component locally and installs only
maintenance in a small Bevy schedule. Its first function is the complete system
shape; the tests below it supply worlds that Bevy uses to fill the query.
Your live component comes from session 06, so do not redeclare it. The prepared
live regression keeps the full installed schedule, with no food opportunity.
The first reference checks valid animals and their costs. The second changes
component presence to explain membership; it is a supplied diagnostic, not
another live exercise to complete.

<!-- runnable: session-07 -->
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

`saturating_sub` still spends up to the remaining reserve and stops at zero.
Changing where the cost is stored does not change what zero means: the final
case in the first test preserves an animal with zero reserve until a later
lifecycle lesson.

`0..3` is a half-open range, so the test runs three maintenance passes. Each
`schedule.run` supplies the system's query from the world. The test then looks up
each specific entity rather than comparing a sum that might hide one missed
animal.

In the membership test, `Energy` implements `Copy`, so each spawn receives its
own initial reserve of 60. The eligible entity reaching 59 proves the system did
run; unchanged controls alone could also mean nothing ran. The entity without
`Creature` has both requested data components, so only the filter excludes it.
A bare grass entity could not expose a missing filter: it lacks those data
components too. Neither test names nor variable names determine query membership.

In `for (costs, mut energy)`, `costs` is the read-only reference requested by the
query. Bevy wraps writable component access in a `Mut<Energy>` value; marking
that binding `mut` lets us write through it. This is the same reserve field as
before, not a temporary energy copy that disappears after the loop. The versioned
[Bevy query reference](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html)
is optional background for those access types.

If the test setup is unfamiliar, [reading a Rust test](../context/reading-a-test.md)
explains how setup, the call under test, and assertions fit together. The test's
`World` and `Schedule` are an explicit way to run the same kind of query as the
game, without opening a browser.

**Optional:** run the complete answer; expect two passing reference tests.

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 07
```

[What this checks](README.md#checking-your-work) · [Recorded evidence](verification.md)

The agent also checks explicit zero overrides, unchanged species defaults and
Reset reconstructing authored costs. The inspector must display the component
value actually used, not the old template value. Provenance is shown only if
the initializer recorded it.

Send: **“Session 07's same-species regression is green. Review query coverage,
preserved zero behavior and reset, then verify individual costs in the browser.”**
The animals now pay different upkeep. Next we will give that difference
a fair test, with the same journey and the same food opportunity for both.
