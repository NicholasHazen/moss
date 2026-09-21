# Session log

Keep entries short and factual. The agent maintains this so Nick can return without rebuilding context.

## September 20, 2026 — starter kit

**Changed:** Created the independent Moss vision, collaboration rules, technical starting direction, learning path, and bootstrap prompt.

**Evidence:** This is a documentation handoff. No Rust workspace, executable, browser build, or simulation test has been produced by the kit.

**Next:** Run the local bootstrap prompt, then hand off the first energy-reserve rule to Nick.

## September 20, 2026 — runnable browser foundation

**Changed:** Added the two-crate Rust workspace, aligned Bevy/ECS 0.18.1 lockfile,
authored fixture and bounded journal, single-thread tick schedule, WebGL2 view,
HTML controls/inspector, native tests, Trunk run path, and checked build notes.
Two bounded helpers handled stack/shell work and simulation scaffolding/review.
No energy depletion or later biological behavior was implemented.

**Evidence:** `scripts/check.sh` passed (10 native tests, format, native/WASM
Clippy, JS syntax). Optimized WASM and Trunk release builds passed. In-app
Chromium rendered and exercised selection, playback, pan/zoom/Fit, reset and
resize. A separately served release ran in Chrome; real tab hiding returned
paused with no catch-up and Step advanced once. Full coverage is in
`BUILD_NOTES.md`.

**Learned:** Per-process Command Line Tools selection avoids the local Xcode
license blocker. A real resize test caught a drawing/input pixel-scale mismatch;
using the same CSS dimensions for both fixed it. In-app tab visibility is not a
substitute for a real Chrome tab-hide test.

**Next:** Pair on Fern's one-unit-per-tick energy decrease, bounded at zero.
The concrete edit and stopping point are in `NOW.md`.

## September 20, 2026 — foundation and learning-path review

**Changed:** Two bounded read-only reviewers checked the code and next learning
steps. Replaced string creature roles with an enum, moved Meadow's name into
simulation data, and fixed entity-list keyboard selection losing focus. Updated
the README tour, architecture status, decisions, and current work card. The
first paired rule will apply to both creatures; Fern is the one to watch.
Food renewal is now an explicit later slice after finite consumption.

**Evidence:** `scripts/check.sh` passed all 10 native tests, formatting,
native/WASM Clippy, and JS syntax. The Trunk release rebuilt successfully.
In-app Chromium verified preserved keyboard focus, selection of all three
entities, Step with unchanged reserves/biomass, and Reset. No biological
behavior or its reserved regression was implemented.

**Next:** Pair on one unit of maintenance per tick for entities with `Creature`
and `Energy`, bounded at zero. `NOW.md` has the edit and observation.

---

## September 20, 2026 — identity, ecology plan, and code landmarks

**Changed:** Responded to Nick's naming/scale feedback with separate species and
ecological-role components, species-first labels, and equal schematic markers
with consistent picking/outlines. Moved fixture data into `fixture.rs`; added
four documented, compiled, unimplemented/unscheduled stubs in `lessons.rs`.
Two read-only helpers reviewed the model and teaching scaffold. Recorded a
concrete population/plant/geometry/daylight/weather plan in `ECOLOGY_PLAN.md`,
and updated the entry points and current work card. No biological rules added.

**Evidence:** All 10 native tests and format/native+WASM Clippy/JS syntax checks
passed via `scripts/check.sh`; Rustdoc and Trunk release builds passed. Browser
inspection verified taxonomy, keyboard focus, uniform markers, grass-corner
picking, Step and Reset, with no console errors. Details in `BUILD_NOTES.md`.

**Next:** Pair on `lessons::spend_energy`; after that, scaffold the separate
population fixture and species summaries. CA spread and weather remain later
experiments, not new active tasks.

---

## September 21, 2026 — coding-session checkpoint

**Evidence:** Reran `cargo test -p moss-sim --locked` through the toolchain wrapper:
all five simulation tests passed. Restarted the live-reloading Trunk release
server on localhost:8080 and verified Hare #1 at tick zero, energy 60 / 100 in
the browser. The maintenance stub is untouched.

**Next:** Nick implements `lessons::spend_energy`, then we wire its ordering and
verify the bounded decrement. `NOW.md` records the current server and the agreed
baseline-plus-actual-movement energy direction.

## September 21, 2026 — loop refresher

**Changed:** At Nick's request, added a commented worked energy-loop example in
`lessons.rs`, explaining mutable query iteration and the zero bound. Updated
`NOW.md` with how to activate it. The executable stub and schedule are unchanged;
no death rule or biological regression was added. `cargo fmt --all -- --check`
passed. Recommended keeping starvation/death separate from this first decrement.

## September 21, 2026 — Nick's first energy rule runs

**Changed:** Nick wrote the query loop and fixed the spelling error after the
compiler reported it. The agent preserved that implementation, formatted it,
chained maintenance before tick completion, and updated the browser note and
current docs. A bounded read-only reviewer confirmed filtering and ordering.

**Evidence:** `scripts/check.sh` passed all 10 existing tests and native/WASM
lint checks; Trunk rebuilt the release preview. Browser inspection showed both
reserves at 57 after three Steps, both still at zero at tick 244, unchanged
positions and 80 grass biomass, and Reset restoring 60. No console errors.
The focused native maintenance regression has not been implemented yet.

**Next:** Pair on `maintenance_spends_energy_and_stops_at_zero` beside the
existing fixture tests. `NOW.md` names that single next edit. No later biology
was implemented.

## September 21, 2026 — test API guidance

**Changed:** Nick started the maintenance test, corrected the mutable borrow,
and added three ticks. Added commented examples of `World::query_filtered`,
read-only iteration, entity counts and assertions, preserving his draft.
Clarified that initial reserve is 60 while capacity is 100. Updated `NOW.md`.

**Evidence:** Formatted the code; the bootstrap integration test target compiles
with `--no-run`. No runtime test pass is claimed for this incomplete regression.
**Next:** Nick adds the assertions, then checks the zero boundary beyond 60 ticks.

## September 21, 2026 — first maintenance regression complete

**Changed:** Nick completed the energy assertions and zero-bound check. Reviewed
the test with a bounded read-only helper; no blocking findings. Preserved his
test and commented API reference; the only code change was rustfmt import ordering.
Updated current docs to mark the first paired exercise complete.

**Evidence:** The focused maintenance test passes. `scripts/check.sh` passes all
11 native tests, formatting, native/WASM Clippy, and JS syntax. This test edit
does not change runtime behavior; the browser smoke test was not repeated.

**Next:** The separate authored population scenario and read-only species
summaries are the recommended mechanical slice. No later biology was implemented.

## Entry shape

Date / small goal. What actually changed. One useful discovery. Commands or checks actually run and their result. Any remaining blocker. The single next action, also reflected in `NOW.md`.

Do not turn this into exhaustive chat transcripts, performance judgments about the learner, or a duplicate backlog.
