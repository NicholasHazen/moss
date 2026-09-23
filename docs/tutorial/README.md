# Building Moss, one explainable change at a time

**Start with [today's v2 lesson: give Fern one affordable step](today-v2.md).**
[NOW.md](../../NOW.md) keeps the active edit and stopping point.
This guide explains how each small change works and how to check it. The
[documentation index](../README.md) leads to setup, design, research and history.

For the weeks after today, use [the short-session learning path](path/README.md):
16 guides organized into four adaptable arcs, with worked examples, returning
reminders and visible results. The intended focus block is 20–30 minutes; actual
progress and useful questions can change the route. V2 keeps today's movement
exercise and brings its explanation, worked answer and review into one page.

Fern and Flint both start with 60 energy. Three ticks at the default rate leave
57. The next visible result is Fern walking to Meadow, then eating a finite meal.
We introduce autonomous choice and individual differences after that loop can
be observed. The short-session path owns that future sequence; the older chapter
filenames remain available as references for existing bookmarks.

## Find your place

- [The coding chapters](#the-coding-chapters)
- [Context when you want it](#context-when-you-want-it)
- [Reading and running in RustRover](#reading-and-running-in-rustrover)
- [How the guide grows](#how-the-guide-grows)

## The coding chapters

Use one route. [Today's v2 lesson](today-v2.md) connects the active movement edit to a
browser result and an optional meal stretch. After that, the agent resumes the
first unfinished outcome in the [short-session path](path/README.md), preparing
its plumbing before your coding block. Reading ahead does not activate a task.

| Visible progress | What you learn |
| --- | --- |
| **Completed reference:** [species maintenance](01-species-energy.md) | Revisit the rule already running: components hold reserves, a resource supplies rates, and a query updates animals. No edit is assigned here. |
| **Today:** [an affordable step, v2](today-v2.md) | Copy a proposed position, borrow real values mutably, validate before mutation, then connect a helper to ECS. |
| **Next / today's stretch:** [a finite meal](05-eating.md) | Transfer bounded quantities; account for actual consumption and competing eaters. Agent preparation follows movement review. |
| **After today:** [sixteen short sessions](path/README.md) | Develop finite food, autonomous foraging, individual costs, growth, daylight and an explainable response to scarcity. |

If today's stretch already covers a meal checkpoint, keep the reviewed work;
the agent skips that duplicate exercise. `NOW.md` remains the one place recording
your actual next edit. The [checking guidance](path/README.md#checking-your-work)
distinguishes testing that edit from running a guide's complete printed answer.

The stable [food-choice chapter](03-food-choice.md), [population chapter](02-populations.md)
and [individual-cost chapter](01d-energy-profiles.md) explain integration and
design beside the corresponding short sessions. They are companions, not a
second curriculum to complete. Their opening links take an old bookmark to the
current lesson. The broader [conceptual map](context/learning-path.md) connects
later ideas without making them prerequisites for visible progress.

## Context when you want it

The [context shelf](context/README.md) is optional reading:

- [Follow one Step through Moss](context/a-tick-through-moss.md) connects the
  browser button, rule and native test.
- [Rust at the point of use](context/rust-at-point-of-use.md) explains modules,
  public paths, borrowing and query syntax.
- [Read a test as a small experiment](context/reading-a-test.md) follows the
  maintenance regression you already have, including how to interpret its result.
- [ECS through Fern and Flint](context/ecs-in-moss.md) connects components,
  resources, queries, schedules and identity.
- [Attributes and defaults](context/attributes-and-defaults.md) explores
  templates, individual values and future temporary effects.
- [From a meal to an ecosystem](context/from-meals-to-ecosystems.md) connects
  finite food to growth, daylight and population experiments.
- [The learning path](context/learning-path.md) shows the broader conceptual
  progression; the [ecology plan](../design/ecology.md) holds model decisions.

## Reading and running in RustRover

Open this file under `docs/tutorial` and use **Preview** or **Editor and Preview**.
[The reading setup](reading-in-rustrover.md) explains navigation. Use the exact
checkpoint command from the repository root; [development commands](../development/README.md)
and [the IDE guide](../development/rustrover.md) cover setup and run configurations.

Check that the named test ran: zero matches is not green. Send an unexpected
failure with your edit. Once green, send the active guide's review request; the agent
reviews the rule, completes remaining checks and updates `NOW.md`.

## How the guide grows

The chapters use [TutorBro](https://chatgpt.com/skills?skill_id=6ab16fad9a7881919144924b499536e8):
keep a familiar example, explain why a change matters and check its effect.
The agent maintains chapters and links using the
[authoring contract and template](authoring/README.md).

[Current example verification](authoring/verification.md) distinguishes executed
checks from future acceptance. A helper test does not prove installed-schedule
behavior, and a native test does not prove browser behavior. Writing an example
in this guide does not install its rule into Moss.
