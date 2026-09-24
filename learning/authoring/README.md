# Maintaining Moss Fieldnotes

[Fieldnotes setup](../README.md) · [Course structure](../content/roadmap.html) · [Project documentation](../../docs/README.md)

Fieldnotes is the project's learning book. Maintain its explanations and complete
answers alongside the code, so the reader can move from a concrete question to
an edit and useful evidence. Apply the project's [coding conventions](../../docs/agents/coding-style.md)
to worked Rust examples as well as live code.

## Keep each document's job clear

[NOW.md](../../NOW.md) owns the active live task and stopping point. The
[ecology plan](../../docs/design/ecology.md) owns proposed project behavior;
Fieldnotes' [introduction](../content/introduction.html) explains its approach,
and the [roadmap](../content/roadmap.html) describes the learning route. The course explains
and practices rules without silently installing them in the live application.
Canonical design, development and agent documents remain project references,
not competing books.

The HTML fragments in `learning/content/` contain reader prose and answers;
`course.json` registers the route. Numbered chapters explain focused concepts,
and cumulative guides extend one saved course project. `learning/checkpoints/`
holds the complete cumulative reference sources. The [reading guide](../content/about.html)
explains the public tools and model limits; private delivery records are omitted
from this source edition. Dated project records stay under `docs/history/` with
their original observations.

The [movement chapter](../content/14-movement.html) is the sole worked guide for
`move_one_cell`. Its tagged answer and the live tests must agree. The earlier
mdBook and parallel Markdown tutorials have retired; their source history is
preserved in Git, not maintained as another learning route.

## Revise a chapter when the code changes

Inspect actual source and tests before changing a lesson. Preserve Nick's
implementation and paired ownership: preparing a guide, fixture or test harness
does not authorize installing its future biological behavior. Name the one
editable symbol, prerequisites and stopping point. Label proposed APIs and
isolated reference behavior where the reader needs that distinction.

An opening overview should connect the concrete problem to the goal, reason,
route and expected outcome in natural prose. Major conceptual transitions need
a reason to appear. End by resolving the opening question and linking the next
useful application. Keep hints and complete answers immediately available.
When scope changes, revise examples, expectations and navigation together.

## Run the document checks

From the repository root:

```sh
python3 scripts/check_docs.py
python3 -B -m unittest discover -s scripts -p 'test_check_docs.py'
python3 learning/scripts/check.py
```

The first command reads Markdown at the repository root, under `docs/` and
`prompts/`, and in Fieldnotes' top-level and authoring documents. It checks local
targets, Markdown headings, reference links and closed fences within its documented
subset. The second checks that tool's fixtures. Neither fetches external URLs,
compiles Rust or verifies rendering.

The course check builds Fieldnotes and validates its generated HTML navigation,
source views and required structure. Its own test suites cover the build and
interactive controls. Use the [course maintenance commands](../README.md#check)
for their exact entry points. Review changed diagrams and layout visually;
structural checks cannot establish how a page reads or prints.

## Check today's worked answer

After changing the movement answer or its integration contract, run:

```sh
python3 learning/scripts/check_movement_reference.py
python3 -B -m unittest discover -s learning/scripts -p 'test_check_movement_reference.py'
```

The runner extracts the one `data-movement-reference="move_one_cell"` Rust block
from the movement chapter and inserts it into a temporary copy. It first runs
the named helper test, then activates maintenance → movement → completion in
that copy and runs all five prepared movement/maintenance tests, including the
two ignored activation tests. It requires the expected names to report passing
and runs Clippy. Live source, tests and the browser remain unchanged.

The runner recognizes the current helper signature and maintenance-only
schedule. It stops if either drifts; review those assumptions before adapting
it. `--keep-workspace` retains the copy for investigation. Otherwise the copy is
removed even after failure. It uses cached dependencies and its own target
directory. This verifies the printed answer, not Nick's body, browser behavior
or the whole workspace suite. The fixture tests exercise failure and isolation
boundaries without invoking Rust.

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
Project runtime evidence belongs in [development verification](../../docs/development/verification.md).

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

The [learning-design research note](../../docs/research/learning-design.md) records
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
