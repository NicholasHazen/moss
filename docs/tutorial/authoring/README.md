# Maintaining the Moss tutorial

[Guide home](../README.md) · [Chapter template](chapter-template.md)

## Directory map

```text
docs/tutorial/
  README.md                 Reader entry point and current route
  01-species-energy.md       Existing chapter IDs stay stable
  02-populations.md          Later chapters follow the same pattern
  03-food-choice.md
  04-movement.md
  05-eating.md
  reading-in-rustrover.md    IDE reading and navigation
  context/                  Optional reusable explanations
  authoring/
    README.md               This maintenance contract
    chapter-template.md     Starting point for a new increment
    verification.md         Evidence, dates and concrete limits
```

This page is for the agent maintaining the guide. Nick should be able to return
to one useful edit without maintaining a curriculum or reconciling several
competing plans.

## On this page

- [Keep each document's job clear](#keep-each-documents-job-clear)
- [Revise a chapter when the code changes](#revise-a-chapter-when-the-code-changes)
- [Write and verify a checkpoint](#write-and-verify-a-checkpoint)
- [Review the explanation twice](#review-the-explanation-twice)
- [TutorBro provenance](#tutorbro-provenance)

## Keep each document's job clear

[NOW.md](../../../NOW.md) owns the single active task, the next edit, and the
stopping point. The [ecology plan](../../ECOLOGY_PLAN.md) owns the current model
direction; the [decisions](../../DECISIONS.md) record consequential choices.
[Build notes](../../BUILD_NOTES.md) hold project verification evidence. The
tutorial explains how to make the selected changes and how to recognize that
they work. It must not quietly create a second backlog or turn a proposed
behavior into an implemented fact.

Keep `01-species-energy.md` through `05-eating.md` and existing linked heading
anchors stable. Add chapters when there is a useful next increment; do not
renumber earlier chapters to express a new priority. Update the guide's reading
order and `NOW.md` instead. If a heading must change, update every incoming link
in the same edit. Check links before the handoff.

Keep explanations needed for the edit in its chapter. Put reusable Rust, ECS,
and project background in `context/` and link to the relevant section. Keep
templates, author instructions, and validation procedures here, outside the
reader's main path. Use ordinary Markdown, relative project links, short code
blocks, unique headings, a short manual contents list, and tables with at most
two columns. Do not require an extra preview plugin or HTML callout support.

## Revise a chapter when the code changes

Before making a chapter active, read its named code, tests, current plan, and
`NOW.md`. Reconcile the examples with Nick's implementation before assigning an
edit. Preserve his work and the paired ownership of biological rules. Preparing
a tutorial, fixture, or test harness does not authorize implementing its future
behavior in the live simulation.

Give each chapter a small status note:

| Status | Meaning |
| --- | --- |
| Ready | Its stated prerequisites exist and the next edit matches the current code. It is active only if `NOW.md` says so. |
| Future | It teaches a proposed increment; name the preparation that must happen before following it. |
| Reviewed | Its explanation and examples were reviewed against the recorded base. This does not mean the feature has been implemented. |

Record the source files and base revision used for the review. Name any relevant
uncommitted change instead of pretending the commit alone describes the base.
Record a review date and a separate verification date, using **not run** when
appropriate. Dates describe evidence; they do not guarantee that later edits
leave the chapter accurate. Recheck a chapter whenever its inputs change.
Keep the detailed base/date record in [verification.md](verification.md) and link
it from chapters; do not make every reader opening carry a wall of metadata.

Keep one concrete next edit prominent. Later steps may be available to read,
but the chapter should say where this sitting can end. When scope changes,
update the affected explanation, expected results, prerequisites, navigation,
and current handoff together. Record consequential model choices in the plan
or decision log rather than hiding them in sample code.

## Write and verify a checkpoint

Open with a concrete result or problem in Moss. Continue through the reason for
the change, the data it needs, and the small edit that makes that result
possible. Use a stable example across the chapter so that a new capability is
visible without requiring the reader to reconstruct a different scenario.
Introduce one principal Rust/ECS concept at a time, where it helps explain the
edit. Connected paragraphs should carry the explanation; a list of headings
and commands is not a substitute for it.

Every code block needs a location and a label: complete addition, complete
replacement, test, excerpt, or expected output. State required imports and
prerequisites. Do not present a future helper or signature as an existing API.
Keep complete worked answers available; a request for a tutorial is not a
reason to withhold code until Nick passes a quiz.

Each checkpoint supplies four things: an action, the expected result, why that
result is useful evidence, and one targeted diagnostic for the likely wrong
result. Use the exact test name and command. Check that the test actually ran;
zero matching tests is not green. When teaching a new rule, distinguish its
intended red result from compilation or setup failures.

Verify changed worked examples against the pinned toolchain, preferably in an
isolated copy when the implementation belongs to a later paired session.
Give that copy its own `CARGO_TARGET_DIR`; sharing the live target directory
has already caused Cargo to reuse example test artifacts during a live check.
Record which examples ran and their actual outcomes. Label future results as
**expected** and observations from execution as **observed**. A helper test does
not prove installed schedule behavior, and a native test does not prove a
browser observation. Keep unexecuted integration and browser checks explicit.

Offer at most one adjacent variation per checkpoint, clearly optional, with
its answer or check immediately available. It should clarify the current
concept, not add an unexpected feature. Finish by resolving the opening
problem, connecting the result back to Moss, and giving a copyable request for
review. The rhythm remains **tests → review → stop**; green does not silently
activate the next behavior.

## Review the explanation twice

First review understanding: can Nick identify the current state, the intended
change, its location, the reason for it, the test evidence, and a stopping point?
Check that the example is stable, proposed APIs are labeled, and biological
ownership remains intact. Fix gaps in the causal explanation before polishing
sentences.

Then review prose: replace repetitive scaffolding with connected paragraphs,
define unfamiliar terms at their first use, keep sections short enough to
navigate in an editor, and remove extra tasks disguised as context. Check the
manual contents links, previous/home/next links, file links, headings, code
fences, and narrow tables. Report review or rendering limitations honestly.

## TutorBro provenance

This structure applies [TutorBro](https://chatgpt.com/skills?skill_id=6ab16fad9a7881919144924b499536e8),
which Nick explicitly requested. The lead agent read its `SKILL.md`,
`references/prose-guide.md`, and `references/exemplars.md` in the signed-in
browser during this guide revision. TutorBro was available through that shared
skill page; it was not locally installed. This maintenance contract records the
verified requirements for maintaining these project documents. Re-read the
source skill and relevant references when its guidance changes. Do not claim
to have applied a newer revision without retrieving it.

The selected techniques are **S02: stable example** and **S05: executable
increments**. They guide the explanation and checkpoints while Moss's project
instructions continue to govern scope, learner ownership, and evidence.
