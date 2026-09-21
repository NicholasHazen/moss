# Observability — one life, one population, one world

**Status:** Shared logging direction. Implement only the pieces needed by the current behavior.

## Three views, one history

**Micro:** What is creature 17 doing, what can it currently perceive, and what happened to it?

**Population:** Why did the grazer count change? Which births and deaths contributed? How do energy, age, or attributes differ across groups?

**World:** How do resource supply, populations, and interventions develop across a run?

The connection matters more than a sophisticated dashboard. Start with an inspector and a small timeline. Add population measurements as soon as they help answer a question, not as a prebuilt analytics platform.

## Keep four kinds of data separate

| Data | Example | Appropriate treatment |
|---|---|---|
| Current truth | Energy reserve is 42; creature is resting. | Authoritative ECS state, read by the inspector. |
| Observed information | Food was visible at a location on tick 18. | Actor-specific data only when memory exists. |
| Semantic history | Creature 17 ate from patch 3 on tick 20. | One event, linked to both participants. |
| Measurements | There are 12 living grazers at tick 100. | Post-commit aggregate sample with explicit scope. |

A sampled chart cannot reconstruct every event. An event log without pending work cannot necessarily resume a simulation. A selected creature's inspector must not feed its omniscient information back into that creature's policy.

## Stable identity

Assign a `SimId` to each significant creature and resource. Combine it with a run identifier when records leave the current run. Keep event IDs stable within the run and allocate them in defined resolution order.

Do not persist Bevy `Entity` handles as identities. Later birth records link parent IDs to the child ID; a species/kind label is not a lineage. A dead creature can retain a small historical record after its live components are removed.

## A semantic event

Use a concrete Rust enum and a shared envelope before considering a generic event schema. The following is illustrative data, not a required JSON format:

```text
Event 81 — ConsumedFood
Tick: 20
Participants: eater=17, source=3
Place: cell (9, 4)
Cause: completed Eat activity, decision 76
Inputs: available biomass=8, eater capacity remaining=6
Changes: biomass 8→2, energy 42→48
```

Other events arrive with their features: activity changed, rest completed, birth, death with a recorded cause, predation, rejected interaction, parameter change, and explicit experiment intervention.

Emit a single shared predation event or explicitly linked predation/death events with distinct meanings. Do not count the same death twice. Initialization is not reproduction: seeded population counts and later birth counts remain distinct.

## Explain only the reasoning that exists

For a simple conditional policy, record the branch and its relevant inputs:

```text
Selected SeekFood because energy 18 was below the start threshold 25.
Patch 3 was observed within range and won the distance/ID tie-break.
```

Do not invent a utility score, emotion, or rationale that was not computed. If later utility scoring exists, keep the actual factors and referenced observations. Label speculative analysis separately from the recorded explanation.

Record meaningful activity changes rather than emitting an identical “still walking” decision every tick. Repeated failures may be coalesced with a count and first/last ticks; do not disguise coalescing as individual event retention.

## Measurements that answer concrete questions

Initial population samples can contain living count by kind, total available food, mean/minimum energy for living creatures, and cumulative births/deaths by recorded cause. Include tick, sample scope, units, and count used by a mean. An empty population has an unavailable mean, not an invented zero-energy individual.

Reconcile population accounting:

```text
living now = seeded population + births + explicit additions
           - deaths - explicit removals
```

Measurements should cover the configured world, not just the camera viewport. Sampling frequency should be visible; no graph should imply exact per-tick data from coarse samples. Later attribute histograms and lineages can build on the same identities and samples.

## Bounded memory without false completeness

Browser storage and memory are not an unlimited archive. Proposed stages:

**First:** a bounded in-memory event ring, a small selected-entity view, and run-level counters. Show the earliest retained tick/event and the number of evicted events. Counters may cover the whole run even when details do not; label those different coverage windows.

**Next:** periodic bounded aggregate samples, lightweight dead-entity summaries, and explicit JSON/CSV export. Put a budget on tombstones and participant indexes too; removing old history must also remove or mark stale index links.

**Later:** local persistence or richer export when a concrete workflow needs it. IndexedDB is a candidate, not a selected dependency. Browser-held data may be subject to quotas and eviction, so a stored run is not a promise of permanent archival. [R7](RESEARCH.md#r7-browser-storage)

At a storage or memory limit, surface the policy: evict visibly, pause recording, or pause the run until export. Do not grow indefinitely or silently claim full lifetime coverage. No SQLite or remote database is required to begin.

## Exports and future saves

A useful export includes schema/rules/build identifiers, configuration, seed and random algorithm when used, run/tick coverage, units, event-retention information, and actual interventions. Export large IDs safely for browser tooling, for example as decimal strings rather than numbers that a JavaScript reader could round. [R9](RESEARCH.md#r9-numeric-ids-in-browser-exports)

A future resumable snapshot additionally needs all authoritative state, ID allocation, random state, pending actions, relevant observations, counters, and version information. Validate imports; reject unsupported or corrupted data without silently resetting the world. Imports are not an early prerequisite.

## Debugging is not telemetry

Console diagnostics are useful but are not the simulation's history. Keep per-tick traces optional and bounded. Do not upload runs, diagnostics, or learning notes to any service by default.

Minimal tests should eventually verify: one food source cannot be consumed twice; one death is counted once; parent links match real births; aggregates reconcile; camera changes do not affect samples; and history eviction is visible. Add each test alongside the behavior it protects.
