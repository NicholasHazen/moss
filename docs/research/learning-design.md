# Teaching small changes in Moss

[Research index](README.md) · [Tutorial authoring](../tutorial/authoring/README.md)

**Reviewed September 22, 2026.** This note explains choices behind the tutorial,
not additional reading Nick must complete. The reader is an experienced programmer
returning to manual coding, learning Rust and ECS through an observable ecosystem.
The aim is a useful next action with enough explanation to make it transferable.
More pages, passing examples and agent approval cannot establish that learning
has happened. Nick's ability to explain and adapt an actual change remains the
important evidence.

## Combine an example with a small use of the idea

The US Institute of Education Sciences guide recommends spacing learning,
alternating worked solutions with problems, combining diagrams with words, and
connecting concrete and abstract representations. It rates these recommendations
as having moderate evidence and distinguishes stronger support for retrieval of
previous content from weaker support for some other quiz practices. Its scope
spans subjects and educational settings; it does not validate this particular
Rust curriculum or a twenty-minute session length.
[IES practice guide, 2007](https://ies.ed.gov/ncee/wwc/PracticeGuide/1).

Our application is modest. A meal lesson shows a complete transfer, then asks
for one bounded implementation or changed input. The answer remains available.
Later growth reuses the same limit calculation, so the reminder has a purpose
in the next edit. We do not turn a return after a gap into a quiz, add a streak,
or ask Nick to remember syntax before allowing him to continue. The 20–30 minute
window comes from his requested working rhythm, not from a research claim.

## Name the job a piece of code performs

Margulieux, Morrison and Decker studied examples labeled by their functional
subgoals in introductory Java courses. Their pilot reported better early quiz
performance, but not better average scores on submitted exams. They also identify
limitations, including an instructor who belonged to the research team. This is
support for investigating explanatory labels, not proof that labels improve every
programmer's learning or that findings transfer unchanged to experienced Rust
learners.
[ITiCSE 2019 paper](https://laurenmarg.com/wp-content/uploads/2019/04/fp097-margulieux.pdf).

For Moss, “compute room,” “choose the allowed transfer,” and “apply that same
amount” explain why three groups of statements exist. Comments such as “declare
a variable” merely restate syntax. The same functional grouping can recur in
growth without pretending the two biological rules are identical. Where a short
function is already legible, extra labels add noise and should be omitted.

## Connect borrowing to the values being accessed

Crichton, Gray and Krishnamurthi investigated Rust ownership misconceptions and
developed a permission-based model with program-state visualizations. Their
evaluation concerned an ownership inventory and a particular textbook
intervention; it does not establish that any ownership diagram is effective.
The useful question for this guide is whether a reader can connect a borrowing
rule to the actual access and mutation in the example.
[OOPSLA 2023 project and paper](https://cel.cs.brown.edu/paper/ownership-conceptual-model/).

The meal diagram therefore follows the caller's same two values into and out
of a function call. It shows that the helper can change them through `&mut`,
without copying an ECS world or transferring ownership of the entities. The
defaults diagram has a different job: the resulting component is an owned copy,
so modifying a template later does not mutate an existing animal. Conflating
those pictures would teach the very confusion we want to remove.

## What revision should look for

Reading, discussing and changing code provide different evidence of understanding.
Sentance, Waite and Kallia's PRIMM study reports an evaluation in 13 schools with
493 pupils aged 11–14 over 8–12 weeks; the institutional abstract reports better
post-test results than the control group. That school context and the abstract's
limited detail cannot establish an effect for an experienced adult learning Rust.
The useful design prompt here is the progression from inspecting an example to
modifying and making something with it, rather than treating copied code as the
only evidence. [Authors' institutional record and abstract](https://eprints.gla.ac.uk/229013/).

For Moss, this informs the [authoring progression](../tutorial/authoring/progression.md):
trace one value, make a small decision, then reuse the idea when another behavior
needs it. Support can recede when that part is comfortable and return when a new
mechanism appears. Complete answers remain accessible. This is our adaptation,
not a claim that Moss implements or validates the studied intervention.

A readable example should expose the decision before its support machinery.
When the learner adds one assertion, first explain the observation that makes
that assertion worth writing. Place the full runnable fixture afterward and
identify which setup can be skimmed. When several Rust features arrive together,
trace the values through an ordinary `match` or loop before introducing a compact
equivalent. Neither verbosity nor terseness is inherently more idiomatic.

Review the boundary between lessons as carefully as each function. An empty patch
can regrow after the daylight lesson, so a later starvation test must state the
light and production conditions. A final-state check can pass under the wrong
schedule when the fixture already contains a target; remove that target when the
claim is that choice observes new growth. These are both teaching and correctness
issues: a misleading example makes the next concept harder to understand.

At activation, the agent supplies one exact test command that exercises Nick's
edit. Running the printed reference is optional and is labeled separately.
After review, one browser observation connects the change to the world. If Nick
gets stuck, revise the point where his model diverges from the code; do not
respond by assigning a whole prerequisite chapter or expanding the feature.

This approach is a revisable design for this learner. Actual session feedback
should determine whether the next pass removes explanation, adds a trace, makes
the edit smaller, or gives a more complete demonstration.
