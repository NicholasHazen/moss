# Learning review — second editorial pass

Reviewed September 23, 2026. Scope: the public edition's entry points, chapters
1–4, and their transitions into growth and lifecycle. This is a read-only review
apart from this report. It does not activate an exercise or propose new biology.

The main progression works: a paid step leads to a bounded transfer, a useful
destination creates a reason for choice, and individual differences create a
reason for owned costs. The first movement edit is particularly clear. I found
four worthwhile refinements, all bounded prose or navigation work. None requires
changing the canonical Rust answers.

## 1. Make the public edition's two kinds of coding checkpoint explicit once

**Priority: P2 — teaching-contract refinement, not an unmarked future API.**

Evidence: `book/src/index.md:36–40` promises that, when the reader wants to code,
the chapter supplies a small checkpoint, exact command and worked answer.
The accurate qualification at `index.md:52–56` says later features are proposals,
but does not distinguish a live edit command from a command that runs an already
complete reference. That operational distinction appears much later, in
`book/src/reference/start-here.md:138–153` and, for example,
`book/src/chapters/02-a-meal-has-two-sides.md:214–228`.

A reader can understand that eating is not installed and still reasonably expect
the meal command to test an implementation they have just edited. In fact, it
extracts a complete answer from a separate canonical Markdown file. The chapter
explains this honestly, but the front-door promise could set the expectation
correctly sooner, especially for a public reader who does not share Nick's
existing paired workflow.

**Bounded change:** replace the broad coding promise at `index.md:36–40`, rather
than adding another warning to every chapter:

> You can follow the main argument and its pictures without opening an editor.
> Chapter 1 also gives you a ready-to-edit function and a test of your change.
> The later chapters let you explore and run complete reference examples; their
> live exercises are prepared one at a time during paired work. Running a reference
> checks the printed answer. It does not install that behavior in your world.

Then retain the existing complete-answer invitation. Do not solve this by
activating future rules or making the book require a particular assistant product.

**Acceptance:** a reader arriving on the home page can tell which command tests
their saved code and which command demonstrates the published answer, before
choosing a chapter.

## 2. Show what autonomous choice does to the familiar first journey

**Priority: P2 — concrete transition gap.**

Evidence: chapter 1 establishes nine immediate steps from reserve 60 to Meadow
(`book/src/chapters/01-one-affordable-step.md:333–339`). Chapter 3 introduces the
start-seeking threshold of less than 45, with maintenance before choice
(`book/src/chapters/03-let-the-animal-choose.md:327–344`). Its concluding browser
story is qualitative (`03-let-the-animal-choose.md:558–564`), while its excellent
literal trace uses a different, already-seeking animal on the patch at reserve
74 (`03-let-the-animal-choose.md:467–509`).

If the future autonomous fixture initializes Fern as Idle with the familiar
reserve 60, the first fifteen ticks will not move her. That could look like a
regression to a reader who just learned that the same starting world produces
an immediate nine-step journey. The policy explains the difference, but the
chapter never brings that consequence back to its original cast and start.

**Bounded change:** before the final browser paragraph, add a short conditional
trace. Keep it explicitly illustrative until the live fixture is chosen:

> If we initialize the autonomous Fern as Idle with reserve 60, she will wait
> before making the journey. With upkeep one, tick 15 ends at reserve 45 and
> `(10, 10)`. Tick 16 pays upkeep first, so choice sees 44, begins Seeking and
> permits the first step; she finishes at `(11, 10)` with 42. With the same
> unobstructed nine-cell route and untouched Meadow, tick 24 is arrival: reserve
> 18 before the meal, 22 afterward, and 76 biomass left. Choice has changed when
> the journey begins, while the step and transfer rules stay the same.

State the other assumptions: radius ten, travel price two, bite four, no growth,
no competitor, no death. Alternatively choose a deliberately hungry diagnostic
start for quick integration and say why. Do not silently select a new live
fixture in this editorial pass.

**Acceptance:** the reader can distinguish “Idle by policy” from “movement is
unscheduled” without changing a correct helper. The existing 74/8 lab remains
the compact explanation of stop timing.

## 3. Explain the reference types hidden by the meal loop's tuple pattern

**Priority: P2 — Rust transfer-of-understanding gap.**

Evidence: `book/src/chapters/02-a-meal-has-two-sides.md:318–332` introduces
`for (id, energy) in eaters`, where `eaters` has type `&mut [(SimId, Energy)]`.
The subsequent prose explains the slice, sorting closure, and persistent patch
mutation (`02-a-meal-has-two-sides.md:459–479`), but leaves the loop bindings'
types implicit. In contrast, chapter 3 carefully explains its explicit
`for &(id, position, biomass)` pattern and value copies
(`book/src/chapters/03-let-the-animal-choose.md:259–278`).

The two visually similar loops teach different operations. In the meal loop,
the local `energy` is a mutable reference into the caller's slice; it is not a
copied `Energy` value. That is the point most likely to matter to an experienced
reader returning to Rust who previously asked for help with loop APIs.

**Suggested insertion after chapter 2's slice paragraph:**

> Iterating this mutable slice yields a mutable reference to each pair. Rust's
> pattern matching lets `(id, energy)` name the fields through that reference:
> here `id` is `&mut SimId` and `energy` is `&mut Energy`. Passing `energy` to
> `eat_from_patch` therefore lends the animal's existing reserve to the helper.
> The record uses `*id` to copy the small identity value out; it does not keep a
> borrow into the slice after the meal.

Optionally add one sentence at the chapter 3 pattern explanation: its leading
`&` explicitly unpacks a shared reference and copies all three values, unlike
the earlier meal loop's mutable field references. This earns a revisit of
borrowing through a changed case without adding a syntax survey.

**Acceptance:** the reader can say why changing energy in one loop changes its
caller, while changing the copied biomass observation in the other would not.

## 4. Give chapter 4 a short route through its central cost experiment

**Priority: P3 — navigation and cognitive-load refinement.**

Evidence: the opening asks where a same-species animal's individual rate should
live (`book/src/chapters/04-two-hares-one-fair-comparison.md:7–22`). Before that
answer, the reader encounters a population summary checkpoint
(`04-two-hares-one-fair-comparison.md:42–206`). After the owned-cost query,
component-granularity, modifiers, effective values and performance occupy
`04-two-hares-one-fair-comparison.md:616–722`, before the promised controlled
journey resumes at line 724. The page has four independent Rust references.

All of this material belongs in the book, and the attribute discussion directly
answers Nick's questions. The problem is not its presence or correctness. The
reader does not get a recommended short route through a chapter that contains
several sessions, and an abstract design question interrupts the payoff of the
cost experiment they have just followed.

**Bounded change:** add a two-sentence route beside the contents, keeping existing
headings, examples and anchors intact:

> To follow one cost from authoring to its visible effect, read **A default is
> an instruction**, **Make the query read the new owner**, then **Give the
> difference a fair journey**. The population summary and the sections on
> component boundaries, modifiers and scale are adjacent investigations you can
> return to after that account makes sense.

Make those three labels links to the existing headings. At the end of the
query section, add a direct “Continue the two-hare experiment” link to the
journey. Do not move or collapse the core defaults discussion, remove the
architecture material, or imply that all four references are one short exercise.

**Acceptance:** a reader with one sitting can trace default → owned component →
query → 51/48 reserves without first deciding which adjacent design questions
must be understood.

## Strengths worth preserving

- The first edit names its file, function, exact test, deliberate red state,
  stopping point and a copyable review request. It does not confuse a helper's
  success with installed movement.
- Complete answers remain available. Later long fixtures are expandable while
  the important decision stays in the reading flow.
- The stable Fern/Meadow example turns borrowing, resource conservation,
  observation and ordering into related problems. The explicit 36-versus-37
  meal explanation is a particularly useful repair of a plausible misconception.
- Chapter 3's 74 → 73 → 77 trace explains why displayed fields can reflect
  different phases. It is stronger than a generic “systems run in order” lesson.
- Chapter 4 distinguishes zero, absence, equal values and provenance, and
  explains component access without promising an unmeasured performance win.
- The move from consumed food to growth, then from scarcity to a timed lifecycle
  rule, is causal and keeps future work visible without installing it.

## Evidence and limits

Read the current project instructions, brief, handoff, subagent contract,
tutorial authoring contract, TutorBro skill, prose guide and exemplar notes.
Applied the stable-example and executable-increment techniques as editorial
judgments, not measured evidence of learning gains. Read the entry pages and
chapters 1–4 in full, inspected the source/reference excerpts relevant to these
findings, and read the openings and endings of chapters 5–6 plus the canonical
food-choice companion and short-session path.

Used shell reads and searches only, plus a small independent arithmetic trace
to check the conditional idle-start values above. That trace produced tick 15:
45 at `(10, 10)`; tick 16: 42 at `(11, 10)`; tick 24: 22 at `(16, 13)` with
76 biomass. It is not a Rust test, a live integration check or observed browser
behavior. No build, browser audit, user study, runtime change or deployment was
performed. The report records inspected source line numbers; integration may
shift them.

## Integration review

Re-read the lead's revised passages on September 23, 2026. All four findings
are addressed without changing the intended rules: the home page distinguishes
live edits from reference execution; the meal loop and observation loop now
explain their different bindings; the conditional Idle-start table correctly
ends tick 15 at 45 and tick 16 at 42 after one step; and chapter 4 offers linked
navigation through its cost experiment while retaining the adjacent design
material. Limiting the new trace to the first movement is sufficient and avoids
adding another meal checkpoint. Its hypothetical fixture is explicitly labeled.

No actionable correction found in these integrated passages. This follow-up was
a source/prose review; exact Rust-block preservation, link checking and the
rendered build are being verified separately by the lead.
