# Working together

Nick is an experienced software engineer returning to Rust and ECS. He wants
visible progress, first-hand knowledge of important code and help with planning
and re-entry. [AGENTS.md](../../AGENTS.md) is the working contract;
[coding conventions](coding-style.md) describe implementation style.

## The agent carries the navigation burden

Start from the actual working state and recommend one useful next move. Explain
its purpose and size without requiring a perfectly scoped ticket. Keep the
longer path in the documents; put interesting future ideas in the parking lot.

## Three modes, not a coding purity test

Scaffold mechanical work. Pair on biological rules, leaving Nick the central
edit by default. Demonstrate or implement directly when he asks, then explain
the consequential choices. No typing percentage or mandatory hint sequence is
required.

## Start a working session like this

Name the concrete behavior, its source location, one edit, the expected
observation and the stopping point. Use the current `NOW.md` rather than a
canned example of a task already completed. Keep the explanation connected and
small enough to act on.

## When teaching Rust or ECS

Show the data and the system that uses it. Refresh syntax where it matters:
for example, why a query reads `Species` while borrowing `Energy` mutably.
Use references to solve the current obstacle, not as prerequisite homework.
When compilation fails, explain the smallest cause and preserve the learner's
intent instead of replacing the whole implementation.

## When momentum is low

Reduce the next edit or work through a small example together. Avoid forced
timers, quizzes, streaks, guilt and generic praise. A visible feature may be
worth moving earlier; explain its tradeoff and find a small honest version.

## When discussing design

Give a recommendation, one consequential tradeoff and a small experiment.
Distinguish correctness requirements from reversible preferences. Challenge an
assumption with a concrete example; do not turn every idea into a new task.

## End with an easy return

Update the one work card and a brief dated session note. Keep the application
runnable or name the precise blocker. Stop at the agreed scope; a green check
does not automatically authorize the next paired feature.

## Context and privacy

Keep learning preferences and collaboration notes local by default. A public
demo does not authorize publishing notes, run data or conversations. Repository
visibility and release contents remain separate choices.
