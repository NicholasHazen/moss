# Maintaining the Moss tutorial

[Guide home](../README.md) · [Chapter template](chapter-template.md) · [Teaching progression](progression.md)

Agent maintenance keeps the guide aligned with the code so Nick can return to
one useful edit. Apply the project [coding conventions](../../agents/coding-style.md)
to worked Rust examples as well as live code.

## Directory map

```text
docs/tutorial/
  README.md              Reader entry point
  today-v2.md            Current movement lesson, linked from NOW.md
  today.md               Preserved first version of that same session
  path/                  Sixteen short sessions and their evidence, after today
  01*.md … 05*.md         Stable chapter filenames
  visuals/               Local SVG diagrams, with descriptive Markdown alt text
  reading-in-rustrover.md IDE reading and navigation
  context/               Optional Rust, ECS and project explanations
  authoring/             Contract, template and current example evidence
```

## On this page

- [Keep each document's job clear](#keep-each-documents-job-clear)
- [Revise a chapter when the code changes](#revise-a-chapter-when-the-code-changes)
- [Run the document checks](#run-the-document-checks)
- [Check today's worked answer](#check-todays-worked-answer)
- [Write and verify a checkpoint](#write-and-verify-a-checkpoint)
- [Prepare a future session for actual use](#prepare-a-future-session-for-actual-use)
- [Review the explanation twice](#review-the-explanation-twice)
- [TutorBro provenance](#tutorbro-provenance)

## Keep each document's job clear

[NOW.md](../../../NOW.md) owns the active task and stopping point; the
[ecology plan](../../design/ecology.md) owns model direction; the
[decision log](../../design/decisions.md) records consequential choices.
The tutorial explains the edit and its evidence without creating another backlog.
Keep reusable explanations in `context/` and maintenance instructions here.

Use the current lesson linked by `NOW.md` to connect the active edit to a
realistic session outcome and one optional stretch. Retire obsolete prerequisites when the route changes;
chapter numbers do not require implementation order. Introduce each concept
where it explains a visible result, and keep longer research optional.

The requested v2 keeps the core explanation and answer on one page. Preserve
`today.md` as its first version; route current entry points to `today-v2.md`.
If the movement contract changes, reconcile v2's `movement-v2-helper` answer
with Chapter 4's `movement-helper` reference and the installed tests together.
When a short session links to a companion chapter's future live test, keep its
discriminating inputs aligned with the reference. A stronger printed-answer
test does not strengthen the live checkpoint it links to automatically.

The week-to-month path lives in `path/README.md` and its numbered session files.
That path owns the future task sequence. Older chapters remain completed
walkthroughs or integration companions, with direct links to the canonical
exercise; optional context explains concepts without assigning a second route.
Each proposed 20–30 minute focus block needs connected teaching prose, a useful reminder of
an earlier concept, one concrete edit or investigation, a worked answer and a
bridge to the next consequence. The arcs group dependencies, not deadlines.
Prepare agent-owned plumbing before the learner's focus window. Keep future
APIs and acceptance distinct from the current runnable checkpoint.

The [progression reference](progression.md) connects those features to reusable
abilities: tracing mutation, reading queries, choosing evidence and investigating
the world. Adapt support from the obstacle actually encountered, not a fixed
assumption about what someone should know by session number. A review checkpoint
can change the route without creating a second active task.

Path examples use a `runnable: session-NN` HTML comment before one complete Rust
test module's contents. `python3 docs/tutorial/authoring/check_path_examples.py`
extracts them into an isolated workspace with its own target directory. This
verifies the written reference, not installation in the live ECS schedule.
Summarize that difference in `path/verification.md` and retain exact commands in
dated guide history; update examples and prose together when the actual lesson
changes. `NOW.md` remains the only active task.

Preserve chapter filenames and existing heading anchors. Change reading order
through navigation; if an anchor must change, update every incoming link.
Use ordinary Markdown, relative links, labeled code fences, short contents lists
and tables of at most two columns. No extra preview plugin is required.

## Revise a chapter when the code changes

Read its code, tests, current plan and `NOW.md` before assigning the edit.
Preserve Nick's implementation and paired ownership: preparing a guide, fixture
or test harness does not authorize installing its future biological behavior.

| Status | Meaning |
| --- | --- |
| Ready | Prerequisites exist and the edit matches current code; active only when `NOW.md` selects it. |
| Future | Proposed increment; name the missing agent preparation. |
| Reviewed | Checked against a recorded base; this alone does not mean implemented. |

Keep one next edit prominent. When scope changes, revise its prerequisites,
examples, expectations and navigation together. Record source/test files, base
revision and relevant uncommitted work in [dated guide history](../../history/README.md#tutorial-checks),
with separate review and execution dates; say **not run** for missing execution.
Keep [verification.md](verification.md) and [path verification](../path/verification.md)
as current coverage summaries linked to those records. Append detailed commands,
reviews and observations to dated files, then refresh only affected summary
claims. Preserve earlier results and heading destinations when consolidating.
Link the relevant summary or exact dated checkpoint from chapters.

## Run the document checks

From the repository root, run the agent-owned navigation check after moving or
editing a guide:

```sh
python3 docs/tutorial/authoring/check_docs.py
```

It checks Markdown at the repository root and under `docs/` and `prompts/`:
local file/image targets, heading and explicit-anchor destinations, reference
definitions and uses, and closed code fences. Links and headings inside fenced
examples do not count as real navigation. The checker reads files only; it does
not follow external URLs, compile examples or verify the IDE's rendering.

The script documents its supported Markdown subset. Keep that scope small and
explicit. A new syntax pattern should either gain a focused fixture or be called
out as outside the check; a green report is not a full Markdown conformance claim.
After changing the checker itself, run its fixtures:

```sh
python3 -B -m unittest discover -s docs/tutorial/authoring -p 'test_check_docs.py'
```

These are maintenance commands for the agent, not another learner checkpoint.
Continue to inspect changed diagrams visually and run changed Rust examples with
the isolated reference runner. Each check establishes a different fact.

## Check today's worked answer

After changing the printed movement answer or its integration contract, run:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --movement
```

This agent maintenance command extracts v2's exact helper, compares it with
Chapter 4 after pinned rustfmt, and inserts it into a temporary copy. It first
checks the helper, then activates the copied maintenance → movement → completion
schedule and runs all five prepared movement/maintenance tests, including the
two ignored activation tests. It checks the expected test names actually passed
and runs Clippy. Live source, tests, the lesson and the browser remain unchanged.

The runner deliberately recognizes the current helper signature and schedule.
If either has changed, review and update its assumptions; it stops instead of
guessing a new integration. `--keep-workspace` retains the copy for investigation;
otherwise it is removed, even after failure. The copy has its own target directory
and uses cached dependencies. This command verifies the printed answer, not Nick's
edit, browser behavior or the full workspace suite.

With no mode flag, the same script checks all sixteen future references;
`--session N` selects one. These modes do not activate movement. The runner lists
the compiled tests, requires at least one in each selected session module, then
requires every discovered test to pass. Ignored references fail verification;
they are not silently enabled. This proves execution, not assertion quality or
coverage of additional tests excluded by conditional compilation. After changing
the runner itself, check its failure and isolation fixtures:

```sh
python3 -B -m unittest discover -s docs/tutorial/authoring -p 'test_check_path_examples.py'
```

## Write and verify a checkpoint

Begin with a concrete Moss result and explain why the next small change produces
it. Follow one familiar animal or test world; introduce the Rust/ECS concept
where it helps explain the edit. Keep complete worked answers available.

Give every code block its file/symbol, placement, required imports and a label:
complete addition, replacement, test, excerpt, or expected output. Name missing
preparation and label future APIs. Each checkpoint needs an action, literal
expected result, why it is useful evidence, and one targeted failure diagnostic.
Use the exact command and test name; zero matching tests is not green.
Distinguish an intended red assertion from compilation or setup failures.

Run changed worked examples on the pinned toolchain. For future paired work,
use an isolated copy with its own `CARGO_TARGET_DIR`; never share the live
target directory. Record actual commands/outcomes and label **observed** versus
**expected** results. Helper tests, installed-schedule checks and browser
observations establish different things; state which remain unverified.
Project runtime evidence belongs in [development verification](../../development/verification.md).

Offer at most one optional variation with an immediately available answer or
check. Resolve the opening problem, provide a copyable review request, and stop.
Green tests do not activate the next behavior.

## Prepare a future session for actual use

A future guide is a design for a lesson, not a ready assignment. Before offering
it, inspect the reviewed code and turn its opening into an operational handoff:
name the one editable symbol, link its actual file, and give one exact live test
command or IDE action. Verify that the test detects the intended missing behavior
before Nick starts. For an assertion exercise, prepare a valid running scenario;
do not add an intentional bug merely to manufacture a red test. Explain whether
the initial state is a missing rule, a missing assertion, or an investigation.

Keep the printed-answer runner secondary. A passing extracted example says
nothing about whether Nick's local body is finished. Put the live checkpoint
first when activated, and retain the optional reference under its explicit label.
Mechanical compilation, fixture preparation, type exports and UI wiring happen
outside the learner's 20–30 minute estimate. If preparation reveals a larger
design change, reduce the next edit or update the route before handing it over.

Read across lesson boundaries before preparing an integration test. Growth can
rescue an animal expected to starve; a pre-authored target can conceal choice
running at the wrong time. State relevant inputs such as light, activity, target
presence and costs, then ask what incorrect implementation would also pass.
Where practical, use a temporary mutation in an isolated workspace to check that
the regression detects the specific ordering or eligibility defect it claims.

Use complete worked answers, but show the decision before its scaffolding.
An input/output trace or a short assertion excerpt can make a longer fixture
readable. Name copied or borrowed values concretely, explain a new Rust feature
where it becomes useful, and revisit a concept through one changed case with an
available answer. Remove repetitive caveats and implementation detail when they
obscure the edit; preserve the actual limitation in the evidence record.

The [learning-design research note](../../research/learning-design.md) records
the sources and limits behind these choices. It informs revisions; it is not a
claim of measured learning gains in Moss. Learn from Nick's actual session:
record the point of confusion and fix its explanation rather than adding a
general prerequisite list. Returning to the same concept is useful when the new
example requires it, not simply because a template has a reminder slot.

## Review the explanation twice

First check understanding: current state, intended change, location, reason,
evidence and stopping point. Confirm prerequisites and learner ownership.
Then edit prose for connected explanations, concrete terms and useful brevity.
Check contents, previous/home/next links, source links, anchors and code fences.
Report execution or rendering limits honestly.

## TutorBro provenance

Nick requested [TutorBro](https://chatgpt.com/skills?skill_id=6ab16fad9a7881919144924b499536e8).
This maintenance pass used the locally installed `tutorbro-tutorial-writing`
skill and its prose-guide/exemplar references. Earlier retrieval and execution
evidence remains in the dated guide records. Re-read sources when guidance changes.
Use **S02: stable example** and **S05: executable increments**, subject to Moss's
scope, learner ownership and verification rules.
