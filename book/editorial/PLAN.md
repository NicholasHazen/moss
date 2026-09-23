# Moss textbook: editorial blueprint

**Working title:** *Moss: Small Rules, Living Worlds*

**Edition:** a separate textbook interpretation of the existing guides

**Editorial baseline:** September 23, 2026; Rust 1.93.1, edition 2024, Bevy ECS 0.18.1

**Status:** first editorial proposal, not a new implementation sequence

Build the book around six sustained investigations. Each begins with something
Fern or Meadow does, follows the state responsible, and develops a reusable
programming idea through a worked example. Preserve the existing short-session
checkpoints inside these chapters, where they offer places to stop. Sixteen
session files need not become sixteen top-level textbook chapters.

The book should reward both reading and doing. A reader can follow a complete
explanation without editing; a returning programmer can move directly from the
opening to the current function. Depth comes from causal explanation, precise
contrasts, useful diagrams and experiments—not a larger preliminary syllabus.

## The promise to the reader

The reader already knows how to program and is returning to Rust and ECS. Teach
the Rust mechanism that a particular decision needs, then revisit it in another
consequential setting. Do not begin with variables, a survey of ECS history, or
an inventory of future components.

By the end of the main narrative, the reader should be able to trace an action
from its inputs to the changes it commits; explain which entities a query can
reach; choose a test that separates plausible explanations; and investigate a
small population without confusing a working rule with a balanced ecosystem.
These are editorial aims, not claims of measured learning outcomes.

Keep Fern, Flint and Meadow as the familiar cast. Fern supplies the continuing
story; Flint's unused hunter role is a reason for a later question, not evidence
of existing predation. Use a second hare when the argument needs a comparison,
and say exactly which conditions differ.

## The first screen must lead to the current contribution

Open with a short view of the world and a visible **Start with one affordable
step** link. Put the editable symbol, exact checkpoint and complete worked answer
in Chapter 1. A reader should reach that edit without completing an introductory
chapter or opening the reference shelf.

The opening also needs one compact statement of the live boundary:

> Maintenance runs. Movement has its inputs, adapter and tests prepared, but
> `move_one_cell` still contains `todo!()` and its adapter is not scheduled.
> Your first contribution is that one function. A passing helper is the first
> stopping point; reviewed activation connects it to the browser.

Link to [the current task](../../NOW.md) and
[the existing v2 lesson](../../docs/tutorial/today-v2.md). The textbook must not
become another source assigning today's work. Treat its baseline as an edition
snapshot and `NOW.md` as the live handoff.

Offer a brief **How to use this book** passage beside the contents: read the
chapter, follow a trace, use as much of the answer as helps, and stop at its named
checkpoint. Reading ahead explores future designs; it does not install them.
Put installation and IDE setup in a compact reference destination linked to the
[development guide](../../docs/development/README.md), outside the narrative's
first movement.

## The chapter architecture

### 1. One step that Fern can afford

**Opening question:** Fern knows where Meadow is; what must be true before we
move her? Begin with the `(2, 2)` to `(4, 4)` helper case and reserves 59 and 1.
The same proposed cell distinguishes an accepted action from a rejected one.

Develop the chapter in four connected movements: propose a value; validate and
commit; let an ECS query supply the real components; follow the resulting tick
into the inspector. Introduce `Copy`, `&mut`, dereferencing and return values
where each changes the reader's account of that action. Explain x-before-y as a
chosen movement rule, not a Rust restriction. Compare a rejected unaffordable
step with maintenance's deliberate saturation at zero.

The current test command and complete replacement belong here. Preserve the
first stop after the named helper passes. The subsequent integration walkthrough
must label the nine-step journey and tenth stationary tick as expected results
after activation: reserve 33 at arrival, then 32. No browser screenshot should
depict those as an observed live outcome unless the relevant build was run.

Use a short optional sidebar, **Why can a correct function do nothing?**, for
missing target, rejected proposal and an unscheduled adapter. This turns the
code map into a diagnostic tool before expanding into general architecture.

**Reusable idea:** construct a tentative change and commit only an accepted
outcome. The meal and growth chapters will return to this pattern.

### 2. A meal has two sides

**Opening question:** reaching Meadow changed Fern's position; why did neither
her reserve nor the patch change? Give eating its own finite transfer.

Start with capacity, supply and bite limit as three independent bounds. Name
biomass and energy separately even under the proposed 1:1 conversion. Follow
both mutable values through one meal, then introduce the second eater only when
the first transfer is understood. The new problem is who sees the remainder.

Show stable-ID resolution with contenders supplied in reverse order. Explain
the difference between legal access and the application's choice of winner:
borrowing protects access; it does not decide resource allocation. Follow actual
consumption into an outcome record, and contrast a meal of 4 with a net reserve
gain of 1 after maintenance and travel. Avoid an unexplained leap from an isolated
helper to a complete live eating system.

Keep two stopping points within one chapter: the bounded helper, then shared-food
integration. The existing movement stretch and path sessions 01–02 are alternate
entry points into those same outcomes, not work to repeat.

**Reusable idea:** account for the actual transfer, and make competition explicit.

### 3. Let the animal choose

**Opening question:** what changes when the fixture stops choosing Fern's meal?
Reuse the movement and eating rules; change where their instruction comes from.

Introduce the borrowed candidate slice and `Option` through a patch that is
absent, empty or out of range. Make nearest-food selection inspectable with
deliberately different distances, then an exact tie. A small diagonal case
should explain why the chosen distance metric matters. Keep the metric used by
the reference rather than substituting one that merely looks nicer on a map.

Then give the animal one remembered activity. A two-threshold Idle/Seeking
transition explains why an enum can carry useful memory without a planner.
Distinguish activity from a current target: Seeking can persist even when no
eligible patch is available. Trace when choice creates or removes the target
and when movement and eating see that result.

Close with the first proposed autonomous foraging loop and one investigation:
an animal can finish a meal above its stop threshold while still Seeking until
the next decision. One more tick is a useful observation. A state machine diagram
alone cannot explain that schedule-dependent delay.

**Reusable idea:** decision, retained state and accepted action have different
jobs. A visible mismatch may come from timing rather than a wrong threshold.

### 4. Two hares, one fair comparison

**Opening question:** what can a population tell us that Fern cannot?
Begin with a small authored group and show how equal means can hide different
individual reserves. Summaries lead back to the individuals and stable IDs.

Now let two hares have different upkeep. Explain a species default, an authored
override and the owned runtime value through construction, not an abstract
attribute framework. `None`, `Some(0)` and an explicit value equal to the default
deserve distinct cases. An already constructed animal is not a live link to a
template. The inspector cannot infer origin from a resolved number alone.

Return to the familiar maintenance query with a new source for its rate. Use
component membership to expose an animal accidentally skipped because required
data is absent. Preserve a non-animal control so a missing `With<Creature>`
cannot pass unnoticed. This is the point for an optional deeper explanation of
archetypes and component access boundaries.

Finish with the controlled comparison: no travel isolates upkeep; equal travel
holds one cause constant while upkeep differs. State that the reference's
supplied travel trace does not itself prove movement executed. Treat storage
layout as a responsibility choice; reserve performance claims for measurement.

**Reusable idea:** who owns a value and what varies in an experiment determine
which conclusion its result supports.

### 5. A world that feeds its inhabitants

**Opening question:** a trustworthy finite meal eventually empties Meadow;
where does replacement food enter the account?

Return to the meal's bounds with growth at 99 of a capacity of 100. Separate a
request to grow from biomass actually retained. Explain a stored quantity and
its inflows/outflows before introducing stock-and-flow notation. A patch refill
does not require plant spread or a cellular automaton.

Compose growth with choice and eating, making the order a model decision. The
full-patch and empty-patch contrasts reveal different ordering mistakes. Then
derive daylight from executed ticks. Keep executing tick and completed tick
distinct; show the actual proposed 240-phase convention around 119/120 and
239/240, including the initial interval's shorter count of growth opportunities.

Night stops new light-driven production, not consumption of already stored
food. That sentence should accompany both the diagram and lab. Display tint
reads the environment; it does not supply its state. End with the same Meadow
across day, night and the next dawn, with an explicit biomass account.

**Reusable idea:** composition is more than putting correct helpers together;
the inputs and ordering determine which opportunity exists this tick.

### 6. A disappearance that the world can explain

**Opening question:** if a hare reaches zero, what does that mean in this model?
Recall that the live foundation leaves animals alive at zero. Present the
after-meal starvation boundary as a proposed policy, with a last-chance meal
and an empty night to make its consequence concrete.

Once that proposal is understood, separate deciding a terminal outcome from
structural removal. Follow a stable ID through collection, removal and retained
evidence. Explain short world borrows at the point where collecting owned
handles makes subsequent mutation possible. Bound the historical claim: the
journal can retain an outcome without retaining a complete life forever.

Use a scarcity run to connect deaths, living counts, stored food and supply.
First reconcile the account; then ask whether the scenario is sustainable or
interesting. Do not turn ecological equilibrium into the correctness test.
Finish by comparing canonical state after the same executed inputs, keeping
camera and render details outside that comparison. A snapshot example is not
proof of full replay, save/resume or browser repeatability.

**Reusable idea:** name when a rule applies, record what it actually did, and
keep the strength of the conclusion within the evidence available.

### Coda. What another kind of life would ask of the model

Use one compact illustrated outlook, not new unfinished implementation chapters.
Flint's first hunt reuses target, movement and exclusive resolution; a claimed
prey must stop being eligible before any later action. Explain why an end-of-tick
starvation rule does not solve that earlier conflict. A simpler in-range hunt
does not require pursuit, and neither form requires sunlight.

Follow with three short connected questions: how can rest reduce fatigue without
creating nutrition; how can a funded birth create a child that first acts next
tick; and what does inheritance initialize rather than copy from current state?
Weather, spatial structure and plant spread can appear as labeled future
applications in this same outlook. They are directions to reconsider after
watching the foraging loop, not extra prerequisites or promises of ready code.

## Source mapping and continuity

Existing guides provide tested examples and task-sized checkpoints. Rewrite
their transitions into a continuous narrative; do not concatenate their status
notices, duplicated setup or repeated commands. Preserve source links so readers
can return to the live project. Historical records establish evidence; they do
not supply the chapter order.

| Book destination | Primary material and editorial use |
| --- | --- |
| Opening and Chapter 1 | [Guide home](../../docs/tutorial/README.md), [v2](../../docs/tutorial/today-v2.md), [movement companion](../../docs/tutorial/04-movement.md), [one tick](../../docs/tutorial/context/a-tick-through-moss.md), [ECS](../../docs/tutorial/context/ecs-in-moss.md), [Rust at use](../../docs/tutorial/context/rust-at-point-of-use.md), [reading a test](../../docs/tutorial/context/reading-a-test.md). Keep movement first; fold the necessary context into its explanation. |
| Chapter 2 | [Meal](../../docs/tutorial/path/01-bounded-meal.md), [shared meal](../../docs/tutorial/path/02-a-shared-meal.md), [eating companion](../../docs/tutorial/05-eating.md). One narrative with two checkpoint stops. |
| Chapter 3 | [Selection](../../docs/tutorial/path/03-finding-food.md), [seeking](../../docs/tutorial/path/04-when-to-seek.md), [choice companion](../../docs/tutorial/03-food-choice.md). Explain the complete loop and its order. |
| Chapter 4 | [Population](../../docs/tutorial/path/05-many-individuals.md), [owned costs](../../docs/tutorial/path/06-owned-costs.md), [querying costs](../../docs/tutorial/path/07-querying-costs.md), [comparison](../../docs/tutorial/path/08-a-fair-comparison.md), [attributes context](../../docs/tutorial/context/attributes-and-defaults.md). Introduce representation through a fair experiment. |
| Chapter 5 | [Growth](../../docs/tutorial/path/09-growing-grass.md), [growth and meals](../../docs/tutorial/path/10-growth-meets-meals.md), [day](../../docs/tutorial/path/11-a-day-from-ticks.md), [light](../../docs/tutorial/path/12-light-controls-growth.md), [ecosystem context](../../docs/tutorial/context/from-meals-to-ecosystems.md). Build a supply account before environmental variability. |
| Chapter 6 | [Starvation](../../docs/tutorial/path/13-deciding-starvation.md), [removal](../../docs/tutorial/path/14-removal-and-memory.md), [scarcity](../../docs/tutorial/path/15-read-a-scarcity-run.md), [return and explain](../../docs/tutorial/path/16-return-and-explain.md), [observability](../../docs/design/observability.md). Keep policy, outcome and evidence distinct. |
| Coda and concept index | [Learning map](../../docs/tutorial/context/learning-path.md), [path outlook](../../docs/tutorial/path/README.md#what-comes-after-this-path), [ecology](../../docs/design/ecology.md), [architecture](../../docs/design/architecture.md). Select questions that reuse a mechanism already explained. |

## Illustrations and labs with a purpose

Use a restrained visual vocabulary: the same creature labels, coordinate marks,
tick direction and names for energy and biomass throughout. Label every quantity
with its unit. Use color alongside words, shapes or line patterns. Every visual
needs a caption that states what to inspect and a textual account of its result.

Prioritize these five interactive labs. Each must work as a small authored model
beside the prose, with a reset and an equivalent static trace. Mark it
**illustrative model** unless it actually executes the pinned Rust example.
Matching a JavaScript model's outputs to a few Rust cases is useful verification,
but does not make the lab a live simulation or establish general equivalence.

| Placement | Reader action, intended discovery and static fallback |
| --- | --- |
| Chapter 1: propose, inspect, commit | Adjust reserve across 1, 2 and 59; attempt the same one-cell step. Show proposed versus committed coordinates and charged distance. A three-case table supplies the same conclusion without interaction. |
| Chapter 2: the last bite | Change remaining biomass and reverse the displayed contender order. Resolve in the stated stable-ID order and show each actual meal plus remainder. Keep the distinction between input order and resolution order visible. A two-eater trace is the fallback. |
| Chapter 3: a tick under a microscope | Advance phase by phase through maintenance, choice, movement, eating and completion using one selected case. Show activity and target separately, especially just after eating. A timeline with before/after state provides the equivalent explanation. |
| Chapter 4: query membership | Toggle the component presence of a small synthetic entity and show which declared query matches. Keep a positive control. Explain that a missing required cost violates this model's construction rule, while an absent target can be meaningful. A component/query matrix is the fallback. |
| Chapter 5: daylight and stored food | Step through the boundary ticks and inspect light, requested growth, actual growth, meal and stored biomass separately. Include a night with edible stock. A boundary table and stock-flow figure replace the controls in print. |

Chapter 6 benefits more from a static evidence spread than another simulator:
one creature's retained outcome, the corresponding population-account change,
and an explicit history-coverage boundary. This directs attention to what is
known rather than implying that an attractive animation proves causation.

Adapt the existing movement, adapter, shared-food, seeking, default-to-instance,
stock-and-flow and tick-order diagrams where their teaching job remains clear.
Do not include every existing SVG by default. Add a small landscape or chapter
opening illustration only if it supports orientation or pacing; decorative art
must not be mistaken for a screenshot of implemented biology.

## Sidebars and reference apparatus

Keep essential reasoning in the reading column. Use optional sidebars for a
second depth of explanation, linked at the exact sentence where they become
useful. A sidebar should answer one concrete question and return to the example.

| Question worth a sidebar | First useful location |
| --- | --- |
| Why doesn't changing `next` move Fern? | Chapter 1's copied proposal and write-back. |
| Why does this compile but Bevy reject the queries? | Chapter 1's adapter, with the declared `With`/`Without` exclusion. |
| What does a green test establish? | Chapter 1's helper stop; revisit briefly at later integration boundaries. |
| Is a mean animal an actual animal? | Chapter 4's population summary and equal-mean contrast. |
| If the values match, why keep the authored override? | Chapter 4's default versus explicit input. |
| Is changing a species value changing an archetype? | Chapter 4's component membership discussion. |
| Why doesn't a dark picture stop growth? | Chapter 5's simulation-owned environment. |
| Can a seed or a snapshot reproduce a world? | Chapter 6's bounded repeatability claim. |

Provide a small concept index with links to the **first explanation** and **next
use** of borrowing, copying, `Option`, enums, queries, resources, ordering, stable
identity and evidence. This is more useful for returning after a gap than a
glossary that only expands vocabulary. A compact glossary can define the terms
that have precise Moss meanings: reserve, capacity, biomass, activity, target,
executed tick, completed tick, outcome and retained history.

At each chapter's end, offer at most a few optional references, each described
by the question it answers. The current [Rust borrowing chapter](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
supports the ordinary-reference discussion; describe it as a language reference,
not proof that a Moss example ran on the pinned compiler. The versioned
[Bevy ECS 0.18.1 query reference](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html)
supports access, required/optional data, filters and disjoint-query discussion.
Both destinations were opened during this editorial pass. Check any additional
external references before publication, and keep speculative comparative design
reading behind the local research notes rather than in the core route.

## Example and evidence policy for this edition

Use a small stable set of visible labels: **current source**, **worked reference**,
**expected after activation**, and **proposed policy**. Introduce their meaning
once, then place the relevant label beside code or output. Avoid repeating a
large disclaimer at the start and end of every section.

Every runnable block needs a named location, required context and one owning
source. Prefer extracting already verified blocks into the publication over
maintaining independent copies by hand. If the textbook revises code, its exact
printed block needs a fresh isolated check. Link to evidence that states what
actually executed and when; inherited verification is not a fresh run.

Keep the helper, installed schedule, WASM build and browser observations separate.
The [existing example runner](../../docs/tutorial/authoring/check_path_examples.py)
checks complete references in isolated workspaces. It does not establish that
the live learner body is complete. The documented navigation checker currently
covers root Markdown, `docs/` and `prompts/`; adding `book/` requires deliberate
coverage or a book-specific check. The publishing lead owns that implementation.

Do not move the existing guide's filenames or anchors. New textbook chapter
anchors can be stable independently, with links back to the original checkpoint.
Retain the source edition's current-state notice while resolving relative links
for the published output. Rendered code, local file links, citations, figures and
lab fallbacks all need inspection in the actual output format.

## A sample opening voice

The following is original draft prose for Chapter 1. Its journey is explicitly
prospective because movement is not active in the live project:

> Fern has enough energy to wait, but waiting is all she can do. Each Step takes
> one unit from her reserve. Meadow stands six cells to the right and three cells
> above her, holding food that no rule yet lets her reach or eat.
>
> We can change the first part with one function. The fixture already names
> Meadow as Fern's destination. Your rule will attempt one cell of travel, decide
> whether Fern can pay for it, and change her position and reserve together.
> Choosing food and eating it will have their own turns.
>
> Start with the smaller journey in the figure: `(2, 2)` toward `(4, 4)`. A step
> to `(3, 2)` costs two energy units. With 59 available, Fern can pay. With one,
> the proposed step must leave her exactly where she was. That rejected attempt
> is useful: it tells us why we should think about the change before writing it
> into the world.
>
> Open `move_one_cell` and run its prepared test. For now it stops at `todo!()`.
> The signature and caller are ready; the decision in the middle is yours. The
> complete answer is here when you want it, and one passing helper test is a
> good place to stop for review.

The final chapter opening should link the symbol and command directly, then
develop the proposed-value diagram. Do not add a page of motivational copy
between that invitation and the edit.

## Production priorities

1. **Prove the opening.** Complete Chapter 1 with its real source map, exact
   checkpoint, answer and two complementary visuals. Review whether a returning
   programmer can find the first edit and explain the rejected case. This sets
   the voice, evidence labels and page design for the rest.
2. **Complete the first living loop in prose.** Draft Chapters 2–3 with their
   bounded references and a coherent phase trace. This is the first substantial
   textbook unit, even while its biology remains future work.
3. **Develop the experimental middle.** Draft Chapters 4–5 around controlled
   comparisons and resource accounts. Add the labs only after their static
   cases and conclusions are clear.
4. **Close the account.** Write Chapter 6 and the coda, reconcile current/future
   labels, then produce the concept index and small reference shelf from the
   finished narrative.
5. **Review the publication as a book.** Check transitions, repetition,
   cross-references, rendered layout and the route back to the live checkpoint.
   Do a complete technical and accessibility pass before presenting it as an
   edition ready to read.

This is an editorial order for the authorized textbook work. It does not reorder
Nick's active task or authorize installing any proposed behavior.

## Iteration and review rubric

Use **pass**, **revise**, or **not applicable**, with a concrete location and
reason. Do not convert the rubric into a learner score or claim a polished page
has demonstrated learning effectiveness.

| Review lens | The question the reviewer must answer |
| --- | --- |
| Entry and continuity | Can the reader locate the current editable function immediately, and does each later idea answer a question the previous behavior raised? |
| Causal explanation | Can a reader follow one named value from input through acceptance/rejection to the changed state without guessing a missing step? |
| Rust and ECS precision | Are copying, borrowing, ownership, query membership and schedule order explained as separate mechanisms where appropriate, using the pinned APIs? |
| Discriminating evidence | Does the example rule out a named plausible mistake, or could that mistake still produce the advertised result? |
| Model boundaries | Are current behavior, proposed policy, illustrative calculations and actual observations visibly distinguishable at the point of use? |
| Visual purpose | Does every visual answer a spatial, temporal, state or quantity question, with legible units and a text equivalent? |
| Lab fidelity | Do boundaries, reset, keyboard operation and static fallback agree with the specified model; is any duplicated implementation labeled honestly? |
| Reading experience | Is the main argument connected prose, with optional depth available without a detour through several files? Are repeated caveats or setup paragraphs removable? |
| Transfer | Is there one nearby changed case, with an available answer, that lets the reader use the idea beyond the worked inputs? |
| Production integrity | Do local links, anchors, figures, exact code blocks and evidence destinations work in the generated output as well as the source? |

Run reviews in three different modes. First read the argument without the code
to find missing reasons. Then follow only the code and traces to find continuity
and state errors. Finally read the rendered chapter in order, including the
controls and navigation, to find layout and pacing failures. Give each pass a
short factual record of changes and unresolved questions; another pass should
answer a concrete concern rather than merely increase the pass count.

## Publishing decisions still owned by the lead

Recommend Markdown as the durable narrative source and a local browser book as
the primary reading edition, with print-friendly static figures and lab traces.
That follows Moss's browser-first subject while keeping the material usable
without interaction. The build tool, output directory and final chapter paths
are publishing choices, not prerequisites for this editorial structure.

The lead still needs to settle how runnable blocks are included without stale
copies, how relative repository links resolve in exported pages, how evidence
appears outside the main narrative, and whether a PDF is part of this first
edition. Public hosting requires its own explicit authorization; a local book
can be completed and reviewed without it.

## Evidence from this blueprint pass

Read the project brief, active task, agent assignment and authoring contracts,
teaching progression, tutorial home, v2 lesson, context shelf, path overview and
the path's checkpoint/evidence structure. Inspected the pinned manifests,
`lessons.rs`, `simulation.rs`, architecture and existing verification summaries.
Read the TutorBro skill and prose guide. Opened the two external references
linked above. This pass used source reading, not a native/WASM build or browser
check. The source confirms movement is still `todo!()` and unscheduled.

Only this new editorial file is assigned to this helper. The proposed structure,
lab specifications and prose are editorial recommendations; no live biology,
existing guide, `NOW.md`, dependency or publication setting is changed here.
