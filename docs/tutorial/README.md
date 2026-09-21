# Building Moss, one explainable change at a time

**Start with [Chapter 1A: species settings](01-species-energy.md#checkpoint-a--describe-the-settings).**
Add the configuration type and its defaults, run the existing simulation tests,
then stop for review. You do not need to read ahead to begin.

Fern and Flint both start with 60 energy. Step three times and both have 57.
That is the rule you wrote, running in the browser and in a renderer-free test.
Our next change lets a fox and a hare pay different passive costs. From there,
we can ask what a hungry animal notices, how it reaches food, and what happens
when two animals reach the same finite patch.

This guide follows those questions through the actual project. The coding path
stays small. Optional context pages explain the surrounding Rust, ECS, and
ecology ideas when you want more of the picture.

## Find your place

- [The coding chapters](#the-coding-chapters)
- [Context when you want it](#context-when-you-want-it)
- [Reading and running in RustRover](#reading-and-running-in-rustrover)
- [How the guide grows](#how-the-guide-grows)

## The coding chapters

**Ready to start:** Chapter 1A. The later checkpoints and chapters require the
agent preparation named on their pages. They describe future behavior, not
features already running in Moss. [NOW.md](../../NOW.md) records your current
edit and stopping point.

1. **[Species energy settings](01-species-energy.md).** The reserve belongs to
   the animal; the rate belongs to shared configuration. Keep the existing
   default, then prove that three ticks can leave the hare at 57 and fox at 54.
2. **[Authored populations](02-populations.md).** Keep the diagnostic chamber,
   add six hares, two foxes and four grass patches, and inspect species totals.
   This is agent scaffolding, followed by a short test walkthrough together.
3. **[Choosing nearby food](03-food-choice.md).** Give a hungry grazer a target
   without moving it. A stable tie rule makes an otherwise ambiguous choice
   explainable and repeatable.
4. **[Paying to move](04-movement.md).** Turn the target into an affordable
   one-cell step. Charge for the accepted distance, after passive maintenance.
5. **[A finite meal](05-eating.md).** Transfer only the food that exists and
   the energy that fits. Two grazers must share the actual remaining biomass.

Each chapter supplies code placement, a worked example, an observable
checkpoint and a review stop. I prepare the mechanical prerequisites and
reconcile later instructions with your current code before you reach them.
The guide does not require you to manage those dependencies.

## Context when you want it

The [context shelf](context/README.md) is optional. These pages answer questions
you can carry back to the code:

- **[Follow one Step through Moss](context/a-tick-through-moss.md):** where the
  button becomes a rule, and why the browser and native tests agree.
- **[Rust at the point of use](context/rust-at-point-of-use.md):** references,
  mutable bindings, loops, tuple patterns and `Option` in this codebase.
- **[ECS through Fern and Flint](context/ecs-in-moss.md):** components, resources,
  queries, schedules and the different meanings of identity and species.
- **[From a meal to an ecosystem](context/from-meals-to-ecosystems.md):** why
  finite food comes before sunlight, growth, weather and population experiments.

## Reading and running in RustRover

Open this file in the Project tree under `docs/tutorial`. Use **Preview** for
reading or **Editor and Preview** when comparing prose and code. Chapter pages
have a local contents list and links back here; the files also remain readable
as plain Markdown. [The reading setup](reading-in-rustrover.md) describes the
layout and navigation without requiring a theme or diagram plugin.

All shell commands assume the workspace root, `/Users/nick/Code/moss`. Use the
existing **Run** configurations or a terminal command printed beside the
checkpoint. Native Debug has a known stall on this machine; normal test Run
works. [The IDE guide](../RUSTROVER.md) keeps those setup details in one place.

When a focused test runs, check its name and result. Zero matching tests is
not a green checkpoint. A separate line saying zero *doc-tests* is normal.
The chapter explains any intended red result and the edit that should make it
green. For an unexpected failure, send the output with your current edit.

Once green, send the chapter's review request. I will review the rule and Rust
usage, handle broader checks and browser plumbing, and update `NOW.md`.
A helper test proves its calculation; a completed behavior also needs the real
installed schedule and a browser observation. Those are different kinds of
evidence, and the chapters label them separately.

## How the guide grows

The chapters use the [TutorBro Tutorial Writing skill](https://chatgpt.com/skills?skill_id=6ab16fad9a7881919144924b499536e8):
keep a familiar example while one idea changes, explain why each change is
needed, and check its effect before continuing. The source skill and both
writing references were read for this revision.

Plans remain in [ECOLOGY_PLAN.md](../ECOLOGY_PLAN.md); the broader journey remains
in [LEARNING_PATH.md](../LEARNING_PATH.md). As those change, the agent updates
the relevant chapter and its evidence, preserving existing links where possible.
The [authoring notes and template](authoring/README.md) make that maintenance
repeatable. You do not need to read them to use the guide.

The [verification record](authoring/verification.md) distinguishes tested examples
from future acceptance checks. The source code is the running implementation;
a worked example in this guide does not install a biological rule.
