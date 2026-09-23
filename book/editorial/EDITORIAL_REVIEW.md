# Independent reading review — September 23, 2026

The book has a coherent teaching arc and a clear first contribution. It follows
consequences in one small world rather than teaching Rust syntax in isolation:
a paid step leads to a bounded meal, a meal creates a reason for choice,
individual differences make comparison useful, and finite food leads to growth
and lifecycle questions. The concrete numerical traces do substantial teaching
work. Preserve that arc and its explanatory depth.

For Nick's existing checkout, the first next action is clear: replace only
`move_one_cell`, run one named test, and stop for review. For a new public reader,
the route to that same starting point is missing. That is the only finding here
that should block calling the public edition a reproducible build-along. The
remaining changes would make the longer web chapters easier to enter and resume.

## 1. Provide the public reader with the edition's exact starting workspace

**Priority: before public build-along release.** Locations:
`src/index.md`, “A book beside the workbench” and “The world at this edition's
starting line”; `src/chapters/01-one-affordable-step.md`, “Your first edit”
(lines 18–29 at review time).

The book invites readers to open a project file and run a prepared test without
first telling them how to obtain this edition's source, install its prerequisites,
or identify the supported environment. The GitHub button leads to a moving
repository, while the chapter describes a particular unfinished helper and
prepared test. Escaped individual-file snapshots preserve evidence but are not
a runnable checkout. The first encounter could therefore be a missing script,
a missing Run configuration, or a project already beyond this exercise rather
than the intended red test.

Add one short **Start coding** page or section before the first command. Link
an exact verified revision or a reviewed starter archive, give the minimum
toolchain/setup path for the platforms actually verified, and identify the
expected red test. Keep the existing short route for someone whose current
Moss checkout already matches the edition. Do not imply that the book's isolated
future references are installed in that workspace.

Suggested bridge in the opening:

> Already using the prepared Moss checkout? Open the first edit below. Starting
> from the public book? Use [this edition's starting workspace] first; it contains
> the unfinished function and the test this chapter expects.

The root agent was already considering an exact starter bundle. This finding
does not select the packaging mechanism or request that unrelated work be
committed. Verify whichever acquisition path is actually published.

## 2. Let the reader resume a long chapter without scrolling past every module

**Priority: worthwhile before publication.** Locations:
`src/chapters/04-two-hares-one-fair-comparison.md`, “Checkpoint: both the right
values and the right members” and “Run the reference, then check the live
connection”; `src/chapters/05-a-world-that-feeds.md`; and
`src/chapters/06-a-disappearance-explained.md`, its four checkpoint sections and
“Inspecting the four complete references.”

Chapter 4 contains four full reference blocks totaling 275 Rust lines, including
one 111-line module. Chapter 6 contains four totaling 247 lines. These are useful
complete answers, but their placement makes a conceptual reread pass through
hundreds of lines of fixture construction and assertions. Commands for those
references appear near the chapter ends rather than beside the blocks. The
global sidebar does not expose the checkpoints within a chapter.

Keep the full answers. Add a short local contents list organized around the
chapter's questions, and put each reference's one exact command beside its own
block. For the longest modules, a native HTML `details` disclosure with a clear
summary would let the narrative continue while leaving the answer one action
away. Keep purpose, key behavior, input/output trace, and explanation outside
the disclosure. Preserve the exact marked Rust bytes and confirm Markdown,
search, copy, keyboard, and print behavior after that change.

For example, chapter 4's opening could offer:

> Follow the argument through [owned costs], [query membership], and [the
> controlled comparison]. The complete reference modules remain available at
> each checkpoint; you can leave them folded while reading for the idea.

This is a navigation/progressive-disclosure recommendation, not a request to
turn a textbook into a terse checklist or remove the tests that explain why
the design works. The short movement answer should stay immediately visible.

## 3. Replace the coda's final assignment-shaped question with a recommended return

**Priority: small handoff improvement.** Location:
`src/chapters/07-another-kind-of-life.md`, final paragraph.

The horizon discussion appropriately labels predation, rest, reproduction,
inheritance, weather, and geometry as future possibilities. Its final bold
question asks the reader to select an interaction and design an explanatory
tick. For a reader with low executive function, that can become another task
at exactly the point where the book should return them to the one already
prepared. Nick explicitly asked to be given a recommended next move instead
of having to invent one.

Keep the question as an optional reflection if desired, then close with an
explicit return link:

> Keep these questions for a later life. To begin this one, return to [one
> affordable step], make its single test pass, and ask for review. The next
> biological change can follow what that small result teaches us.

Do not make a later feature active merely because the reader reached the coda.

## 4. Tighten status notices after the edition is actually finished

**Priority: final publication polish.** Locations:
`src/index.md`, “Six questions to grow a world”; and the openings of chapters
2, 4, and 6.

The index currently says the early edition “is being built” and that the
outline does not mean every chapter is ready, despite all seven chapters being
present. That is honest during production but leaves the published reader
uncertain whether a listed chapter is missing, editorially unfinished, or
simply describing unimplemented runtime behavior. Those are different states.

When QA is complete, give the edition's publication status once, and preserve
the separate biological status: chapter 1 is the prepared edit; later chapters
are worked designs and isolated references. Each future chapter still needs
its concise entry notice because it may be opened directly.

The openings of chapters 3 and 5 already work well: concrete problem first,
then the scope notice. Use that ordering for chapters 2, 4, and 6 instead of
making the first paragraph an implementation disclaimer. Do not remove the
limitations next to examples where they explain a consequential evidence gap.

## What already works and should survive editing

- The index separates Fern's nickname, species, role, and ECS archetype without
  introducing a large ontology. Chapter 4 revisits the distinction when multiple
  animals make it consequential.
- Chapter 1 gives a useful edit before theory, distinguishes the intended
  `todo!()` failure from compiler/setup failure, and explains copying versus
  writing through a borrow with the same two assignments.
- Chapter 2 distinguishes a safe mutable borrow from a correct allocation
  policy. Its reversed input order and four-versus-five biomass cases make that
  distinction observable.
- Chapter 3's 74/73/77 trace resolves an apparent contradiction using phase
  order. The 72 start reuses the model and supplies an immediate answer.
- Chapter 4 addresses the actual attribute discussion: missing versus zero,
  copied defaults versus live inheritance, owned values versus provenance,
  component access boundaries, and baseline/effective/current state. It avoids
  claiming a performance result from an attractive type layout.
- Chapter 5 follows an empty patch through both growth-and-consumption and
  no-production. The same final biomass acquiring two different explanations
  is a strong reason for outcome records.
- Chapter 6 distinguishes requested maintenance from actual deduction and
  exposes the no-debt starvation consequence. It also separates retained detail
  from whole-run coverage instead of promising a complete history.
- The code map begins with a behavior to investigate, not an exhaustive file
  list. The reference shelf tells readers what question to bring to each source
  and is clearly optional.
- Future behavior, illustrative numbers, expected integration results, and
  executed reference checks remain substantially distinct. I found no prose
  claim that the proposed ecology already runs in the live browser.

## Scope and evidence

Read the index, all seven chapters' teaching prose and example framing, summary,
code map, concept index, reference shelf, and publishing README/configuration.
Inspected the length and placement of the complete Rust blocks and read selected
blocks where their presentation mattered. Read AGENTS, PROJECT_BRIEF, NOW, the
tutorial authoring and subagent contracts, and TutorBro's skill, prose guide,
and exemplar notes. Review used the stable-example and executable-increment
techniques; it did not claim measured learning gains.

This is an independent editorial review, not another Rust correctness audit.
No builds, tests, browser operation, network research, live-source mutation, or
guide edits were performed. Root-provided test results were context, not rerun
evidence. The only file changed by this review is this report. Line numbers may
move as root integrates its parallel formatting work; heading locations above
are the intended anchors.

## Reader refinements completed

At root's subsequent request, revised the seven chapters. Chapters 2–6 now have
local question-based contents lists. Each of the sixteen future reference
modules has its exact command beside it; the longer modules use ten native
`details` disclosures with their central functions retained as labeled visible
excerpts. The movement helper remains open. Chapter titles 2, 4, 6 and 7 match
the unnumbered title/eyebrow style, with explicit aliases for their previous
heading anchors. Future-status notices follow the concrete opening problem,
and the coda ends with the recommended return to the prepared movement edit.
Root's new chapter 6 evidence figure was preserved. Chapter 1's five movement
controls now start disabled until root's JavaScript setup enables them, matching
the other labs' no-JavaScript fallback.

Source-only checks passed: `check_markdown` inspected 20 Markdown files and
186 local destinations; `check_examples` confirmed all 17 canonical copies.
A separate SHA-256 comparison confirmed every marked Rust block was byte-for-byte
unchanged from the pre-edit baseline. All ten disclosure tag pairs balanced,
and `git diff --check` passed. No HTML rebuild, browser/keyboard/search/print
check, Rust execution or live code change was performed in this refinement.
Root retains those publication checks and the separate starter-workspace task.
