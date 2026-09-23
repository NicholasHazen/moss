# Sources that earn their place in the Moss book

**First editorial source map · verified September 23, 2026.**

These twelve reading stops support particular explanations; they are not a
prerequisite reading list. Put a link beside the moment it helps, with the short
invitation supplied below. Keep the chapter understandable without leaving it.

**Evidence boundary:** “Verified” describes a fact checked in the linked primary
source. “Use in Moss” and the visual ideas are editorial recommendations. None
of these sources verifies a Moss build, browser behavior, benchmark, biological
model or learning outcome. The project currently pins Rust **1.93.1**, edition
**2024**, and Bevy/Bevy ECS **0.18.1**; current upstream documentation can be newer.

No third-party passage, code sample, diagram or image was imported in this
research. “Reusable code” below means that an identified license supplies a
route for reuse with its conditions, not that every asset on that website shares
the code license. The recommended edition uses original Moss examples and
figures throughout. Link-only selections need no borrowed artwork.

## Placement at a glance

The chapter names here describe subjects, not fixed new filenames. Existing
guide links identify the relevant learning checkpoint without activating it.

| ID | Put it beside this question | Existing checkpoint |
| --- | --- | --- |
| S1 | What actually moves, copies or gets borrowed? | [Affordable movement](../../docs/tutorial/today-v2.md), [finite meal](../../docs/tutorial/path/01-bounded-meal.md) |
| S2 | How can a search return no result, and what settles a tie? | [Finding food](../../docs/tutorial/path/03-finding-food.md), [owned defaults](../../docs/tutorial/path/06-owned-costs.md) |
| S3 | Which entities does this query reach, and what may it mutate? | [Querying costs](../../docs/tutorial/path/07-querying-costs.md) |
| S4 | When does another system see a change? | [Growth before meals](../../docs/tutorial/path/10-growth-meets-meals.md), [removal and memory](../../docs/tutorial/path/14-removal-and-memory.md) |
| S5 | Why can the world advance steadily while pictures arrive irregularly? | [A tick through Moss](../../docs/tutorial/context/a-tick-through-moss.md) |
| S6 | What does a browser do while its tab is hidden? | [Return to an explainable run](../../docs/tutorial/path/16-return-and-explain.md) |
| S7 | Is this number a reserve, a rate or a completed transfer? | [A shared meal](../../docs/tutorial/path/02-a-shared-meal.md), [scarcity accounting](../../docs/tutorial/path/15-read-a-scarcity-run.md) |
| S8 | What belongs in a repeatable experiment record? | [A fair comparison](../../docs/tutorial/path/08-a-fair-comparison.md) |
| S9 | How much can one explicit food assumption change an ecosystem? | [Growth](../../docs/tutorial/path/09-growing-grass.md), [ecology context](../../docs/tutorial/context/from-meals-to-ecosystems.md) |
| S10 | Does changing a template change existing individuals? | [Attributes and defaults](../../docs/tutorial/context/attributes-and-defaults.md) |
| S11 | What is the difference between a baseline, an effective cost and energy left? | [Individual costs](../../docs/tutorial/path/06-owned-costs.md) and a later temporary-effects sidebar |
| S12 | What evidence would justify changing a data layout? | A later measurement sidebar following [individual-cost queries](../../docs/tutorial/path/07-querying-costs.md) |

## S1 · The Rust Book: ownership, then references

**Read:** [What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
and [References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).

**Verified:** The chapters distinguish moving a value from copying a `Copy`
value, then explain references, mutable access and the last-use boundary of a
borrow. Their examples make ownership a property of values, not a claim that
every function argument is a fresh copy.

**Use in Moss:** Place the borrowing chapter beside the first helper that changes
the caller's energy and biomass. Return to the ownership chapter when a proposed
position is copied or a template initializes an owned component. Draw Fern's
same reserve before and after a call; do not draw a duplicate ECS world.

**Reader invitation:** “Open this when `&mut` feels like punctuation: follow one
caller-owned value through a function, then return to the meal.”

**Version caveat:** These are living book pages. Their concepts apply to the
pinned edition; independently compile any Moss example on Rust 1.93.1.

**Reuse:** Code/text have a permissive route through the book's
[MIT license](https://github.com/rust-lang/book/blob/main/LICENSE-MIT), with its
notice conditions. Use original examples and diagrams here; import no graphics.

## S2 · Standard-library microreferences: `Iterator` and `Option`

**Read:** [`Iterator::min_by_key`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.min_by_key)
and [`Option::unwrap_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or).

**Verified:** `min_by_key` returns `None` for an empty iterator and selects the
first among equal minima. `unwrap_or` supplies a value only for `None`; its
argument is evaluated eagerly.

**Use in Moss:** After tracing an ordinary loop, show why a food search returns
`Option` and why an explicit stable-ID tie-break matters when input order is not
the intended policy. Reuse `Option` for defaults: `Some(0)` is an explicit zero,
not a missing override. A candidate strip can show filtered candidates, the
comparison key and the possible empty result.

**Reader invitation:** “Open this when a short iterator expression hides which
item wins—or when zero and ‘not supplied’ must mean different things.”

**Version caveat:** The fetched standard-library pages identify Rust 1.98.1.
The linked methods are documented as stable since 1.6.0 and 1.0.0 respectively;
do not borrow newer neighboring APIs without checking the pinned compiler.

**Reuse:** Prefer link-only. Rust's
[copyright notice](https://github.com/rust-lang/rust/blob/main/COPYRIGHT)
describes MIT/Apache licensing and file-specific exceptions if code reuse is
later needed.

## S3 · Bevy 0.18.1: a query is an access declaration

**Read:** [`Query`, particularly component access, optional access and disjoint queries](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html).

**Verified:** Query types select component data and declare access. Optional
components broaden matching; incompatible mutable access in two system queries
can panic during initialization. `Without` filters can make the sets disjoint;
`ParamSet` is another documented approach.

**Use in Moss:** Introduce the query as a sentence about the animals a system can
reach. A component-presence grid should include an intentionally incomplete
animal: requiring its cost component excludes it. This makes the spawn invariant
visible. Explain `&Energy` versus `&mut Energy` before adding generic syntax.

**Reader invitation:** “Open this when an animal is missing from your loop, or
when two individually sensible queries cannot live in one system.”

**Version caveat:** This URL is pinned to 0.18.1. Documentation of query costs is
not a Moss performance measurement; optional access is not universally slow.

**Reuse:** Bevy source examples have a route under the release's
[MIT license](https://github.com/bevyengine/bevy/blob/v0.18.1/LICENSE-MIT).
Use a smaller original Moss example and retain the link as API authority.

## S4 · Bevy 0.18.1: ordering and deferred changes

**Read:** [`IntoScheduleConfigs::chain`](https://docs.rs/bevy/0.18.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html#method.chain)
and [`Commands`](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Commands.html).

**Verified:** `chain()` orders successive systems and inserts `ApplyDeferred`
on an edge when its preceding node has deferred parameters. `Commands` queues
structural world changes for deferred application. `chain_ignore_deferred()`
specifically omits those added application points.

**Use in Moss:** Label actual mutation and command-application points in a tick
diagram. Do not say every queued change waits until the tick ends. Follow this
with the separate model decision: explicit claims prevent two rewards for one
prey, and eligibility determines a newborn's first action. Queue timing alone
does not define either biological policy.

**Reader invitation:** “Open this when two correct systems produce the wrong
story because one sees the world before—or after—a change.”

**Version caveat:** Both links are 0.18.1. The first uses Bevy's re-export because
that exact documentation page was retrievable; it is not a recommendation to
add full Bevy to `moss-sim`.

**Reuse:** Same [MIT route](https://github.com/bevyengine/bevy/blob/v0.18.1/LICENSE-MIT)
as S3; write an original schedule trace.

## S5 · Glenn Fiedler: fixed time and the accumulator

**Read:** [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/)
by Glenn Fiedler, June 10, 2004.

**Verified:** The article contrasts variable and fixed simulation steps, explains
an accumulator that separates simulation and rendering rates, and describes how
catch-up work can exceed the time available to execute it.

**Use in Moss:** Supply the conceptual background after the reader already knows
what one Moss tick changes. Draw two original lanes: irregular browser frames
and whole executed ticks. Separate a completed simulation step from the picture
that happens to show it. Keep Moss's hidden-tab policy explicit.

**Reader invitation:** “Open this when you wonder how smooth pictures and
repeatable simulation steps can run at different rates.”

**Version caveat:** This is an original physics-programming essay from 2004,
not Bevy/browser documentation. Its continuous-motion examples do not select
Moss's tick rate, permit hidden-tab catch-up, or prove cross-platform determinism.

**Reuse:** **Link-only.** No reusable license for the article's diagrams or code
was established in this pass. The book needs only its explanation as a reading
destination and an independently drawn Moss timeline.

## S6 · MDN: the hidden browser tab

**Read:** [Page Visibility API](https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API).

**Verified:** The page describes visibility notifications, throttled background
timers and browsers commonly ceasing animation callbacks for hidden tabs.

**Use in Moss:** Put a short browser sidebar beside pause/step/repeatability.
The browser reports visibility; Moss chooses to pause and discard backlog.
An original timeline can show a large wall-clock gap and no corresponding
executed simulation ticks. It should not imply a browser guarantees Moss's policy.

**Reader invitation:** “Open this when returning to a background tab raises the
question: did the world keep living while I was away?”

**Version caveat:** This is living browser documentation, checked on the date
above. Browser behavior and exact throttling thresholds vary; the book should
cite the general constraint and separately record observed browser behavior.

**Reuse:** **Link-only for this edition.** MDN's
[reuse policy](https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Attrib_copyright_license)
provides CC BY-SA documentation terms and separate, date-dependent code terms;
do not treat the entire page, logos or site design as CC0.

## S7 · MIT System Dynamics Group: stocks, flows and units

**Read:** [Mapping the Stock and Flow Structure of Systems, assignment 3](https://ocw.mit.edu/courses/15-871-introduction-to-system-dynamics-fall-2013/resources/mit15_871f13_ass3/)
([direct PDF](https://ocw.mit.edu/courses/15-871-introduction-to-system-dynamics-fall-2013/b950bd7b4f565b7b19145fb16bd55c8e_MIT15_871F13_ass3.pdf)), September 2013.

**Verified:** Sections A–B distinguish stocks from flows, ask for units, and
separate material transfers from information links. They explicitly ask where
the modeled sources and sinks end.

**Use in Moss:** Borrow the questions, not the worksheet. Label energy remaining
in units, maintenance in units/tick, and a completed meal as an actual quantity.
Create an original tick ledger showing starting reserve plus accepted intake
minus paid costs. State conversion rules between biomass and energy instead of
adding unlike quantities or claiming a physically closed ecosystem.

**Reader invitation:** “Open this when a rate, a reserve and a reported outcome
have started looking like the same number.”

**Version caveat:** This is a 2013 systems-modeling resource, not a game API or
ecological validation. No assignment completion is required.

**Reuse:** **Link-only.** [OCW terms](https://ocw.mit.edu/pages/privacy-and-terms-of-use/)
state CC BY-NC-SA 4.0 with conditions. Use original Moss figures rather than
reprinting the worksheet or its diagrams.

## S8 · NetLogo: write down the experiment

**Read:** [BehaviorSpace manual](https://docs.netlogo.org/behaviorspace.html).

**Verified:** BehaviorSpace systematically varies model settings, records run
measurements and supports repetitions. The manual describes controlled random
seeds, world dimensions, stopping conditions and measurement choices.

**Use in Moss:** Place after the first two-hare comparison. Offer a compact
experiment card containing the rule/configuration, one changed parameter,
initial conditions, executed-tick horizon and measurements. If randomness later
arrives, record seeds and repeat across them. Use a small comparison before
introducing a broad parameter sweep.

**Reader invitation:** “Open this when a promising run makes you ask whether
the same setup usually behaves that way.”

**Version caveat:** The fetched manual is NetLogo 7.0.4 and documents features
introduced in 6.4. It is a methodological comparison; no NetLogo dependency,
export format or experiment runner is proposed for Moss by this citation.

**Reuse:** **Link-only for this edition.** The
[manual license](https://docs.netlogo.org/copyright.html)
is CC BY-SA 3.0; the application and individual models have different licenses.
Draw an original Moss experiment card rather than copying the UI.

## S9 · Uri Wilensky: compare an ecosystem's assumptions

**Read:** [Wolf Sheep Predation](https://ccl.northwestern.edu/netlogo/models/WolfSheepPredation),
NetLogo Models Library, Uri Wilensky (1997).

**Verified:** The model describes both a sheep/wolf version with implicit food
for sheep and a version with explicit grass consumption and regrowth. The page
links a NetLogo Web runner and invites comparisons of parameters and outcomes.

**Use in Moss:** An optional excursion after finite food becomes understandable:
inspect which rule supplies food before comparing population curves. Return to
Moss with one named modeling assumption to investigate, such as finite patches
versus an unlimited resource. Treat a collapse as an outcome to explain, not
automatic evidence of a programming bug.

**Reader invitation:** “Open this when you want to see how making the food supply
explicit can change the story of a small ecosystem.”

**Version caveat:** The model's citation year is 1997; the library page is a
maintained presentation. The Web runner was linked but not executed in this
research. Its rules, random behavior and stability claims do not validate Moss.

**Reuse:** **Link-only.** The page declares CC BY-NC-SA 3.0 for the model; do not
import its screenshots, code or artwork into this edition.

## S10 · Flecs: copying and inheriting are different contracts

**Read:** [Component Traits, “OnInstantiate”](https://www.flecs.dev/flecs/ComponentTraits.html).

**Verified:** The documentation distinguishes default `Override` behavior, which
gives an instance an owned copy, from `Inherit`, which can resolve a base
component until overridden. It also documents `DontInherit`.

**Use in Moss:** Ask what an existing hare should read when its species template
changes. An original two-panel diagram can contrast an owned initialized value
with a live shared lookup. The difference is observable behavior and ownership;
it is not merely an implementation-style preference. An explicit zero override
should survive both initialization and reset under the selected Moss contract.

**Reader invitation:** “Open this when ‘default’ sounds obvious until you ask
whether changing it should also change creatures already alive.”

**Version caveat:** This is a living Flecs page, not a pinned Bevy contract.
It supports a comparison of policies, not a claim that Bevy has the same
inheritance mechanism or that either policy is faster in Moss.

**Reuse:** **Link-only.** No documentation-asset license was established here;
do not infer diagram rights from the engine's source-code license. Write a small
original Rust example if the comparison earns one.

## S11 · Unreal 5.6: base and effective attributes

**Read:** [Gameplay Attributes and Attribute Sets](https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-attributes-and-attribute-sets-for-the-gameplay-ability-system-in-unreal-engine?application_version=5.6).

**Verified:** Epic's documentation distinguishes an attribute's base value from
its current value under active effects, and describes attribute-set hooks for
bounds and changes.

**Use in Moss:** Add only when a temporary condition has an explanatory purpose.
Draw three labeled values: individual baseline cost, effective cost under a
condition, and reserve remaining. The reserve is a stock; the costs are rates.
An original numerical trace can show a modifier ending without rewriting the
baseline. This does not require an ability-system framework.

**Reader invitation:** “Open this when a temporary penalty should change what
an animal pays without erasing what belongs to that individual.”

**Version caveat:** The URL selects Unreal Engine 5.6. These are that engine's
concepts and APIs, not selected Moss modifier rules. Composition, rounding and
expiration remain explicit design decisions when that feature is undertaken.

**Reuse:** **Link-only.** No permission to reuse the page's code or diagrams was
established; keep the book's Rust example independent of the C++ framework.

## S12 · Factorio: a plausible optimization can do nothing

**Read:** [Friday Facts #204 — Another day, another optimisation](https://www.factorio.com/blog/post/fff-204),
kovarex and Zulan, August 18, 2017, especially “Prefetching”.

**Verified:** The developers report that moving less frequently accessed entity
data into separate storage required broad code changes without a measurable
performance improvement. They then describe measured, workload-sensitive
prefetching experiments.

**Use in Moss:** Close an attribute-layout sidebar with a reason to measure the
actual workload. Propose a Moss comparison that preserves initial conditions,
executed ticks and outcomes, and separates simulation time from drawing time.
The test of correctness precedes a speed comparison. Do not transplant a native
CPU optimization into a browser chapter merely because the case study is vivid.

**Reader invitation:** “Open this when an elegant data-layout change seems so
obviously faster that measuring it feels unnecessary.”

**Version caveat:** This is a production report about particular 2017 Factorio
code, hardware and maps. It provides a measured counterexample, not a Bevy
benchmark or a browser population limit.

**Reuse:** **Link-only.** No reuse license for the article's charts or code was
established. Any book chart should use newly collected Moss measurements and
state their conditions.

## Integration recommendations

1. **Keep four links near the first coding arc:** S1 for borrowing, S2 for the
   search result, S3 for ECS access, and S4 for ordering. A short invitation is
   more useful than an undifferentiated “further reading” list.
2. **Use original visuals to explain consequences:** borrowed values (S1), a
   query's matching grid (S3), an ordered tick with actual deferred application
   points (S4), separate clock lanes (S5–S6), a reserve ledger (S7), and
   template-to-instance ownership (S10–S11).
3. **Make outside experiments optional:** S8 and S9 invite curiosity after the
   book's own example makes sense. Returning to Moss should produce one small
   question or comparison, not a new tool-installation assignment.
4. **Keep engine comparisons in sidebars:** S10–S12 explain tradeoffs without
   turning Moss into an engine survey or implying a framework migration.
5. **Keep source authority narrow:** the API docs establish contracts, a model
   demonstrates its own rules, and a production report establishes its reported
   experience. Moss acceptance and observation need Moss evidence.

## Research evidence and limits

- Read `AGENTS.md`, `PROJECT_BRIEF.md`, `NOW.md`, the helper contract, research
  index, learning-design research, attributes/energy research, tutorial overview
  and sixteen-session path overview. Inspected the existing worktree and pinned
  toolchain/manifests. No existing work was changed.
- All main URLs above were opened successfully through web research on September
  23, 2026. Method anchors for `min_by_key`, `unwrap_or` and `chain` were checked
  against the returned documentation. Copyright/license pages linked here were
  also inspected. No external application or interactive model was run.
- The research tool could not retrieve the attempted Rust 1.93.1/1.93.0 hosted
  documentation URLs, so S1–S2 explicitly use living pages with a version caveat.
  This is a retrieval limitation, not evidence that those hosted versions do
  not exist. The standard-library methods selected have older documented
  stabilization dates; book examples still need the project's own checks.
- The direct `bevy_ecs` 0.18.1 `IntoScheduleConfigs` page was not retrievable in
  this pass; its Bevy 0.18.1 re-export and versioned source were retrieved. The
  entry uses the working re-export URL without altering crate boundaries.
- The existing research's pinned Veloren energy URL could not be retrieved in
  this pass. It remains useful prior local research but is not presented here
  as newly web-verified evidence. S11 supplies a verified baseline/effective
  comparison without copying another game's implementation.
- A local Python structure check passed: entries S1–S12 are present in order,
  all 17 relative Markdown links resolve, and the file has no trailing
  whitespace. `rg` confirmed the manifest's exact Bevy pins; the toolchain
  file specifies Rust 1.93.1.
- Only this new editorial file was created. No native tests, WASM builds,
  browser smoke checks, benchmarks or learning evaluations were performed.

**Stopping point:** a first curated map is ready for the lead editor to integrate.
No source choice here assigns or implements a biological rule.
