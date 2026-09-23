# 14. Remove a creature without losing the reason

[Path home](README.md) · [Previous: deciding starvation](13-deciding-starvation.md) · [Next: read a scarcity run](15-read-a-scarcity-run.md)

**Future session · 20–30 minutes, after accepting the starvation policy.** Your
edit is one regression, planned as `starvation_is_recorded_once` in
`crates/moss-sim/tests/lifecycle.rs`. The agent prepares the end-of-tick resolver,
bounded death-history extension and inspector coverage first. The current
journal records initialization only; it has no death events or tombstones.

## Separate the rule from the structural change

Last session's predicate answered whether a creature finishes the allowed meals
with zero energy. Removing that creature is a different operation. An ECS query
borrows matching component data, while despawning changes which entities and
components exist. Keeping these operations in separate phases makes the Rust
borrows clear and gives us an explicit place to record the actual outcome.

Moss's existing reset already demonstrates the pattern: collect temporary Bevy
entity handles, end the query borrow, then despawn those entities. For starvation
we also copy each stable `SimId`. The Bevy handle addresses the live entity;
the stable ID identifies the participant in history after that handle is gone.
When information leaves a run, its identity also needs the run number.

## Give the terminal outcome one owner

The prepared resolver must run after allowed feeding and before publishing the
completed tick. Once it resolves a death, no later action may use that creature.
With immediate exclusive-world removal, later queries cannot find it. If future
systems queue deferred removal instead, they must first mark it unavailable
and have every later resolver honor that state. Merely queuing a despawn is
insufficient protection against two hunters receiving the same prey reward.

That hunter conflict is a future lesson. Here, make one starvation resolve once.
Calling the removal phase twice must not add a second death. Your installed
test will also tick again and confirm the removed creature cannot move or eat.
A correct event count alone would not prove that those other systems stopped.

The record also needs the right tick. During resolution the completed counter
still describes the previous tick; stamp the death with the executing tick,
`completed + 1`, using checked addition. For example, start with 119 completed
ticks and arrange a terminal outcome during night tick 120. After the tick,
both the clock and death record must say 120; initialization records remain at
tick 0. The prepared journal regression checks this boundary as well as the
absence of a duplicate on the following tick.

## Preserve evidence with an honest boundary

Start from the regression you want to protect: Fern has reached the accepted
terminal condition, another creature has positive energy, and resolving twice
must remove Fern only once. The surviving creature is useful evidence: a resolver
that deletes every animal would also make Fern disappear.

Your edit is the installed regression's literal expectations: one death, the
correct stable participant, no surviving entity for that ID, and no later food
or movement outcome involving it. Use the live command prepared with the test.
If the count becomes two, look for both a lifecycle system and another caller
recording the transition. If the entity persists, inspect deferred command
application before changing the rule.

The **optional worked reference** below uses actual Bevy queries and removal.
It returns the IDs actually removed for the existing journal's caller; it does
not introduce another history store. The test uses two terminal creatures,
inserted out of order, to make stable outcome ordering visible as well as the
surviving control. This is a smaller claim than the installed regression,
as [checking your work](README.md#checking-your-work) explains.

| Phase | What the code may still be holding |
| --- | --- |
| Find terminal creatures | Borrowed `Energy` and `SimId` values in the world. |
| Remove the copied candidates | Owned entity handles and IDs; no component references. |

Read the collection block and the removal loop as those two phases. The copied
IDs remain meaningful after the components that supplied them are gone.

<!-- runnable: session-14 -->
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

The iterator chain first filters for zero reserves, then maps each borrowed row
to copied handles and IDs. `collect` materializes that finite list. Once the
query iterator is finished, the loop can borrow the world mutably to remove
entities; it is no longer holding references into the storage being changed.
Here `Vec<_>` is inferred as `Vec<(Entity, SimId)>`: the closure returns those
owned values. `collect` stores whatever an iterator yields; it does not itself
copy borrowed components. The `*id` in this mapping keeps component references
out of the candidate list.
The standard loop would express the same two phases. This is a reason to collect
temporarily, not a rule that every ECS query needs an allocated list.

The returned vector contains actual outcomes, not merely the earlier candidates.
`World::despawn` returns whether removal occurred, and only successful removal
adds an ID. Calling the resolver again finds no terminal entity to remove, so
it produces an empty vector. The prepared caller must record each returned ID
once; a second caller must not separately record the same death.

The reference checks removal and the surviving control; the live regression
also uses Moss's journal and complete schedule. Bounded retention remains the
journal's job: it currently retains 32 events and reports evictions. A later
inspector may have whole-run death counts but only recent details, so it must
show those different coverage windows. Losing a retained detail is not evidence
that the death never happened.

There is a small trap in the phrase “since tick 120.” Suppose 35 events share
that tick. A 32-entry journal drops three of them, yet its oldest retained tick
still says 120:

```text
Tick 120:  [3 earlier records evicted] [32 later records retained]
```

The timestamp locates the first surviving detail; it does not promise every
event from that tick. The prepared inspector should say **first retained event**
and show the eviction count. It should also name which outcomes it collects.
Today's journal only records initialization, so even zero evictions cannot
explain each maintenance change. The [observability plan](../../design/observability.md#bounded-memory-without-false-completeness)
owns those coverage rules. There is no need to reimplement the journal to learn
removal.

## Leave a trace that explains the disappearance

Ask: **“Review my one-death regression, including the following tick, then verify
the browser's removal record and visible history coverage.”** The agent handles
the browser glue and records which checks actually ran. Stop after one creature's
terminal path is understandable; a full historical dashboard is not required.

Next we read a small scarcity run. You will combine living counts with seeded
counts and actual deaths, while remembering that the retained event list may
cover less time than the counters used for the population account.
