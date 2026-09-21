# Moss — bootstrap my workbench, then help me build

You are my lead pair-programming agent for **Moss**, a new browser-first Rust/ECS ecosystem sandbox. This folder contains its project brief and working instructions.

My goal is to build a living world and regain hands-on fluency in Rust/ECS. I want you to remove mechanical friction, not deliver the whole simulation while I watch. I am an experienced software engineer but rusty on Rust; I completed Rustlings and most of the Rust book and read Bevy ECS documentation a couple of years ago.

I benefit from a clear next action, small examples, visible progress, and help maintaining the path. Be a thought partner, practical mentor, and constructive reviewer. Avoid long framework surveys, giant unexplained patches, or asking me to plan the whole project before we start.

## Read and orient

Read `AGENTS.md`, `PROJECT_BRIEF.md`, `NOW.md`, and `docs/BOOTSTRAP.md`. Consult `docs/ARCHITECTURE.md`, `docs/BROWSER_EXPERIENCE.md`, and `docs/RESEARCH.md` for the technical work. Use `docs/WORKING_TOGETHER.md` and `docs/LEARNING_PATH.md` to prepare the handoff.

Inspect the actual workspace, existing instructions, local tools, and any uncommitted work. This kit alone does not contain an initialized Rust project. Do not assume Aftermarket code or documents are present, and do not import its desktop, economy, SQLite, or advanced cognition requirements.

Give me a brief statement of what you will scaffold and what you will leave for me. Then perform the authorized bootstrap; do not stop at a plan. Make reasonable reversible choices and record them. Ask only for genuinely blocking permissions or decisions, not every minor preference.

## Your authorized work

Set up the smallest practical Rust workspace with `moss-sim` and `moss-web`, following the one-world design in the architecture note. Verify current official APIs and dependency compatibility; pin and record a set that actually builds rather than guessing versions from memory.

Build a browser shell with a small bounded scene, a food patch, and one or two identifiable creature placeholders. Include real simulation IDs and positions, a fixed starting energy reserve for the first exercise, a simple selected-entity inspector, and a real tick counter. Make clear that the placeholders are not autonomous yet.

Implement the mechanical view and playback work: canvas sizing, map-style pan and pointer-anchored zoom, Fit world, click-versus-drag selection, Pause/Play, Step, and Reset. Handle hidden-tab suspension without offline catch-up. Keep camera/UI state separate from the biological rules.

Wire native simulation tests and a WebAssembly build, a local browser run path, minimal error reporting, and a small bounded journal of actual startup/initialization events. No database or analytics platform is required. Use the environment's normal installation permissions and trusted official package sources. Do not publish or deploy anything publicly.

Keep patches understandable. Use concrete functions and types; do not create a generic framework, many empty modules, an LLM controller, or a second mirrored simulation.

## Use helpers where they help

If actual subagents are available, you may use at most two bounded helpers for web compatibility, build/test review, or mechanical scaffolding. Use `prompts/SUBAGENT_TASKS.md` as a guide. Give them distinct responsibilities and file ownership; you integrate and explain the result.

Do not delegate creature behavior or my reserved exercise behind the scenes. Do not pretend independent agents ran if the environment does not support them.

## Verify and be precise

Run formatting, relevant lint checks, native tests, and the actual WebAssembly build. Exercise the browser controls if you have a real browser tool. Record exactly what you ran and observed, including browser/version and selected graphics backend.

A native compile is not a browser test. A browser you cannot launch is an unverified runtime target, not a pass. If access or installation blocks part of the work, preserve a useful checkpoint, explain the exact blocker, and leave one actionable next step.

Update the README with the exact verified commands and working directories. Create `docs/BUILD_NOTES.md` with resolved versions, feature flags, checks, and limitations. Update the decision log only where an actual choice changed the proposed defaults.

## The boundary that matters

**Do not implement energy depletion, seeking food, grazing, resting, breeding, hunting, progression, genetics, or a utility framework during bootstrap.** Do not hide the first exercise's solution in a helper function. Do not create a fake animation that looks like a creature is already making decisions.

The first learner-owned edit is: **make one creature's energy reserve decrease once per complete simulation tick, bounded at zero, and see the change in the inspector.** We will then build toward food seeking and eating in small paired steps.

You may provide the data shape, code location, a test outline, and a small relevant syntax example without completing that behavior. Keep the existing application runnable; do not leave reachable placeholder panics.

## Finish with a handoff I can actually use

Show the browser result or report its exact verification limit. Give me a short tour of no more than five important code locations. Explain where state lives, how a tick runs, and how the inspector reads it.

Replace `NOW.md` with the single next learning card: actual file/function, intended change, why it matters, one expected observation, and a stopping point. Add a short factual session-log entry. Keep later ideas in the parking lot rather than adding them to today's task.

End with the smallest concrete action for me to take. Do not continue implementing the biological loop after the scaffolding is complete. If I later ask you to demonstrate or take over a slice, help directly and explain it; learning support is not a rule against giving answers.

**Build the workbench, make the next step inviting, and leave me a real part of the world to bring to life.**
