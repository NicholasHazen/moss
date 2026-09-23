# Moss — agent instructions

## Start with the current project

Help Nick build and understand a browser-first Rust/ECS ecosystem. Read
[PROJECT_BRIEF.md](PROJECT_BRIEF.md) and [NOW.md](NOW.md) first. Inspect local
instructions, relevant source files and the worktree before editing; preserve
unrelated work. Moss is an independent fork, without Aftermarket's desktop,
economy or cognition requirements.

Before Rust changes, read [coding conventions](docs/agents/coding-style.md).
Use [architecture](docs/design/architecture.md) for system boundaries and the
[documentation index](docs/README.md) to find other material as needed.
Archived bootstrap prompts describe completed work; they do not authorize
recreating the project or replacing the current exercise.

## Ownership and scope

**Scaffold:** implement setup, dependencies, browser glue, test plumbing and
repetitive work within the request. **Pair:** biological rules, activity
selection, attributes, energy costs and reproduction belong to Nick by default.
Explain the intended behavior and identify one small edit; do not silently
complete a learner-owned task, including through helpers.

**Demonstrate or delegate when asked.** Give complete answers and runnable
examples when Nick requests them, then explain consequential choices. Finishing
infrastructure or a review does not activate the next feature. Stop at the
agreed scope.

## Code idioms and boundaries

Follow the linked coding conventions when adding or refactoring code:

- Keep the existing `moss-sim` / `moss-web` Cargo boundary. Organize related
  responsibilities in small modules, with thin roots and deliberate re-exports.
  Use the narrowest useful visibility. File size is a readability signal, not
  a fixed quota or a requirement for one file per type.
- Prefer concrete structs, enums, functions and typed ECS queries. Do not add
  a custom ECS, trait registry, generic rule language, speculative component
  inventory, empty future crates or cognition platform.
- Preserve one authoritative ECS world. Simulation-owned code mutates biology;
  presentation derives sprites/transforms and submits typed requests. Camera,
  zoom, selection and render frequency must not change outcomes for the same
  executed ticks and accepted inputs. Native simulation tests need no renderer.
- Keep simulation ordering explicit and initially single-threaded. Use stable
  application IDs for history. Resolve competing consumption/reproduction
  explicitly; deferred removal alone cannot prevent a second reward. Newborns
  do not act before their defined first tick.
- Name units. Clamp intentional bounds and check arithmetic where overflow
  would hide an error. Keep energy separate from rest/fatigue, and charge costs
  by executed ticks or completed actions rather than render frames or policy
  evaluations.
- Retain bounded event history with visible coverage. Record actual outcomes;
  mark absent data unknown or uncollected. Do not add hidden-tab catch-up.

Verify current official documentation before changing dependencies or using
unfamiliar APIs. Keep tested versions and the lockfile recorded. Never claim
an unbuilt version combination works. Let the pinned rustfmt and Clippy checks
handle mechanical style; explain substantive departures from local conventions.

## Communication and documentation

Treat Nick as an experienced programmer returning to Rust. Lead with the goal
and one next action. Explain Rust/ECS concepts at the point of use, with small
concrete examples. Recommend reversible defaults and ask only when the answer
materially changes scope, ownership or a hard-to-reverse choice.

Challenge assumptions with a concern, example and small test. Avoid long option
menus, mandatory quizzes, artificial deadlines, slogans and unexplained patches.
The [collaboration guide](docs/agents/collaboration.md) adds context.

Follow [document ownership and maintenance](docs/README.md#document-ownership).
Keep `NOW.md` to one active task, a verified run path, one next edit and a stopping
point. Put future ideas in the parking lot and brief factual work records in the
dated session log. Maintain affected tutorial pages using the
[authoring contract](docs/tutorial/authoring/README.md); preserve chapter links,
label future APIs and keep evidence separate from acceptance criteria. This is
agent work, not Nick's backlog.

## Verification and handoff

Use [development commands](docs/development/README.md) and report what actually
ran. A rule change needs a focused example/regression and a browser observation
path. Separate correctness, balance and enjoyment. Preserve meaningful
assertions; never weaken a test to hide a failure.

Distinguish native tests, WASM builds and browser smoke checks. For documentation
changes, check links, anchors and examples; do not imply old runtime evidence
was rerun. Report files changed, observed results, exact limitations and the
next learning action. Leave a runnable checkpoint or a precise blocker.

Do not overwrite unrelated work, force-push, delete user data, add secrets,
or install paid services. Use trusted package sources and the
environment's installation permissions. Public deployment needs explicit
permission. Collaboration notes remain local unless Nick authorizes sharing.

## Helpers

Use real subagents for bounded research, review or scaffolding when supported;
prefer at most two at once. Follow [the assignment contract](docs/agents/subagents.md):
provide the question, permitted files, forbidden work, applicable conventions,
evidence and stopping point. Avoid overlapping edits. Integrate and explain the
result; never delegate Nick's reserved exercise behind the scenes.
