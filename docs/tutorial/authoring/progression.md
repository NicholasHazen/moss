# Help a programmer take ownership of Moss

[Authoring contract](README.md) · [Proposed learning route](../path/README.md) · [Research and limits](../../research/learning-design.md)

This is the agent's lesson-design working reference. Nick should not need to
maintain a curriculum, complete a skills inventory, or read this page before
coding. His active edit stays in `NOW.md`. The route is a proposal we improve
through actual work, not a fixed course whose page count defines success.

## Two kinds of progress

A new behavior and a new ability can emerge from the same edit. A bounded meal
makes arrival worthwhile; tracing its two mutable arguments explains why both
the animal and patch change. A maintenance refactor makes individuals differ;
reading its query explains why a missing component can silently exclude one.
Plan both outcomes, then use one concrete example to connect them.

The learner is an experienced programmer, so do not equate unfamiliar Rust syntax
with unfamiliar programming. Explain the specific language or ECS mechanism that
blocks the next decision. Conversely, a correct copied body does not establish
that a query, test fixture or schedule around it is understood. During review,
follow one value through the actual changed code together. Keep the discussion
inside the useful work rather than adding a separate oral examination.

## Abilities that recur across the route

| Ability to develop | Where to practice it in the existing work |
| --- | --- |
| Trace ownership and mutation | Movement and the meal change borrowed values. Session 06 constructs owned costs. Session 14 collects owned handles before mutation; 16 owns a snapshot. Follow which values survive the function call. |
| Read ECS access and eligibility | During the first movement review, follow Fern through the prepared adapter. Session 04 shows systems communicating through a target. Session 07 predicts which component combinations match; 16 contrasts required and optional targets. |
| Choose evidence that separates explanations | Session 02 reverses contenders; 03 separates distance from ID and makes different distance metrics disagree; 05 uses a fractional mean; 08 holds travel constant. Explain what wrong rule would produce a different result. |
| Locate the responsible layer | A helper can be correct while its adapter skips an entity or its system is unscheduled. Compare one helper result, one installed tick and one inspector field before changing arithmetic. |
| Investigate and revise a model | Observe the first foraging loop, then make a controlled comparison. Growth and daylight distinguish stored food from supply. Scarcity distinguishes a broken account, an unsustainable scenario and an uninteresting experience. |

These are recurring opportunities, not completion boxes. No single check proves
general proficiency. Record only the obstacle or insight actually encountered in
the existing session note, then adapt the next explanation. Avoid inventing a
diagnosis such as “doesn't understand ownership” from one compiler error.

## Let assistance follow the difficult part

At activation, recommend one starting point. If the mechanism is new, trace a
complete worked case before editing. If the mechanism is familiar but its Rust
expression is rusty, name the small decision and leave the surrounding signature
and test ready. If both are comfortable, lead with the behavior and let Nick write
the short body before consulting the answer. Keep the complete answer accessible
in all cases. A session number never automatically earns the removal of help.

For example, growth revisits the meal's capacity calculation. Start from the
99-of-100 case and ask for the actual retained amount before expanding the code.
If that is already clear, go straight to the prepared helper and its test. If
the return value is confused with the requested rate, use that one trace again.
Do not require a fresh tour of every `u32`, `let` and function parameter.

A small change of input can check whether the idea travels: make the winning
food patch empty, use an explicit zero override, or remove a required component.
Prefer replacing a redundant example to appending another task. Give the answer
nearby. This is a useful way to reason together, not a condition for access to
the next feature.

## Make agent-owned integration legible

Scaffolding ownership means Nick does not have to type every adapter. It should
not make the important connections invisible. After a helper is reviewed, show
one actual entity entering its adapter, the components borrowed or copied, and
the subsequent system that reads the result. The existing
[movement adapter walkthrough](../context/ecs-in-moss.md#a-helper-reaches-the-world-through-an-adapter)
provides the first example without activating it early.

Use the same representative entity and numbers as the live review. If it does
not move, distinguish missing target, empty patch, insufficient reserve and an
unscheduled system. These lead to different edits even when the screenshot is
identical. Reveal the one relevant connection; do not turn a review into a tour
of every module or a requirement to rewrite working plumbing.

## Replan from a world worth watching

At an arc boundary, the agent prepares one short autonomous sequence alongside
the focused regression. Name the transition worth observing, pause there, and
connect it to a rule or component. Let the result inform the next recommendation.
The first foraging loop may expose a target-switching problem, an unfair food
priority, or a need for a second feeding role. Each is more informative than
assuming the next numbered page must be the best use of the next session.

Preserve actual dependencies. Our proposed pursuit/contact hunt needs a way to
pursue and exclusively claim one prey; a simpler in-range consumption rule could
start without pursuit. Neither needs sunlight. Individual maintenance comparisons need a
controlled opportunity; they do not need a universal attribute system. A shortage
can be studied before implementing death. When changing the route, distinguish
these necessary capabilities from the order chosen to introduce them.

Keep one recommendation in `NOW.md` after review. Put alternative directions in
the existing design documents, not a menu Nick must project-manage. Proposing a
better route never authorizes implementing its paired biology. Today's exercise
remains unchanged while this teaching plan develops.

## Checkpoints are evidence for the next revision

Review passes can establish that examples run, links work and claims agree.
They cannot establish that the learning experience is finished. During this
open-ended refinement goal, use each pass to identify the next concrete teaching
question. Keep useful explanations; remove repetition; test proposed examples;
and be explicit about improvements that remain hypotheses until a real session.

The next useful evidence is often small: which line required a reminder, whether
the diagram clarified access, or whether the observed world raised a question
Nick wanted to pursue. Do not demand a feedback form or a learning score. Use
what he says and does in the work, and remain willing to revise the design.
