# One world: cumulative course ecology

This complete reference library connects the first ecological arc in one Bevy
world: multiple grazers share finite patches, daylight renews bounded supply,
individuals pay upkeep, meals transfer actual units, and starvation happens after
a chance to eat. Owned snapshots and bounded event history make those interactions
inspectable. It is an explicitly proposed course game model. It does not change
the live `moss-sim` crate, its schedule, or the movement assignment in
[`NOW.md`](../../NOW.md).

The original first-arc mode remains unchanged. Explicit population mode extends
this same world with maturation, funded births, and inherited upkeep. Mobile
checkpoint A adds local choice, paid grid travel, current contact, and spatial
births. Checkpoint B adds optional fatigue and committed rest; C adds local hunting
with immediate terminal exclusion. Their APIs and policies are documented below.
Randomness and biological calibration remain outside this reference. The first
two modes use authored feeding contacts; mobile mode derives them from actual positions.
These are repeatable authored comparisons, not evidence of long-term stability
or completion of the ecosystem curriculum.

## Run the native reference

The package is its own Cargo workspace. It pins Rust **1.93.1**, Bevy ECS
**0.18.1**, and an independent `Cargo.lock`; its only direct dependency is
`bevy_ecs` with default features disabled and `std` enabled. The local cache must
already contain those locked dependencies. No browser, renderer, or live Moss
dependency is needed for these commands.

From this directory (`learning/ecosystem`):

```sh
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 test --offline --locked
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 run --offline --locked --example compare
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 run --offline --locked --example populations
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 run --offline --locked --example mobile
cargo +1.93.1 fmt --all -- --check
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 clippy --offline --locked --all-targets -- -D warnings
```

The private target directory keeps these builds away from live-project and
single-file lesson artifacts. Native tests establish Rust and installed-schedule
behavior. This package does not itself supply a WASM host or browser observation;
those are separate integration work.

## Find the responsibility before editing it

| File | Responsibility |
| --- | --- |
| [`src/lib.rs`](src/lib.rs) | Thin public export surface. |
| [`src/model.rs`](src/model.rs) | Stable identities, authored scenarios, validation, components, owned readings, and actual tick flows. |
| [`src/simulation.rs`](src/simulation.rs) | Installation, explicit single-threaded schedule, reset, scenario switching, and read-only snapshots. |
| [`src/supply.rs`](src/supply.rs) | Bounded external input on lit simulation ticks. |
| [`src/feeding.rs`](src/feeding.rs) | Local contact eligibility, deterministic competing claims, and immediate two-store transfer. |
| [`src/lifecycle.rs`](src/lifecycle.rs) | Upkeep and permanent terminal transitions. |
| [`src/history.rs`](src/history.rs) | Bounded retained events and conservative whole-tick coverage. |
| [`tests/acceptance.rs`](tests/acceptance.rs) | Black-box public API acceptance cases; no access to private components. |
| [`examples/compare.rs`](examples/compare.rs) | Twelve-tick limited/generous supply comparison. |

Tests written by a learner can be added as another integration-test file and run
with normal Cargo. A prepared practice copy should preserve those tests separately
from the course's acceptance suite. This package has no HTML extraction or
trailing-test replacement convention.

## Public API and ownership

```rust
use moss_course_ecosystem::{CourseWorld, Scenario};

let mut world = CourseWorld::new(Scenario::limited_supply()).unwrap();
let before = world.snapshot(); // owned, sorted, and read-only with respect to the world
world.step();                  // exactly one complete simulation tick
let after = world.snapshot();
assert_eq!((before.tick, after.tick), (0, 1));

world.reset();                 // same selected scenario, new run ID, tick zero
world.reset_with(Scenario::generous_supply()).unwrap();
```

`CourseWorld::new(Scenario)` validates all authored inputs and returns
`Result<CourseWorld, ScenarioError>`. `step(&mut self)` executes the installed
schedule. `snapshot(&self)` returns an owned `Snapshot`. `reset(&mut self)` restores
the currently selected scenario. `reset_with(&mut self, Scenario)` validates before
changing anything, selects the new scenario, and starts the next run. A later
`reset` repeats that new selection. A rejected switch leaves both current state
and the saved reset scenario unchanged.

Run IDs begin at one and increase across both forms of reset. Tick, run, ledger,
and eviction counters use checked arithmetic; exhaustion panics rather than
wrapping. There is no recovery contract for a partially executed tick after a
panic. Ordinary invalid configuration is returned as an error before a new run
is installed.

`Scenario` exposes authored `GrazerSeed` and `PatchSeed` vectors, a validated
`Daylight`, and an event retention limit. IDs are `SimId(u32)` and must be unique
across both kinds of entity. Initial reserve/biomass cannot exceed capacity.
Empty populations, zero capacities, zero upkeep, zero meal requests, and zero
retained events have explicit valid meanings. Daylight rejects a zero cycle or
more lit ticks than cycle ticks; entirely dark and entirely lit cycles are valid.

`Snapshot` contains run and completed tick, daylight configuration and current
phase, sorted grazer/patch readings, the latest actual `TickLedger`, and an owned
`HistorySnapshot`. Tick zero has `lit: None` because no environmental phase has
executed. Every configured rate and feeding contact remains visible in the
readings. Modifying an owned reading cannot mutate the world. The internal roster
contains Bevy entity handles only; it is an index, not a mirrored biological world.

## Exact biological choices

One `Schedule`, using `ExecutorKind::SingleThreaded` and an explicit `.chain()`,
executes these phases:

1. Advance the tick and clear the latest flow ledger.
2. Charge upkeep to living grazers, limited by their current reserves.
3. Grow patches on lit ticks, limited by remaining patch capacity.
4. Let living grazers eat in ascending stable-ID order.
5. Mark any still-living zero-reserve grazer dead.
6. Mark the tick's history as collected.

All authored grazers initially enter alive, including an initial zero-reserve
grazer. Reaching zero during upkeep does not prevent its meal attempt. An available
meal can rescue it in this same tick. A grazer still at zero afterward becomes
permanently dead; it remains in the roster at zero for inspection, pays no more
upkeep, and receives no later meal. This is a selected game rule, not a claim about
real starvation. Keeping dead rows preserves the starting-cohort denominator.

Each grazer's optional `feeding_site` names the patch with which it has contact.
`None`, a missing ID, or an ID belonging to a grazer grants no contact and no meal.
All patches can still appear in the global inspector. Seeing a patch there does
not make it locally eligible. There is no automatic fallback to another patch.

A meal accepts the minimum of the grazer's per-tick request, current patch biomass,
and remaining reserve capacity. It subtracts from the patch and credits the grazer
immediately before the next claimant runs. A full or zero-demand consumer leaves
food for later consumers. A successful meal is one-for-one in these toy units.
The stable-ID ordering is deterministic and deliberately gives lower IDs priority;
it is not a fairness algorithm. Spawn order cannot change the result.

Upkeep is a sink; daylight growth is an external source. Eating moves units between
stores. Growth limits the accepted addition before adding, and feeding limits the
accepted transfer before either mutation, including near `u32::MAX`. The ledger
reports actual additions, paid upkeep, transferred food, and new starvations.
For the tested population sizes:

```text
stores_after + upkeep_paid = stores_before + growth_added
stores = sum(grazer reserves) + sum(patch biomass)
```

The reserve floor means unpaid upkeep is not retained as debt. It is consequently
important to compare actual upkeep paid with configured upkeep rather than
silently treating them as the same quantity.

The authored comparison uses four-tick days: ticks 1–2 are lit, 3–4 dark, then the
cycle repeats. Neither rules nor observation read wall time. Extra snapshots,
redraws, or delayed requests cannot create an additional dawn.

## Follow the connected result

Both comparison scenarios begin with grazer IDs 1 and 2 at reserve 3, capacity 8,
and meal request 2. ID 1 pays upkeep 2; ID 2 pays upkeep 1. Both contact patch 100,
which begins empty with capacity 8. Only external supply differs: limited adds up
to **2**, generous up to **6**, on each lit tick.

The native example produced these literal readings on September 23, 2026:

| Tick | Limited reserves 1 / 2 | Limited alive / biomass | Generous reserves 1 / 2 | Generous alive / biomass |
| --- | --- | --- | --- | --- |
| 1 | 3 / 2 | 2 / 0 | 3 / 4 | 2 / 2 |
| 2 | 3 / 1 | 2 / 0 | 3 / 5 | 2 / 4 |
| 3 | 1 / 0 | 1 / 0 | 3 / 6 | 2 / 0 |
| 4 | 0 / 0 | 0 / 0 | 1 / 5 | 2 / 0 |
| 8 | 0 / 0 | 0 / 4 | 0 / 7 | 1 / 0 |
| 12 | 0 / 0 | 0 / 8 | 0 / 8 | 1 / 5 |

At generous tick 3 both grazers eat from stored biomass, exhausting it before the
second dark tick. ID 1's larger upkeep then spends reserve faster. It dies at tick
8 despite having first claim on food; ID 2 remains alive through tick 12. The label
“generous” means greater external input, not guaranteed survival for every animal.
The initial test draft incorrectly expected both to survive; following this
literal trace corrected that expectation without changing the authored rules.

After both limited-supply grazers die, supply accumulates because no eligible
consumer remains. Observing biomass 8 at tick 12 therefore does not establish that
food was available when either grazer needed it. Current state and event history
answer different questions. These results establish this twelve-tick comparison,
not long-term stability, reproduction, adaptation, or an ecological forecast.

## History: unknown is different from zero

Positive actual growth, upkeep, and meals are recorded, along with each one-time
starvation. Events include run, tick, and stable subject/patch IDs. Zero transfers
and rejected meal attempts create no event; inspecting a complete interval can
still establish that it contains zero starvation events.

The journal retains at most `history_limit` events. When an event is evicted, its
entire tick is conservatively outside complete coverage even if other events from
that tick remain. `complete_after_tick` is an exclusive lower bound;
`collected_through_tick` is an inclusive upper bound. `evicted_events` reports the
actual number lost. `starvations_between(first, last)` returns `None` for an invalid,
future, or incompletely retained interval and `Some(0)` for a fully covered interval
with no starvation. A zero-length journal can lose tick 1 but still know that an
event-free tick 2 had zero transitions. Dead state never depends on retaining its
terminal event.

Within each phase, stable-ID ordering also makes event retention deterministic.
The reset operations clear retained events and coverage while advancing the run
identity. A saved owned report remains an observation of its old run.

## Observed verification and limits

- **19 public API acceptance tests passed** with `cargo +1.93.1 test --offline
  --locked`. They cover competing claims, no second reward, capacity and zero
  demand, absent contact, full-reserve upkeep, rescue before starvation, dead
  ineligibility, phase-sensitive literals, the twelve-tick comparison, accounting,
  immutable reads, spawn-order independence, bounded/zero history, reset and
  atomic scenario switching, integer limits, empty populations, invalid setup, and
  entirely lit/dark days.
- `compare` ran natively and printed all twelve readings for both scenarios.
- Pinned rustfmt and Clippy with `--all-targets -- -D warnings` passed.
- Two isolated copies under `learning/work/ecosystem-probe-*` compiled with
  deliberate faults and failed the intended acceptance test. Starvation before
  feeding produced `(0, Dead)` instead of `(2, Alive)`. Omitting patch depletion
  gave competing consumers `3/3` instead of `3/2`. The runnable probe script is
  scratch evidence at `learning/work/ecosystem-negative-probes.py`; the reference
  files were not mutated.

The first test run was 18 passed and one incorrect comparison expectation failed;
the corrected complete run is the 19-pass result above. There are no hidden ignored
tests. This section records native first-arc verification. It does not certify a
learner's later modifications, a browser host, teaching efficacy, or the full
original goal.

## Select the population extension explicitly

The first arc remains selected by `CourseWorld::new` and `reset_with`. It keeps
dead rows, has no birth policy, and returns `None` from `population_snapshot`.
`Scenario`, `Snapshot`, `TickLedger`, and `EventKind` retain their existing public
fields and variants. The original nineteen-test acceptance file is unchanged.
The course's `learning/checkpoints/shared-meadow` preserves the earlier source
for the first construction guide; later work extends the learner's saved project.

The additive APIs are:

```rust
use moss_course_ecosystem::{
    CourseWorld, FounderTrait, PopulationConfig, Scenario, UpkeepTrait,
};

let scenario = Scenario::generous_supply();
let founders = scenario.grazers.iter().map(|seed| FounderTrait {
    id: seed.id,
    upkeep: UpkeepTrait::new(u8::try_from(seed.maintenance_units_per_tick).unwrap()).unwrap(),
}).collect();
let config = PopulationConfig::new(founders);
let mut course = CourseWorld::new_population(scenario.clone(), config.clone()).unwrap();
course.step();
let stores = course.snapshot();
let population = course.population_snapshot().unwrap();
assert_eq!((stores.run, stores.tick), (population.run, population.tick));
course.reset(); // repeat this scenario AND population policy under the next run ID
course.reset_with_population(scenario, config).unwrap(); // select a new validated pair
course.reset_with(Scenario::limited_supply()).unwrap(); // explicitly turn population mode off
assert!(course.population_snapshot().is_none());
```

`new_population(Scenario, PopulationConfig)` returns
`Result<CourseWorld, PopulationError>`; `reset_with_population` returns
`Result<(), PopulationError>`. Invalid inputs leave current state, run identity,
history, and the saved reset selection unchanged. Population mode canonicalizes
authored grazer, patch, and founder arrays by stable ID so the **complete** owned
report is independent of insertion order. Ordinary first-arc construction retains
its original behavior. Reads never advance a tick.

`PopulationSnapshot` is supplemental version `moss-course-population-v1`. It owns
the initial scenario, validated policy, sorted `PopulationReading` values, living
juvenile/adult counts, lifetime `PopulationTotals`, latest `PopulationLedger`,
identity/accepted-birth cursors, and bounded `PopulationHistorySnapshot`. The
initial scenario is provenance for repeating the run, not a second evolving
world. `individuals` contains current living entities after completed ticks.
Each reading supplies stage, deadlines, actual `UpkeepTrait`, and optional
`BirthOrigin`; its ordinary `Snapshot` row supplies reserve, capacity, meal rate,
and fixed feeding contact. No second biological implementation is introduced.

| New file | Responsibility |
| --- | --- |
| [`src/population/config.rs`](src/population/config.rs) | Authored policy, bounded trait, and atomic input validation. |
| [`src/population.rs`](src/population.rs) | Stage data, population reports, maturation, terminal cleanup, and lifetime counters. |
| [`src/reproduction.rs`](src/reproduction.rs) | Read/revalidate a pair, preflight every fallible birth value, then commit payment and reserve a deferred child slot. |
| [`src/population/history.rs`](src/population/history.rs) | Versioned birth/maturation events with explicit retained coverage. |
| [`src/simulation/tests.rs`](src/simulation/tests.rs) | Internal deferred-command, newborn-eligibility, and preflight-exhaustion checks. |
| [`tests/population.rs`](tests/population.rs) | Fifteen black-box public population acceptance tests. |
| [`examples/populations.rs`](examples/populations.rs) | Fully declared forty-tick matched-cohort experiment. |

## Population policy and transaction boundary

`PopulationConfig::new(founders)` selects contribution **2** and dissipated birth
cost **1** per parent, maturity delay **2** ticks, parental cooldown **3** ticks,
child capacity **8**, child meal request **2**, maximum **8** living entities,
mutation cycle `[0]`, and population history limit **128**. Founders begin adult
and first become eligible at tick 1. Every authored grazer needs exactly one
founder trait, whose value must agree with its actual maintenance rate.

`UpkeepTrait::new` accepts only values **1–3**. The value is the actual number of
upkeep units requested on each eligible tick, not a label in the inspector.
Mutation entries are `i8` values restricted to **-1, 0, +1**; an empty cycle or
another value is an input error. Delays must be positive, the living limit must be
positive and accommodate all founders, contributions must be positive, and child
capacity must hold both contributions. Payment arithmetic and initial child-ID
availability are checked during validation. Zero history and zero child meal
request remain valid choices. Population mode's trait bounds deliberately narrow
the first arc's unrestricted upkeep configuration.

The explicit single-threaded population schedule is:

```text
advance tick / clear both ledgers
  → mature eligible juveniles
  → upkeep → bounded daylight growth → competing meals → terminal starvation
  → resolve funded births → ApplyDeferred
  → remove dead entities → ApplyDeferred
  → complete both histories and lifetime counts
```

Within the birth phase, eligible adults are grouped by shared feeding-site ID,
groups are visited in site-ID order, and members pair in ascending stable-ID order
as disjoint pairs. There is no sex, compatibility score, mate search, or spatial
proximity rule. The shared non-absent site must currently resolve to a **Patch**;
an empty real patch still qualifies, while a missing ID or grazer ID does not.
The child inherits that accepted contact. A globally visible patch is not
automatically a local contact.

Both parents must be alive, eligible this tick, past cooldown, and able to pay
contribution plus cost while retaining at least one reserve unit. Each pays 3
under the default policy: 2 transfers to the child's initial reserve and 1 leaves
the modeled stores. The child therefore starts with 4; neither parent can fund it
alone. Births are resolved after starvation, so a dead parent cannot reproduce and
its vacated slot is available this tick. A selected pair denied specifically by
the living cap increments `capacity_blocked_pairs`; `capacity_blocked_ticks`
increments once if at least one such denial occurred. A full world with no funded
eligible pair does not count a blocked tick.

Preflight checks both parents again and computes deadlines, the next ID, accepted
birth ordinal, child values, transfer/cost ledger additions, and event-eviction
headroom before the first debit. Rejections cannot spend one parent's reserve,
advance inheritance, reserve a slot, or queue a child. The cap-denial count itself
is an intentional observation. After acceptance, both costs and cooldowns change
immediately, and an available slot is reserved before `Commands::spawn` queues the
child. A second pair cannot claim that pending slot. `ApplyDeferred` makes queued
children visible at the explicit phase boundary. Re-running birth resolution in
the same tick cannot reward the original pair again.

A child born at tick `t` has `first_eligible_tick = t + 1`; visibility alone grants
no maintenance or meal in the birth tick. It matures at `t + maturation_ticks`,
before that tick's other biological phases, and its initial birth deadline is
that maturity tick. It does not inherit a parent's cooldown. Parents' next birth
deadline becomes `t + cooldown_ticks`. Dead entities are removed in population
mode, so living storage stays bounded; descendants retain their parents' stable
IDs in owned origin data even after a parent disappears.

Child IDs begin one above the maximum authored grazer **or patch** ID. They are
never recycled within a run. Using the last representable ID exhausts the next-ID
cursor; a later attempted birth panics before debit. Counter/deadline exhaustion
also panics before that birth's mutation. This is a per-birth guarantee: completed
upkeep, growth, meals, or earlier births are not rolled back if a later operation
panics. The public API does not promise to resume a partially executed tick.

Inheritance algorithm `upkeep-copy-cycle-v1` uses the zero-based accepted-birth
ordinal. Even ordinals copy the lower-ID parent's trait; odd ordinals copy the
higher-ID parent's. The same ordinal indexes the mutation cycle, the result clamps
to 1–3, and `BirthOrigin` retains both the proposed and actually applied delta.
Rejected pairs consume neither cursor. This is an authored deterministic policy,
not Mendelian genetics, random mutation, or evidence of evolution. In particular,
the model gives greater upkeep no compensating advantage.

## Literal birth trace and accounting

The population acceptance fixture starts both founders at reserve 5, capacity 8,
meal request 2, upkeep 1, contact 100. Patch 100 starts empty, capacity 20, adding
up to 6 on every tick of a fully lit four-tick cycle. It uses the default policy:

| Completed tick | Reserves by stable ID | Patch biomass | Population consequence |
| --- | --- | --- | --- |
| 1 | `1:3, 2:3, 101:4` | 2 | Parents each reach 6 before paying 3; child 101 is juvenile. |
| 2 | `1:4, 2:4, 101:5` | 2 | Child 101 takes its first maintenance and meal. |
| 3 | `1:5, 2:5, 101:6` | 2 | Child 101 matures; founders remain on cooldown. |
| 4 | `1:3, 2:3, 101:7, 102:4` | 2 | The original pair produces child 102. |

The tick-4 population ledger reports transfer 4 and dissipation 2. Transfer moves
existing units; it is not new supply. Across each completed population tick:

```text
stores_after + actual_upkeep + birth_dissipation = stores_before + actual_growth
living + total_starvations = initial_founders + total_births
living = juveniles + adults ≤ configured_living_limit
```

Starvation removes a zero-reserve entity, so that cleanup discards no stored
energy. The tests use widened sums and check these equations at every tick of
eight forty-tick runs. Actual upkeep can be less than its trait value when reserve
hits the floor; a configured demand must not be silently substituted into the
energy equation.

## Forty-tick controlled comparisons

The native `populations` example declares four two-founder upkeep cohorts:
`[1,1]`, `[1,2]`, `[2,2]`, and `[1,3]`, assigned respectively to IDs 1 and 2.
Every founder starts at reserve 5, capacity 8, meal request 2, and contact 100.
The shared patch starts at zero with capacity 20. All population defaults above
remain fixed. Both histories retain at most 128 events. Each run starts fresh and
executes exactly **40 ticks**.

For each cohort, the environmental comparison changes the temporal supply
pattern while retaining a requested budget of 24 per four-tick cycle: **steady**
adds up to 6 on each of four lit ticks; **pulsed** adds up to 12 on the first two
ticks and nothing on the next two. This is a matched supply-budget intervention,
not one parameter with identical light exposure. Capacity could make actual
accepted input differ; in these eight observed runs it was exactly **240** in
each. Within each matched pair, founders, IDs, contacts, and population policy are
identical. Across cohorts, the declared trait assignment changes.

Observed September 23, 2026:

| Cohort / supply | Living (juvenile / adult) | Births / deaths | Living trait counts (1 / 2 / 3) | Cap-denied pairs / ticks |
| --- | --- | --- | --- | --- |
| 1,1 / steady | 6 (1 / 5) | 15 / 11 | 6 / 0 / 0 | 0 / 0 |
| 1,1 / pulsed | 5 (1 / 4) | 15 / 12 | 5 / 0 / 0 | 1 / 1 |
| 1,2 / steady | 5 (1 / 4) | 14 / 11 | 4 / 1 / 0 | 0 / 0 |
| 1,2 / pulsed | 5 (0 / 5) | 17 / 14 | 5 / 0 / 0 | 0 / 0 |
| 2,2 / steady | 3 (0 / 3) | 1 / 0 | 0 / 3 / 0 | 0 / 0 |
| 2,2 / pulsed | 3 (0 / 3) | 1 / 0 | 0 / 3 / 0 | 0 / 0 |
| 1,3 / steady | 5 (1 / 4) | 14 / 11 | 4 / 0 / 1 | 0 / 0 |
| 1,3 / pulsed | 5 (0 / 5) | 17 / 14 | 5 / 0 / 0 | 0 / 0 |

| Cohort / supply | Actual upkeep | Birth transfer / dissipation | Final reserve sum / biomass | Base complete ticks; evicted events |
| --- | --- | --- | --- | --- |
| 1,1 / steady | 200 | 60 / 30 | 20 / 0 | `(28,40]`; 275 |
| 1,1 / pulsed | 208 | 60 / 30 | 12 / 0 | `(26,40]`; 233 |
| 1,2 / steady | 208 | 56 / 28 | 14 / 0 | `(26,40]`; 211 |
| 1,2 / pulsed | 210 | 68 / 34 | 6 / 0 | `(27,40]`; 228 |
| 2,2 / steady | 238 | 4 / 2 | 8 / 2 | `(22,40]`; 150 |
| 2,2 / pulsed | 238 | 4 / 2 | 8 / 2 | `(21,40]`; 130 |
| 1,3 / steady | 208 | 56 / 28 | 14 / 0 | `(26,40]`; 211 |
| 1,3 / pulsed | 210 | 68 / 34 | 6 / 0 | `(27,40]`; 228 |

The population journal retains full `(0,40]` coverage with zero evictions in all
eight runs, so `births_between(1,40)` returns the corresponding known birth count.
The base journal has lost early events, so `starvations_between(1,40)` is `None`
in **all eight**, including the no-death `[2,2]` runs. Lifetime totals are retained
counters; they remain known independently of the journal's ability to explain an
interval. Born and Matured events are in the supplemental journal; starvation
events retain the original base schema. A child's surviving origin record is
not a promise to retain every ancestor forever.

More births do not necessarily produce more living entities at tick 40: the
`[1,2]` pulsed case has three more births and three more deaths than its steady
match. The `[1,2]` and `[1,3]` endpoint flows coincide despite different configured
upkeep; the reserve floor and actual accepted meals matter. Equal endpoints do
not establish identical paths or equivalent traits. These eight authored runs
are neither independent statistical samples nor a claim of long-run stability,
adaptation, ecological calibration, or a universally preferable supply pattern.

## Population verification and remaining boundary

At the population checkpoint, the pinned offline native run passed **38 tests**: the unchanged 19 first-arc
acceptance cases, 15 top-level population acceptance cases, and 4 focused internal
tests. Population cases cover literal phases, inheritance changing actual upkeep,
clamped deltas and donor alternation, immediate maturity eligibility, real versus
phantom contact, positive parental remainder, pending-slot reservation, cap
accounting, terminal slot release, retained ancestry, history limits, forty-tick
flow/census equations, complete-report insertion-order independence, owned reads,
empty populations, reset modes, and atomic invalid configuration.

The internal checks observe a queued child before and after applying commands,
then retry birth resolution in the same tick. Separate late maintenance and meal
checks assert the newborn's state **and** absence of child events/ledger flows;
opposing illegal changes cannot cancel and pass. Nine preflight-exhaustion cases
cover first-action/maturity/cooldown deadlines, accepted birth count, per-tick birth
count, transfer/cost sums, exhausted child identity, and event-eviction count. Each
checks unchanged reports after explicitly flushing the rejected queue.

Four scratch copies compiled successfully and failed the intended assertion:
removing the newborn maintenance guard changed reserve 4 to 3; removing its meal
guard changed 4 to 6; omitting the second parental debit produced reserves
`3/6/4` instead of `3/3/4`; omitting immediate slot reservation produced six living
entities under a cap of five. The reference remained unchanged. The reproducible
scratch driver is `learning/work/ecosystem-population-negative-probes.py`, with
each result in its own `ecosystem-population-probe-*` directory.

Pinned rustfmt and Clippy (`--all-targets -- -D warnings`) passed. Probe builds use
separate target directories so deliberately broken artifacts cannot contaminate
the reference's incremental cache.

`tests/acceptance.rs` still has SHA-256
`1968bd2f287779574689c63cc6476610827a4c07f1086fa69a7e0e99b6e1bb9c`.
This native extension does not itself validate a browser adapter, a learner's
later implementation, or the complete course. Rest and predation remain subsequent
interacting work; mobile checkpoint A below adds spatial choice and travel. The
live Moss crate and active movement assignment are untouched.

## Mobile checkpoint A: local choice and paid travel

This mode carries the same population and its inherited upkeep into a bounded
integer grid. It adds no rest, fatigue, hunting, pathfinding, random exploration,
collision avoidance, or new inherited attributes. Animals may share cells;
patches have distinct cells. `MobileScenario::short_journey()` and `::meadow()`
are complete authored presets defined in
[`src/mobile/scenarios.rs`](src/mobile/scenarios.rs), shared by the native example
and callers that want the same preview input.

```rust
use moss_course_ecosystem::{CourseWorld, MobileScenario};

let mut course = CourseWorld::new_mobile(MobileScenario::short_journey()).unwrap();
course.step();
let report = course.mobile_snapshot().unwrap();
assert_eq!(report.grazers[0].cell.x, 1);
assert_eq!(report.base.grazers[0].reserve_units, 3);
course.reset(); // same complete mobile selection, next run ID, tick zero
course.reset_with_mobile(MobileScenario::meadow()).unwrap();
```

`MobileScenario::new(scenario, population, space)` takes the existing `Scenario`
and `PopulationConfig`, plus a `SpatialConfig`. Its fields are private so later
optional policies need not add mandatory struct-literal arguments. Borrowed
`scenario()`, `population()`, and `space()` accessors expose authored inputs;
`history_limit()` reads the mobile journal bound, and `with_history_limit(limit)`
selects a bound, default 128. The constructor records raw inputs; `new_mobile` and
`reset_with_mobile` validate the whole configuration and return `MobileError`.

The spatial types are `Cell { x, y }`, `GridBounds { width_cells, height_cells }`,
`Motion { sensing_radius, max_cells_per_tick, travel_units_per_cell }`,
`GrazerPlacement { id, cell, motion }`, and `PatchPlacement { id, cell }`.
`SpatialConfig` holds `bounds`, grazer/patch placement vectors, and `newborn_motion`.
Coordinates and rates are `u32`. Bounds must be positive; cells must be inside
them. Each authored entity needs exactly one matching placement, and two patches
cannot occupy one cell. Zero speed permits a stationary actor; radius zero sees
only its own cell. Travel cost must be positive, including the newborn template.
Initial grazer `feeding_site` must be `None`: a fixed token cannot grant remote
contact in this mode. The old two constructors keep their previous meanings.

Invalid mobile configuration preserves current state, run ID, journals, and the
saved reset selection. `reset_with_population` explicitly returns to fixed-contact
population mode; `reset_with` selects the first arc. `mobile_snapshot` returns
`None` in either older mode. A mobile `reset` recreates both the scenario and its
spatial configuration with empty observations/targets and fresh counters.

| File | Mobile responsibility |
| --- | --- |
| [`src/mobile/config.rs`](src/mobile/config.rs) | Public authored types and validation/normalization. |
| [`src/mobile.rs`](src/mobile.rs) | Spatial components, installation, contact projection, and complete owned report. |
| [`src/mobile/perception.rs`](src/mobile/perception.rs) | Copied local observations, retained-target policy, and target-change outcomes. |
| [`src/mobile/movement.rs`](src/mobile/movement.rs) | Affordable bounded x-then-y movement, actual costs, and a per-tick action guard. |
| [`src/mobile/history.rs`](src/mobile/history.rs) | Target changes, completed travel, and birth placement with retained coverage. |
| [`tests/mobile.rs`](tests/mobile.rs) | Fifteen additional public acceptance cases. |
| [`src/simulation/mobile_tests.rs`](src/simulation/mobile_tests.rs) | Intervening changes, current-contact revalidation, and late newborn/preflight probes. |

The shared feeding and reproduction adapters now read optional mobile components.
Their fixed-contact paths remain unchanged. Small named query-data aliases keep
those concrete access lists readable; they introduce no generic rule registry or
second biology implementation.

### One tick and three kinds of knowledge

The single-threaded schedule advances the tick and clears all ledgers, matures
juveniles, charges upkeep, grows patches, collects **all** local observations,
chooses targets, travels, derives current contacts, feeds, resolves starvation,
resolves births, applies queued births, removes dead entities, applies cleanup,
then completes the three journals. The observation phase therefore precedes
every actor's movement; it is not recomputed opportunistically for each claimant.

Local sensing uses Manhattan distance with widened arithmetic. The adapter copies
all patches within the actor's radius, including empty ones, and sorts them by
stable ID. The selector receives that owned local view. It retains a current
target while that patch is still locally observed and nonempty; otherwise it
chooses the smallest `(distance, stable ID)` among eligible readings. No local
opportunity means no target and no wandering. The global inspector may still
show distant food. Knowledge in that table is not actor perception.

`PatchTarget` records `patch`, `observed_cell`, and `observed_tick`. It describes
an intention based on this tick's observation, not reserved food. Travel follows
that observed destination and checks that the target still has a current patch
identity; it does not steer from a newly fetched global position. Positive
biomass is rechecked for the meal, not guaranteed by the old reading. Current
contact requires the selected patch's actual cell to equal the grazer's cell at
feeding time. The feed adapter rechecks this even if an earlier derived contact
token has gone stale. Missing/empty/wrong-contact targets do not redirect the
action toward another patch in the same tick.

Movement accepts the minimum of remaining Manhattan distance, the configured
cell budget, and affordable whole steps. It takes x steps first, then y; arithmetic
computes this bounded prefix directly rather than looping an unbounded number of
times. The actual accepted distance alone spends reserve. Zero speed, no target,
arrival, a removed patch, and unaffordable movement spend zero travel units.
Spending the last reserve on arrival is permitted: a same-tick meal may rescue
the grazer before starvation, just as upkeep at zero can be rescued in the
earlier modes.

The per-actor `last_travel_tick` records the last **travel-phase opportunity**,
even if it accepted zero cells. It prevents a second late invocation from moving
again in that tick. It is not the timestamp of the last nonzero movement; use
actual `Travelled` events for that question. First-eligible guards apply to
observation, choice, movement, contact, and the existing energy actions.

Spatial births keep the same two-parent payment, cooldown, inheritance, allocator,
and cap rules. Both parents must currently be at their selected real shared patch
cell, revalidated during the birth transaction. The patch may have become empty
after their meals. A child receives that cell, the configured newborn motion,
inherited upkeep, and next-tick eligibility. Its target, local observation, and
travel-opportunity timestamp start absent; its derived feeding contact starts
`None`. Birth placement and mobile journal headroom are prepared before either
parent pays. Deferred spawning never leaves a completed-tick child without its
spatial components.

### Complete reports and actual outcomes

`MobileSnapshot` is version `moss-course-mobile-a-v1`. It includes run/tick,
canonical complete `initial: MobileScenario`, `base: Snapshot`,
`population: PopulationSnapshot`, sorted mobile grazer rows and patch placements,
latest `MobileLedger`, lifetime `MobileTotals`, and `MobileHistorySnapshot`.
Each mobile grazer row includes its cell/motion, target, observation timestamp
and `observed_cell` (its own position when that view was collected), copied visible
patches, and travel-opportunity timestamp. `None` observation means
uncollected; an observed empty vector means no local patch was seen. Mutating any
owned report cannot change the world. All authored ID arrays are normalized,
including the arrays inside the report's initial configuration.

Both mobile ledger and totals expose actual `travel_cells`, `travel_units`, and
`target_changes`. Target ID changes are journaled; refreshing the same target's
observation timestamp does not invent another change. New events are
`TargetChanged`, `Travelled`, and `BornAt`, each with run/tick and stable IDs.
`travelled_cells_between` returns `None` for invalid, future, or incomplete
coverage and `Some(0)` for a fully covered interval without travel. Mobile costs
are additional sinks, not modifications to the old ledger's meaning:

```text
stores_after + actual_upkeep + actual_travel_cost + birth_dissipation
    = stores_before + accepted_growth
```

The observed short journey starts at `(0,0)`, reserve 5, upkeep 1, capacity 8,
meal request 2, sight 3, speed 1, and travel cost 1. Patch 100 at `(2,0)` contains
three units with no renewal:

| Tick | Cell | Reserve | Biomass | Travel cells / cost | Meal |
| --- | --- | --- | --- | --- | --- |
| 1 | `(1,0)` | 3 | 3 | 1 / 1 | 0 |
| 2 | `(2,0)` | 3 | 1 | 1 / 1 | 2 |
| 3 | `(2,0)` | 3 | 0 | 0 / 0 | 1 |

At tick 1 the copied view has `observed_cell = Some(Cell { x: 0, y: 0 })`
while the current cell is `(1,0)`. At tick 2 those values are `(1,0)` and
`(2,0)`. The owned report keeps both facts, without reconstructing the observation
origin from a potentially evicted travel event.

The longer `meadow` preset declares bounds `11 × 5`; grazers 1–4 at `(2,2)`,
`(3,1)`, `(2,3)`, `(3,2)` with reserve 18, capacity 24, meal 5, upkeep
`[1,1,2,2]`, sight 5, speed 1, and travel cost 1. Patches 1000/1001 at `(3,2)`/
`(7,2)` have capacity 18, initial biomass 6/18, and growth 8/4 on the first four
ticks of each eight-tick day. The grazer cap is 10; other population defaults
remain unchanged. Newborns use sight 5/speed 1/cost 1 and the population default
reserve contribution, capacity 8, and meal 2. All three history limits are 128.

The native example ran exactly 120 ticks on September 23, 2026:

| Tick | Living / juvenile | Total births / starvations | Travel cells (cost equals cells) | Reserve sum / biomass |
| --- | --- | --- | --- | --- |
| 1 | 5 / 1 | 1 / 0 | 3 | 75 / 18 |
| 2 | 6 / 2 | 2 / 0 | 4 | 73 / 18 |
| 4 | 6 / 1 | 3 / 1 | 4 | 69 / 18 |
| 8 | 4 / 1 | 4 / 4 | 21 | 39 / 3 |
| 12 | 3 / 1 | 5 / 6 | 21 | 38 / 18 |
| 40 | 3 / 1 | 12 / 13 | 63 | 37 / 6 |
| 80 | 3 / 1 | 22 / 23 | 116 | 34 / 8 |
| 120 | 4 / 1 | 32 / 32 | 173 | 42 / 4 |

Final IDs 1, 2, 1031, and 1033 are at `(7,2)`. Accepted growth totals 606, actual
upkeep 419, birth transfer 128, birth dissipation 64, target changes 89, and
cap-denied pairs/ticks zero. Initial stores 96 plus growth 606 equal final stores
46 plus upkeep 419, travel 173, and birth cost 64. This is one authored run with
repeated travel and turnover, not calibrated ecology, measured evolution, or a
claim about long-run stability.

At tick 120, the base journal has complete `(99,120]` coverage with 592 evictions,
population history retains `(0,120]` with zero evictions, and mobile history has
`(69,120]` with 166 evictions. Full-range travel history is therefore unknown even
though its lifetime counter is known to be 173. Current positions cannot explain
every earlier journey after the relevant events are lost.

### Mobile verification boundary

The current pinned offline run passed **59 tests**: the unchanged nineteen
first-arc tests, unchanged fifteen population tests, fifteen public mobile tests,
and ten internal tests including the six new spatial probes. They cover literal
travel, partial/zero speed, extreme bounds, local versus global visibility,
retained targets when nearer supply renews, real competing consumption, spatial
births, current-contact rejection, invalid reset atomicity, all mode switches,
complete-report insertion-order independence, history coverage, and per-tick
120-tick accounting. Current rustfmt and strict Clippy passed.

The exact learner-authored test in `learning/content/mobile.html` was decoded and
compiled in a separate project with its own target directory: one test passed.
Its two grazers move from cell 0 to 1, both having observed two units at patch 100.
Only the first eats, yielding reserve 4/2. Patch 101 at cell 2 was outside that
initial local view. On the next tick both can observe it, travel there, and end at
reserve 4/2 again. This verifies the proposed lesson answer without replacing the
learner's independent test-writing task.

Five isolated negative copies compiled and then failed the intended check:
removing the travel debit broke the literal journey; trusting a stale contact
cell broke the feeding revalidation probe; removing the first-eligible travel
guard let a newborn act; skipping spatial parent revalidation admitted a birth
from a stale contact; and making the observer's grazer `Position` access mutable
conflicted with its patch-position query. That last change failed at Bevy schedule
initialization with `error[B0001]`, rather than a Rust borrowing diagnostic.
Restoring `&Position` repairs it because observation changes `MobileActor`, not
the actor's position. Every copy used a separate Cargo target directory. These
probes establish that the checks distinguish the tested defects; they do not
prove all possible schedules or future spatial policies correct.

`tests/acceptance.rs` remains byte-identical with the SHA above;
`tests/population.rs` remains byte-identical with SHA-256
`5185f6dc7a22ef368935be82151ca2896100c7975f248b113514fd413ee64fe4`.
Browser adapters, rendered map/keyboard checks, later rest/predation, and the full
goal remain separate work. No live source, learner-owned current movement helper,
or assignment was changed by this checkpoint.

## Checkpoint B: optional fatigue and committed rest

`MobileScenario::new(...)` still defaults to checkpoint A. Enable the additional
policy with `scenario.with_rest(RestPolicy::default())`; its `rest()` accessor
returns `Option<&RestPolicy>`. `CourseWorld::new_mobile`, `reset_with_mobile`, and
`reset` select, validate, and repeat the whole configured mode. Rest is absent
from the first-arc and fixed-contact modes. No live Moss source or learner-owned
movement exercise changes here.

```rust
use moss_course_ecosystem::{CourseWorld, MobileError, MobileScenario, RestPolicy};
fn main() -> Result<(), MobileError> {
let mut world = CourseWorld::new_mobile(MobileScenario::resting_journey())?;
world.step();
let space = world.mobile_snapshot().unwrap();
let rest = world.rest_snapshot().unwrap();
assert_eq!(space.grazers[0].cell.x, 1);
assert_eq!(rest.grazers[0].fatigue_points, 2);
world.reset_with_mobile(MobileScenario::meadow().with_rest(RestPolicy {
    minimum_rest_ticks: 3,
    ..RestPolicy::default()
}))?;
Ok(())
}
```

This code is an API excerpt; run the complete native example with:

```sh
CARGO_TARGET_DIR=learning/work/ecosystem-target cargo +1.93.1 run --offline --locked --manifest-path learning/ecosystem/Cargo.toml --example resting
```

`RestPolicy` fields and defaults are `maximum_fatigue_points: 10`,
`effort_points_per_cell: 2`, `enter_at_points: 6`, `exit_at_points: 2`,
`minimum_rest_ticks: 2`, `recovery_points_per_tick: 2`, and `history_limit: 128`.
The first six are `u32`; the history limit is `usize`. Validation requires
`exit < entry <= maximum`, positive effort no greater than maximum, positive
recovery, and positive minimum commitment. Recovery greater than the maximum is
valid and recovers only the fatigue actually present. Invalid rest configuration
returns `MobileError::InvalidRestPolicy` before replacing the current world or
saved reset configuration. These are proposed integer game rules, not measured
animal physiology.

### State, decisions, and effects

Each rest-enabled mobile actor begins with zero fatigue and `Activity::Foraging`.
The other enum case is `Activity::Resting { remaining_ticks }`. This stored
commitment is distinct from the fatigue exit condition. A pure transition helper
selects an activity without charging time or recovering fatigue. Its adapter runs
after upkeep/growth and before observation. At or above the entry threshold it
selects rest; the following target-selection phase cancels the old target. A
resting actor gets no new perception, no travel, no feeding contact, no meal, and
no parental transaction. The shared upkeep system continues charging reserve.

Only the later executed rest phase recovers `min(fatigue, recovery)` and reduces
the commitment by one toward zero. It records one action even when actual
recovery is zero. The `last_rest_tick` guard prevents a repeated invocation from
performing another action. Waking requires both zero remaining commitment and
fatigue at or below the exit threshold, at a subsequent decision tick. It enters
foraging with no retained target, then receives a fresh current-tick observation.
The last copied view remains timestamped while resting; it is historical local
knowledge, not a fresh current view. Reads never run either decision or effect.

Travel additionally accepts only whole steps whose full effort fits the remaining
fatigue capacity. It adds exactly `accepted_cells * effort_points_per_cell`,
checked before position/reserve mutations. It never saturates away an accepted
step's effort. No target, no movement, an unaffordable step, and a stationary meal
add zero fatigue. This differs intentionally from lesson 21's abstract effort on
every foraging tick: the cumulative world now charges actual spatial actions.

Newborns receive zero fatigue, foraging activity, and absent action/transition
timestamps. The existing first-eligible tick prevents an erroneous late rest
phase from changing a visible newborn. Resting parents are rejected again inside
the birth transaction, including a deliberately injected stale target/contact.
Upkeep, actual travel costs, and birth dissipation remain energy sinks; fatigue
recovery produces no energy. A hungry resting actor can die beside food, because
this policy has no hunger override.

The mobile schedule is still single-threaded and fully ordered. Its explicit
phases are now grouped into three chained tuples to stay within Bevy 0.18.1's
supported tuple arity; the outer chain preserves the order between groups.

```text
begin tick/clear ledgers → mature → upkeep → grow
→ decide rest/wake → observe foragers → select/cancel target
→ travel → execute rest → derive contacts → meals → starvation
→ births → ApplyDeferred → cleanup → ApplyDeferred → complete journals
```

### Reading the supplemental report

The earlier public `Snapshot`, `PopulationSnapshot`, `MobileSnapshot`, mobile
ledger, and mobile event variants keep their meanings. Call
`rest_snapshot() -> Option<RestSnapshot>` for the supplemental version
`moss-course-rest-b-v1`. `None` means the policy is disabled. A complete observation
of a rest-enabled run includes **both** the mobile and rest reports; the older
projection alone does not contain every future-relevant activity state.

`RestSnapshot` owns `run`, `tick`, `policy`, sorted `grazers`, latest `ledger`,
lifetime `totals`, and `history`. Each `RestGrazerReading` contains stable `id`,
`fatigue_points`, `activity`, `last_rest_tick`, and `last_transition_tick`.
An absent timestamp means that action/transition has never occurred in this run.
The counters share `RestCounters`: `effort_points`, `recovered_points`,
`rest_actions`, `entered_rest`, and `woke`, all `u64` and checked on addition.

The separate bounded journal records `EnteredRest`, `Woke`, `TravelEffort`, and
`Rested`, with run/tick and stable actor IDs. `Rested` records actual recovery,
fatigue after the action, and remaining commitment. Its coverage follows the
existing conservative whole-tick convention. `rest_actions_between(first,last)`
returns `None` for invalid/future/incompletely retained intervals, including a
partially evicted tick; `Some(0)` requires complete evidence. A lifetime total can
remain known after causal events are lost. There is no second biological copy in
this report, journal, or future browser adapter.

| File | B responsibility |
| --- | --- |
| `src/mobile/rest.rs` | Policy, fatigue/activity state, checked counters, and owned projection. |
| `src/mobile/rest/systems.rs` | Pure activity decision, ordered adapter, and guarded execution. |
| `src/mobile/rest/history.rs` | Supplemental outcomes and explicit retained coverage. |
| `src/mobile/movement.rs` | Whole-step effort bound and accepted effort accounting. |
| `src/mobile/perception.rs`, `src/feeding.rs`, `src/reproduction.rs` | Rest eligibility at intention and actual action boundaries. |
| `tests/rest.rs` | Public cumulative behavior, accounting, reset, and history cases. |
| `src/simulation/rest_tests.rs` | Late newborn/effect calls, stale permissions, and preflight failures. |
| `examples/resting.rs` | Declared journey and matched 120-tick meadow observations. |

### Observed journeys and controlled comparison

`resting_journey()` starts grazer 1 at `(0,0)` with reserve 20/capacity 24,
upkeep 1/meal 2/sight 5/speed 1/travel cost 1. Patch 100 at `(5,0)` holds 20,
capacity 20, no renewal. The grid is `6 × 2`; the population configuration is the
single founder from `short_journey`; default rest is enabled. All fatigue begins
at zero. The actual native trace is:

| Tick | x | Reserve | Fatigue | Activity after tick | Observed tick |
| --- | --- | --- | --- | --- | --- |
| 1 | 1 | 18 | 2 | Foraging | 1 |
| 2 | 2 | 16 | 4 | Foraging | 2 |
| 3 | 3 | 14 | 6 | Foraging | 3 |
| 4 | 3 | 13 | 4 | Resting, 1 left | 3 |
| 5 | 3 | 12 | 2 | Resting, 0 left | 3 |
| 6 | 4 | 10 | 4 | Foraging | 6 |
| 7 | 5 | 10 | 6 | Foraging | 7 |
| 8 | 5 | 9 | 4 | Resting, 1 left | 7 |
| 9 | 5 | 8 | 2 | Resting, 0 left | 7 |
| 10 | 5 | 9 | 2 | Foraging | 10 |

With recovery 10 instead of 2, tick 4 already reaches fatigue zero, yet tick 5
still executes a rest action recovering zero points; waking occurs at tick 6.
With minimum 3 instead of 2 and default recovery, tick 6 instead ends at x3,
reserve 11, fatigue zero, resting with zero commitment left. Its first meal is at
tick 8 instead of tick 7. These differences were executed and asserted.

The original design's proposed minimum 1 versus 2 does not distinguish waking
under the default 6→4→2 recovery path: both need two actions for the exit
threshold. The comparison used here changes **only minimum 2 versus 3**. It
uses the full `MobileScenario::meadow()` specification in checkpoint A above,
default rest in every other field, and exactly 120 ticks. All versions, IDs,
initial stores, growth, inheritance, newborn motion, and capacities match.

| Minimum | Tick | Living | Births / starvations | Travel cells | Rest actions | Reserve / patch stores |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | 8 | 3 | 3 / 4 | 15 | 7 | 31 / 18 |
| 3 | 8 | 4 | 3 / 3 | 14 | 8 | 32 / 18 |
| 2 | 12 | 3 | 4 / 5 | 19 | 12 | 28 / 34 |
| 3 | 12 | 3 | 4 / 5 | 18 | 11 | 37 / 22 |
| 2 | 40 | 0 | 7 / 11 | 36 | 21 | 0 / 36 |
| 3 | 40 | 2 | 7 / 9 | 46 | 33 | 26 / 18 |
| 2 | 120 | 0 | 7 / 11 | 36 | 21 | 0 / 36 |
| 3 | 120 | 1 | 10 / 13 | 71 | 53 | 24 / 31 |

Minimum 2 accepted growth 98, upkeep 108, travel cost 36, and birth dissipation
14: initial stores 96 + growth 98 = final stores 36 + those costs. Minimum 3
accepted growth 308, upkeep 258, travel cost 71, and birth dissipation 20:
96 + 308 = 55 + 258 + 71 + 20. The single final survivor is founder 1. Effort /
recovered totals are 72/42 versus 142/106; fatigue is a separate store and departed
actors are no longer in the current fatigue census. Both runs pass the energy
and population census equations after every tick. This outcome does not claim
that longer rest generally improves survival; it shows why the actual coupled
run is needed. Early store timing and later turnover differ, despite matching
initial conditions.

At tick 120 with minimum 2, base coverage is `(2,120]` with 11 evictions;
population, mobile, and rest journals retain `(0,120]` with none. With minimum 3,
base coverage is `(68,120]` with 296 evictions, population `(0,120]` with none,
mobile `(6,120]` with 28, and rest `(10,120]` with 31. Neither complete endpoints
nor known lifetime totals restore missing early causal events.

### B verification and limits

The final pinned native suite passed **75 tests**: 49 byte-identical public cases
from earlier checkpoints, 12 new public rest cases, and 14 internal tests. The
new internal cases deliberately call decision/effect phases late, inject stale
resting permissions, and exhaust an effort counter before movement. The two exact
public investigation tests decoded from `learning/content/resting.html` also
passed in a separate copied project and independent Cargo target directory.

Seven isolated defects compiled and then failed their intended assertions:
ignoring remaining commitment, allowing a newborn's early rest effect, recovering
twice in one tick, reducing commitment during policy evaluation, accepting travel
beyond available fatigue, allowing a resting meal, and allowing resting parents
to reproduce. Each mutant had its own target directory; compilation failures were
not counted as behavioral evidence. The source and outputs are retained under
`learning/work/ecosystem-rest-probe-*`.

Commands executed from the `learning/ecosystem` package were:

```sh
cargo +1.93.1 fmt --all -- --check
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 test --offline --locked
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 clippy --offline --locked --all-targets -- -D warnings
CARGO_TARGET_DIR=../work/ecosystem-target cargo +1.93.1 run --offline --locked --example resting
```

Formatting and strict Clippy passed. The earlier acceptance/population SHA-256
values above are unchanged; `tests/mobile.rs` remains byte-identical to checkpoint
A with SHA-256 `99e59a86b5df7558e9517fe342fd49d7c0c97f28f1077242ea1493270d5971e6`.
B adds no dependency or calibrated biological claim. Native tests, the scalar
WASM adapter, and actual browser behavior remain distinct evidence. Predation is
not implemented here, and this checkpoint does not complete the full learning
goal.

## Checkpoint C: local hunting in the same world

The opt-in hunter policy adds finite, mortal, nonreproducing hunters through
`MobileScenario::with_hunters(HuntConfig)`. It preserves all prior public seeds,
reports, ledgers and event variants. `CourseWorld::hunt_snapshot()` supplies
hunter observations, activity, actual costs/transfers, cause-specific outcomes
and bounded history. The older projections remain grazer/patch views; C's closed
budget and census require the supplemental report.

The private shared `Body` now holds reserve/capacity, upkeep, first eligibility
and one terminal cause. Grazer diet/contact and hunter attack settings are
separate components. Both roles use the same bounded travel and rest calculations
in one world. Capture preflights a whole transfer, pays an explicit attack cost,
marks the prey terminal immediately, and prevents a later meal or parent claim.
Stable-ID order determines contention. Hunters cannot obtain a new destination
from the global inspector or switch prey inside a failed action.

[Native hunting contract and observations](../design/hunting-native-notes.md)
records the exact API, modules, phase ordering, literal two-hunter fixture,
actual moving-prey counterexample, four matched 120-tick experiments, per-tick
budgets/censuses, and history coverage. Run the complete declared experiments:

```sh
CARGO_TARGET_DIR=learning/work/ecosystem-target cargo +1.93.1 run --offline --locked --manifest-path learning/ecosystem/Cargo.toml --example hunting
```

The hunter-present pilot ends with no animals in both minimum-rest variants;
those observed extinctions were retained rather than retuned. A separate
mechanism test confirms births, descendant action, maturation, both roles' rest,
and capture all occurred in the same run. These are authored game models, not
calibrated ecology or evidence that a particular policy is generally better.

Final C native verification passed **94 tests**: the unchanged 61 prior public
cases, 15 new top-level hunting cases, and 18 internal cases. Pinned formatting
and strict all-target Clippy passed. Two exact learner-authored HTML investigation
tests passed in a separate copied project. Every four-mode pre-C projection
matched frozen B byte-for-byte over ticks 0–120 and reset, as detailed in the
native notes.

Eight isolated mutants compiled, then failed their intended behavioral checks:
delayed terminal marking, partial prey credit, borrowing attack cost, remote
capture, capturing a newborn early, overflowing available capture effort, a second
accepted capture in one tick, and steering from the prey's changed global cell.
Each copy used a private Cargo target. The successful-capture guard test initially
had a capacity condition masking the defect; increasing only that private
fixture's capacity made the intended distinction observable. No production
behavior was altered to accommodate the probe.

The earlier three SHA values remain unchanged. `tests/rest.rs` is also
byte-identical to frozen B, SHA-256
`61a282ec9a8747b988f8895427b2db3af04ba65380c9814d8a55518a5479dac2`.
The final commands were the same pinned `fmt --check`, `test --offline --locked`,
and strict all-target `clippy` listed for B, plus `--example hunting`. Native
verification does not substitute for the separately owned WASM or browser checks.
The full learning goal remains active.

## Refuge capstone: protection changes eligibility

The optional `MobileScenario::with_refuges(RefugeConfig)` adds authored static
sites to the same world. `RefugeId` is a positive, separate site identity; it does
not consume an actor ID. Construction and replacement validate unique IDs,
unique in-bounds cells, then canonicalize site order before installation. A
failed reset preserves the entire old run and its saved configuration.

A private typed system records current grazer membership before local perception
and after travel. Protected prey is absent from eligible hunter observations;
current protection also rejects a previously observed same-cell capture before
any debit. A newborn receives membership at its accepted birth cell while
keeping its next-tick action deadline. Refuges change no reserve, recovery,
foraging target, birth rule, or starvation rule. There is no refuge-seeking or
fleeing behavior.

`CourseWorld::refuge_snapshot()` returns an owned, sorted current-membership
report, including run, tick, canonical sites, and each living grazer's current
cell, optional site and assessment tick. Disabled is `None`; enabled with zero
sites is a present empty configuration. This report is not a history and cannot
supply missing past evidence. All earlier public reports retain their meaning.

[Native refuge contract and observations](../design/refuge-native-notes.md)
records the complete API, source map, phase choices, literal arrival/departure
investigation, four matched placements, accounting and coverage. Run the native
comparison from the repository root:

```sh
CARGO_TARGET_DIR=learning/work/ecosystem-target cargo +1.93.1 run --offline --locked --manifest-path learning/ecosystem/Cargo.toml --example refuges
```

All four runs begin with identical actors, positions and 128 stored units and
continue for 120 ticks. Disabled and transit-cell protection end with no animals;
protection at the first food cell leaves one grazer. Protecting both food cells
produces the same measured outcome as protecting the first in this fixture.
Food-cell protection reduces captures from three to one while grazer starvation
increases from four to five. These results describe the declared model and
placement, not a general survival guarantee. Every tick closes the full energy
budget and grazer census; accepted growth may differ because feeding creates
capacity headroom.

The native suite passes **106 tests**, including twelve new public refuge cases;
the 76 prior public cases remain byte-identical. Two exact guide investigation
tests pass separately. Four compiled defects fail their intended assertions:
missing observation filtering, missing capture revalidation, missing post-travel
refresh, and missing newborn membership. The pre-perception refresh is an
explicit reassessment boundary but is redundant in current normal static runs;
its removal alone is not claimed as independently detected.

Five prior modes match frozen hunting C across all old public reports for ticks
0–120 and reset ticks 0/1: **19,872,248 identical bytes**, SHA-256
`da1586195115d6a55be8d7032742ca19441cae1fa5aa35e2f6651be7dce0746f`.
Formatting, the pinned offline suite, and strict all-target Clippy are the same
required native checks as C. This capstone has no new browser mode; native,
learner-workspace, and course presentation checks remain distinct.
