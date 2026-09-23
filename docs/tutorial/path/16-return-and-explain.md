# 16. Return to the world and explain what changed

[Path home](README.md) · [Previous: read a scarcity run](15-read-a-scarcity-run.md)

**Future consolidation session · 20–30 minutes.** Your edit is one comparison
assertion in a prepared repeatability test, planned in
`crates/moss-sim/tests/repeatability.rs`. The agent prepares a canonical snapshot
of the simulation fields used by the completed lessons and a browser reset
check. The reference below teaches how to compare snapshots using today's
component types. It does not run a simulation or pretend the future month is live.

## Recover the thread without rereading everything

Begin with Fern's current inspector and the last useful checkpoint in `NOW.md`.
You have followed a reserve through maintenance, actual travel and finite meals;
given food production a light source; and considered a terminal outcome with
an honest history. The recurring idea is that a small rule changes authoritative
state, while tests and inspection let you explain the consequence.

There is no need to recall every query signature before continuing. Trace one
value instead: which component stores Fern's reserve, which system changes it,
which schedule position gives that change meaning, and which observed result
would contradict your expectation? The answer travels across Rust, ECS and
simulation design without requiring a separate theory exercise for each.

Today, check a promise we have relied on throughout: the same starting state,
accepted inputs and executed ticks should produce the same relevant outcomes.
First we need a comparison that notices a changed instruction without mistaking
query order for a changed animal. Then we can use that comparison on an actual
run. This is bounded repeatability, not a general save or replay platform.

## Compare identities and values, not presentation

ECS iteration order is not the order of a creature's story. A comparison gathers
the selected simulation fields and sorts them by stable `SimId`. It excludes
Bevy entity handles and render transforms because those are implementation and
presentation details. This agreed representation is the *canonical snapshot*:
equivalent values appear in the same order. Stable IDs are scoped to a run;
comparing two reset runs also requires the same authored scenario and rules.
Reset advances the run
number, so compare equivalent biological values while accounting for that
expected metadata difference; retain the real run numbers in historical identity.

The reference creates equivalent component data in opposite insertion orders.
Both animals have reserve 57; Fern has an accepted food target and Flint does
not. These are authored test values, not observations produced by running ticks.
The snapshot should be equal because the relevant values match.

Now change Fern's reserve in one world to 56: the comparison must notice. Restore
57 and change only the accepted target from patch 3 to patch 4: it must notice
that too. The animals still look identical in a screenshot, but their next
instructions differ. A comparison of positions and reserves alone would miss
that distinction. Choosing what to include is part of defining the claim.

## Add the comparison that matters

In the prepared live test, add the comparison of two canonical results and
retain at least one independent literal expectation. Equal results alone could
mean two equally empty queries. The agent supplies its command and the complete
fixture described next. In the smaller snapshot check, expect reordered but
equal values to compare equal, and a changed reserve or target to compare
unequal. If a target change goes unnoticed, inspect which fields the snapshot
copied before questioning repeatability.

For the future installed check, start at completed tick 239 with an empty patch
and two seeking hares, each at reserve 1 with maintenance 1 and travel cost 2.
Neither has an authored target. Fern stands on the patch; the other hare is one
cell away. Dawn at executing tick 240 supplies 1 biomass. Both hares reach zero
after maintenance, but only Fern can eat without paying for a move.

Under the proposed, later-accepted after-meal policy, the expected result is one
unit grown, one eaten, Fern alive at reserve 1, patch biomass 0, and the other
hare removed with one starvation record. If the accepted policy differs, revise
that expectation first. The agent prepares the fixture and its inspection path;
your comparison protects a trace containing the preceding lessons' actual work.

The **optional worked reference** below teaches the smaller snapshot comparison
using today's components. Its `AnimalSnapshot` is a local read model, not a
component or another source of simulation state. It does not execute ticks;
[checking your work](README.md#checking-your-work) explains the separate live
checkpoint.

<!-- runnable: session-16 -->
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

`Option<&FoodTarget>` keeps an animal in the query even when that component is
absent. Requiring `&FoodTarget` instead would silently drop Flint. Mapping the
optional borrowed target into an optional copied `SimId` lets the snapshot own
its data after the query finishes, like session 14's collected IDs.

The live regression must also include the completed lessons' patch biomass,
activity, individual costs, clock and light, alongside the relevant outcomes.
Pending work or random state cannot be omitted while claiming a complete
continuation comparison. The reference's narrower snapshot is sufficient only
for the comparisons it actually makes.

## Close this arc with a portable explanation

Run the scenario twice from the same authored starting state. In browser review,
vary camera position and pauses while executing the same ticks and accepted
inputs. Compare the resulting simulation values, accounting for the new run
number after Reset. This checks browser input plumbing separately from the
snapshot example. The agent maintains the evidence and the next edit in `NOW.md`.

Ask: **“Review my repeatability comparison and help me explain one complete
growth-to-meal-to-lifecycle trace. Record the next small checkpoint.”** A short
explanation supported by one result is enough; this is not an examination.

Flint's first hunt remains a strong recommendation: bounded prey perception,
one accepted target, and exactly one rewarded consumption outcome would give a
second role something meaningful to do. Earlier arc reviews may already have
brought that work forward; completing every comparison here is not a graduation
requirement for meeting Flint. The agent should recommend the next useful slice
from the world you actually have and the questions it has raised.

Separate fatigue/rest and reproduction with parental costs and next-tick newborn
eligibility remain later possibilities. Each can reuse these habits of ownership,
ordering and explanation without requiring an all-at-once framework project.
