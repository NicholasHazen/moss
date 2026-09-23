# Later-chapter editorial review — September 23, 2026

The later chapters explain their designs carefully and preserve the first
learner-owned movement edit. I found one concrete ambiguity about the installed
code, followed by three bounded teaching and navigation refinements. None
requires another feature, a changed rule, or a longer curriculum. Keep the
existing worked Rust blocks and heading destinations intact.

## 1. Put the owned-cost component explicitly in the future

**Priority: P2. Classification: factual-status ambiguity, worth correcting.**

Location: `book/src/chapters/04-two-hares-one-fair-comparison.md:434–438`,
“Checkpoint: both the right values and the right members.” The passage says:

> The future live integration uses the one component already prepared in
> `energy.rs`; it must not declare a competing second type.

The edition's `crates/moss-sim/src/energy.rs` contains `MovementRules`, `Energy`
and `SpeciesEnergyRules`; it does not contain `AnimalEnergyCosts`. “Already
prepared” can mean “prepared by the preceding future checkpoint,” but a reader
following the file reference can reasonably read it as a statement about the
downloaded starter. The chapter's opening correctly calls individual costs a
proposal; this local tense weakens that useful distinction.

Replace the two sentences with:

> Its local component repeats the preceding concept so the reference can run
> independently. When the owned-cost checkpoint is prepared for the live
> project, it will introduce one `AnimalEnergyCosts` component in `energy.rs`.
> The later maintenance checkpoint must reuse that type rather than declaring
> another component with the same name.

This preserves the important Rust/ECS lesson: identical field names do not make
two separately declared structs the same component type. It also names the
missing preparation without adding it to Nick's current assignment.

## 2. Complete the glossary's smallest ECS model

**Priority: P3. Classification: navigation and recall gap, not a technical error.**

Locations: `book/src/reference/concept-index.md:16–28` and
`book/src/reference/code-map.md:28–41`. The glossary defines component, entity,
query, resource and schedule, but omits **system** and **world**. Several existing
entries depend on those words. Chapter 1 explains a system well at lines
274–277, so a reader who lands on the index should be able to recover that
explanation without searching the full chapter.

Add only these two entries, linking the existing explanations rather than
creating another general ECS lesson:

| Word or phrase | Proposed meaning in the book |
| --- | --- |
| **System** | A function that declares the ECS data it accesses and can be run by a schedule. `move_to_food` is the prepared system adapter; `move_one_cell` is the ordinary helper it calls. See [the adapter](../src/chapters/01-one-affordable-step.md#how-the-little-function-reaches-an-ecs-world). |
| **World** | The ECS store holding entities, their components and shared resources. Moss's simulation and browser use one authoritative world; presentation reads it instead of keeping a second biological copy. See [the code map](../src/reference/code-map.md). |

When integrating these rows into `src/reference/concept-index.md`, use its
normal destinations: `../chapters/01-one-affordable-step.md#how-the-little-function-reaches-an-ecs-world`
and `code-map.md`.

This distinction matters to the first exercise: finishing a helper does not
register a system, and registering a system does not create its required data.
No further glossary inventory is needed for this pass.

## 3. Show the borrow boundary at the removal operation

**Priority: P3. Classification: optional teaching refinement.**

Location: `book/src/chapters/06-a-disappearance-explained.md:369–383`, following
the complete removal answer. The prose correctly explains `collect()`, the
explicit `*id` copy, and why retained component references would obstruct
structural mutation. This is a consequential new Rust idea, but the visual in
this chapter follows records and counts, not the change from borrowed query
data to owned candidates.

A small two-column trace here would give the existing explanation a visual
anchor without adding a new SVG, interactive model or code block:

| Phase | What the resolver holds |
| --- | --- |
| Inspect | Each query row exposes an entity handle and borrowed `SimId` and `Energy` components. Read the reserve while those borrows are available. |
| Keep candidates | Copy `(Entity, SimId)` into a vector. The vector holds handle and ID values; it contains no reference into component storage. |
| Commit removal | Query iteration has finished. Use the owned handles to despawn; report an ID only when removal succeeds. |

Use a short caption or transition:

> The list carries the information we need across the mutation boundary; it
> does not carry the animals or their component borrows with it.

Keep the paragraphs explaining why `collect()` alone does not copy borrowed
components. If the added table feels repetitive in the rendered chapter,
replace the paragraph at lines 369–372 with the trace rather than stacking both.
The goal is a visible change in the kind of data retained, not more coverage.

## 4. Give shape and plant spread their own destinations

**Priority: P3. Classification: discoverability preference tied to Nick's questions.**

Location: `book/src/chapters/07-another-kind-of-life.md:145–173`. Weather,
simulation geometry and cellular-automaton spread are three separate answers
under one heading, “Let the environment change a known relationship.” The last
two paragraphs already address Nick's explicit questions about sizes, shapes
and whether Meadow is a growing CA. Their placement makes them hard to find
again through chapter headings, the concept index or direct links.

Keep the existing weather heading and its anchor. Add two narrow subheadings:

- Before line 159: `### When a larger picture changes contact`
- Before line 167: `### Refill a patch or spread to a neighbor`

No new prose is necessary. For a direct return path, the sentence about
Meadow's unchanged footprint in chapter 5 (`05-a-world-that-feeds.md:33–37`)
can link “changing its footprint” to the first subsection, or the concept index
can add one compact **Footprint** entry linking there. Prefer one link, not both.

The refinement keeps the coda a horizon discussion. It should still finish by
returning to the prepared affordable-step edit, not by inviting another design
assignment.

## 5. Let the reader open dense figures at their natural size

**Priority: P3. Classification: rendering refinement from root's observation.**

Location: `book/src/chapters/06-a-disappearance-explained.md:481–485`, and
equivalent image/caption locations in other chapters. During this review,
root independently observed the 1000-pixel evidence diagram shrinking to about
307 pixels in a 375-pixel viewport. A figure can fit without overflow while
its labels become difficult to read. I did not reproduce that browser
observation in this source-only review.

Root's proposed direct links to the full-size SVGs are a proportionate fix.
For this figure, place the following beside its caption:

> [Open the full-size evidence diagram](../src/assets/a-life-and-its-evidence.svg).

Use `../assets/a-life-and-its-evidence.svg` when integrating into the chapter.
Use a similarly descriptive link for each figure instead of a generic “View
larger,” so the destination remains understandable out of context. Preserve
the existing alt text and explanatory caption; opening a separate diagram
should be optional. A custom zoom widget, new viewer dependency or complete
figure redesign would add more scope than this issue needs.

## What to preserve

- Chapter 4 follows an attribute from authored instruction to owned value,
  query access and a controlled outcome. Its `None`/`Some(0)`/`Some(1)` cases
  answer the real default/override concern, and the provenance discussion
  explains what cannot be reconstructed from the final number.
- The same chapter explicitly separates ownership, component access boundaries
  and performance claims. It does not sell one representation as universally
  scalable. The baseline/effective/current example is a useful answer to the
  RPG-stat question without introducing a speculative framework.
- Chapter 5 gives unchanged stores two different explanations. Following
  Meadow from daylight through night makes actual outcomes, stock and flow
  useful concepts rather than vocabulary introduced in advance.
- Chapter 6 exposes a subtle consequence of saturation: a requested cost of
  two need not mean two units are paid or required to survive. That is a strong
  design review embedded in the lesson. Preserve the concrete trace and its
  explicit policy boundary.
- “First retained” versus “complete since,” independently sampled living
  counts, and selected versus complete snapshots are precise and practical.
  The limits explain what evidence can support rather than serving as generic
  disclaimers.
- The code map begins with a behavior to investigate and separates helper,
  installed-world and browser evidence. The final coda returns to one named
  first edit. Both support resuming work without recreating a plan.

## Review scope and limits

Read the current project instructions, brief, handoff, subagent contract and
tutorial authoring contract. Applied TutorBro's skill and prose guide using
the stable-example (S02) and executable-increment (S05) techniques. Read all
teaching prose, example framing, traces, headings and visual captions in
chapters 4–7, plus the index, code map, concept index and edition notes. Inspected
the relevant `energy.rs` and `simulation.rs` definitions and the first chapter's
system explanation. Read the prior editorial report to avoid repeating already
integrated suggestions.

This was an editorial/source review. I did not execute Rust, JavaScript or
publishing tests, operate a browser, recheck external references, benchmark
anything, or inspect the final rendering of proposed changes. The previous
edition's verification remains previous evidence. This review found no new
algorithm defect; it was not an exhaustive algorithm audit. Only this report
was written. Root owns integration and validation of any accepted refinement.

## Integration review

Read root's revised chapter 4 status paragraph, System/World glossary entries,
chapter 6 removal table, chapter 7 subsection destinations, chapter 5 footprint
link and all ten descriptive full-size figure links. The reported findings are
addressed. The revisions preserve the installed/future distinction, existing
first edit and coda's return path. This re-read did not repeat runtime or browser
verification; root is handling the build and rendering checks.

One small precision edit was returned to root: the removal table says the
mapping copies its tuple “into a vector,” although `map` yields the tuple and
`collect` builds the vector. Suggested text: “The mapping yields owned
`(Entity, SimId)` pairs; `collect()` gathers them into a vector with no references
into component storage.” The following prose already explains that distinction
correctly. No other actionable correction emerged from this integration review.
