# Bootstrap — build the workbench, leave room to learn

**Historical:** original bootstrap instructions from September 20, 2026.
The foundation and first maintenance exercise are complete. This document
does not authorize repeating setup or replacing the current task. Resume through
[NOW.md](../../NOW.md) and the current [agent instructions](../../AGENTS.md).

**Audience:** The local coding agent.  
**Scope:** First runnable browser foundation and first learning handoff.  
**Starting status:** This kit contains documents only; inspect the actual repository before assuming anything else.

## Goal

Remove setup friction so Nick can begin writing a small creature rule in an application he can see and understand. Complete the infrastructure, then stop before implementing the living behaviors.

## Establish the environment

Read `AGENTS.md`, `PROJECT_BRIEF.md`, and `NOW.md`. Inspect existing files, repository status, local instructions, operating system, available Rust tools, and browser-test capabilities. Preserve unrelated work. Do not assume that Aftermarket files or code exist here.

Verify official dependency/API references in `docs/research/README.md`. Pick one compatible stable Rust/Bevy/WebAssembly toolchain; pin and record the combination that actually builds. Keep Bevy and directly used `bevy_ecs` versions aligned. Prefer minimal web/2D features over pulling in irrelevant rendering, audio, or platform services.

Use the environment's permitted installation flow. Do not run opaque remote scripts, perform privileged changes without permission, or silently switch away from Rust/ECS when setup hits friction.

## Build a small real foundation

The proposed layout is:

```text
Cargo.toml                   Small workspace; shared dependency choices
Cargo.lock                   Resolved application dependencies
rust-toolchain.toml          Verified toolchain and required target
crates/moss-sim/              Simulation library and native tests
crates/moss-web/              Browser app, minimal HTML/CSS, bundler config
```

Use `moss-sim` for a validated small world configuration, stable IDs, authored fixture initialization, explicit ticking, and a small bounded event journal. Use `moss-web` for the Bevy app, display, controls, browser lifecycle, and inspector.

Follow the single-world starting design in `docs/design/architecture.md`. Use native tests to run the simulation schedule without opening a window. Avoid a second ECS mirror, generic adapters, or unused future modules.

The first fixture has a bounded chamber, one or two distinguishable creature placeholders, and a food patch. Give them real identities and positions; creatures may have a fixed starting energy reserve for the first lesson. They are clearly labeled as **not autonomous yet**. Do not animate a fake grazer that appears to have behavior it does not possess.

## Make the browser useful

Implement a canvas that resizes with its container, drag-to-pan, pointer-anchored zoom within limits, and Fit world. Selection should distinguish clicks from drags. A small inspector reads identity, kind, position, and any implemented starting state.

Provide Pause/Play, Step, Reset, and a tick display. Initially it is fine that only the clock changes: biological maintenance belongs to Nick's first exercise. Reset restores the authored fixture and starts a fresh run record. Camera and selection are presentation state.

On page hide, pause without building a time backlog; on return, remain paused until resumed. Keep browser input out of panels when it belongs to the map and vice versa. Show a useful loading or startup error outside the renderer.

Use Trunk as the proposed dev server/bundler, with a verified invocation and static asset output. Do not deploy publicly. Test the actual browser target rather than substituting a native window and declaring web support complete.

## Wire only honest observation

Emit real startup/initialization events with participant IDs. Display a small recent-event view and explicitly bounded retention. Do not manufacture births, eating, decisions, or movement for the demo.

No database, full save/load, graph UI, genealogy explorer, or charting framework is needed in bootstrap. The interfaces should not prevent later inspection; they do not need to implement the whole observability document.

## Verify the infrastructure

Run formatting, relevant lint checks, native simulation tests, and a WebAssembly build using the selected features. Check the real dependency graph for mismatched Bevy ECS versions or unnecessary platform dependencies.

At minimum, test fixture initialization, ID uniqueness, invalid world dimensions, and tick/reset semantics. Reserve the energy-depletion behavior and its final test implementation for the learning step; do not hide a completed solution in a helper function.

Exercise startup, resize, pan, zoom, selection, pause/step/reset, and hidden-tab return in a real browser when tools permit. Record browser/version and which checks were observed. If a browser is unavailable, label runtime checks unverified and give Nick a short manual path; do not claim a smoke-test pass.

Build and serve a release bundle locally where possible. A working dev server is not proof that asset paths work in the static bundle. Subpath hosting can remain a documented follow-up if it is not tested.

## Produce a small handoff, not another planning project

Update `README.md` with exact verified setup/run/test commands and their working directories. Add `docs/development/verification.md` with the resolved versions, enabled features/backend, actual commands/results, platform/browser coverage, and remaining limitations. This file does not exist until there is real evidence to record.

Replace `NOW.md` with one learner-owned task: decrease a creature's energy reserve once per simulation tick, bounded at zero. Name the actual file/function to edit, one expected observation, and a clear stopping point.

Provide a brief code tour with at most five important locations. Explain one component, one query, and where a simulation tick is requested. Keep the browser plumbing available for review without requiring Nick to understand all of it before making his first change.

## Stop here

Do **not** implement grazing, food seeking, rest, reproduction, predation, progression, genetics, or a utility framework. Do not pre-solve the first energy exercise in another file. Do not leave reachable `todo!()` calls that crash the running app.

After verified scaffolding and the learning handoff, stop. The completed task is a usable workbench with an obvious first meaningful edit—not a completed life simulator.

If the environment blocks part of the work, preserve what runs, report the exact blocker, and leave a single actionable next step. Do not replace an environmental blocker with a speculative rewrite.
