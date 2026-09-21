# Verified bootstrap build

September 20, 2026. Working directory for all commands: `/Users/nick/Code/moss`.

## Resolved stack

| Item | Tested value |
| --- | --- |
| Host | macOS 26.6.2 (25G83), Apple Silicon |
| Rust | rustc 1.93.1 (01f6ddf75 2026-02-11), pinned in `rust-toolchain.toml` |
| Cargo | 1.93.1 (083ac5135 2025-12-15) |
| Targets | `aarch64-apple-darwin` tests; `wasm32-unknown-unknown` app |
| Bevy / Bevy ECS | Both exactly 0.18.1; one ECS version in the browser graph |
| Trunk | 0.21.14, preinstalled |
| wasm-bindgen library / CLI | Both exactly 0.2.123 |
| web-sys / js-sys | 0.3.100, resolved through the lockfile |
| wgpu / winit | 27.0.1 / 0.30.13 |
| serde / serde_json | 1.0.229 / 1.0.151 |

Bevy's default features are disabled. The explicit features are `std`,
`async_executor`, `bevy_asset`, `bevy_log`, `bevy_window`, `bevy_winit`,
`bevy_render`, `bevy_core_pipeline`, `bevy_sprite`, `bevy_sprite_render`,
and `webgl2`. The renderer explicitly requests GL; there is no automatic WebGPU
fallback. `Camera2d` uses `Tonemapping::None`, so no LUT assets are needed.
The simulation's direct ECS dependency enables only `std`.

`cargo tree -p moss-web --target wasm32-unknown-unknown` confirms no audio,
PBR, UI, or glTF crates in the active browser dependency graph. Cargo's lockfile
also contains platform/optional resolution entries; those are not a list of
compiled browser features. `Cargo.lock` is included. This folder had no Git
repository at bootstrap; no commit or remote was created.

## Actual checks

- `cargo fmt --all -- --check`: passed.
- `scripts/with-toolchain.sh cargo test --workspace --locked`: 10 passed
  (5 simulation; 3 playback; 2 camera mathematics).
- `scripts/with-toolchain.sh cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `scripts/with-toolchain.sh cargo clippy -p moss-web --target wasm32-unknown-unknown --locked -- -D warnings`: passed.
- `scripts/with-toolchain.sh cargo build -p moss-web --target wasm32-unknown-unknown --release --locked`: passed.
- `scripts/check.sh`: passed, including JavaScript syntax validation.
- `scripts/with-toolchain.sh trunk build --locked --release`: passed.
- `scripts/with-toolchain.sh trunk serve --locked --release`: served the real
  browser application on `127.0.0.1:8080` after local-listener permission.
- `python3 -m http.server 8081 --bind 127.0.0.1 --directory dist`: served the
  separately built static release after stopping Trunk. It loaded in regular
  Chrome and the in-app browser. The static HTML has no development websocket.

The tests establish authored fixture contents/unique IDs/bounds, rejected
dimensions, tick/reset semantics, preservation of presentation-only state,
journal eviction coverage, bounded playback, hide-and-return backlog disposal,
and pointer-anchored zoom. They do not establish future biological correctness
or balance. Energy-depletion behavior and its regression are deliberately
reserved for the first paired edit.

## Environment fixes

The first dependency fetch could not resolve crates.io under the sandbox;
the approved network-enabled `cargo fetch` succeeded from the standard registry.
No tool installation was needed.

The initial native and WASM checks failed before compiling Moss because the
system-selected Xcode requires license acceptance. The existing standalone
Command Line Tools works (`Apple clang 17.0.0`). Commands are wrapped with a
per-process `DEVELOPER_DIR=/Library/Developer/CommandLineTools` on macOS.
No license was accepted and no global developer-directory setting changed.

A first WASM check exposed Bevy's component derive path needing a direct
`bevy_ecs` dependency in the browser crate's target-specific dependencies.
Adding the same exact 0.18.1 dependency resolved it; final lint/build checks pass.

Trunk 0.21.14 rejects the environment's `NO_COLOR=1` because it expects a boolean
word. The wrapper normalizes that value to `true`. Local TCP binding was denied
under the sandbox; the approved localhost-only server commands succeeded.

The real resize test exposed a camera/input scale mismatch at a changed device
pixel ratio. The camera now uses `ScalingMode::Fixed` with the canvas's current
CSS dimensions and the shared units-per-pixel scale. Both drawing and hit testing
therefore use the same coordinates. Resizing between 1280 × 800 and 620 × 780,
then clicking the drawn creatures, was verified after the fix.

## Browser evidence

Codex in-app Chromium reports Chrome/153.0.0.0. Bevy reported `backend: Gl`,
WebGL 2.0, and ANGLE Metal on Apple M4 Max. Startup, the bounded grid and all three
shapes rendered. Fern's inspector showed ID1, cell (10,10), energy60/100;
Flint showed ID2, (21,6), energy60/100; initialization had four events.

Three Step clicks produced tick3 while paused; energy remained unchanged.
Play advanced the clock, Pause held it at tick19 through camera interactions.
Clicking a shape selected it. A 60px/-40px drag moved the drawing by that amount
and preserved selection and authoritative position. Wheel zoom from16.8 to27.8
pixels/cell kept Fern under the pointer. Fit world restored the whole chamber.
The inspector journal scroll operated independently of canvas input.
Reset restored the fixture, cleared selection, changed run1 to run2, replaced
history, and a subsequent Step showed tick1.

Regular Google Chrome **152.0.7977.83 (arm64)** also loaded the static release
(running version verified at `chrome://version`, distinct from a pending update
on disk). While running, opening a
different Chrome tab hid Moss; closing the disposable tab returned to tick2 with
“Paused after hiding.” It stayed at2 while selecting Fern, and Step advanced to3.
There was no catch-up burst. The in-app browser's visibility/tab controls did not
emit a page-hide signal, so they were not used as evidence for this check.

The initial bootstrap release WASM was 34,684,450 bytes before HTTP compression. Trunk's optional
Binaryen optimization is disabled (`data-wasm-opt="0"`) to use the installed
toolchain without another download. This is a functional local checkpoint, not
a download-size or startup-performance target.

Bevy emits an expected WebGL capability warning that order-independent
transparency is unavailable, and uses CPU preprocessing. This scaffold draws
opaque 2D sprites; no browser application errors were observed during these checks.

The paused static release with Fern selected is captured in
[the checkpoint screenshot](screenshots/bootstrap.png).

## Foundation review verification — September 20, 2026

Two read-only helpers reviewed the implementation and learning path. The code
review independently reran the 10 native tests. The lead reproduced keyboard
selection dropping focus to `BODY`, then changed the entity list to preserve its
buttons when only selection changes. Creature roles now use a concrete enum,
and Meadow's name lives in `FoodPatch` rather than being invented by the browser.
The existing fixture test also checks that name. No biological rule was added.

After these changes, `scripts/check.sh` passed all 10 tests, formatting,
native/WASM Clippy with warnings denied, and JavaScript syntax validation.
`scripts/with-toolchain.sh trunk build --locked --release` passed; the rebuilt
WASM is 34,684,303 bytes before compression.

The rebuilt static release was reloaded in the in-app Chromium browser. Enter
selected Fern while keeping focus on her button, then Tab reached Flint. Flint
displayed the hunter role and 60 / 100 energy after three Step clicks; Meadow
displayed its name, cell (16, 13), and 80 biomass. Reset returned to tick zero in
a new run, and Fern remained selectable. No browser console errors were reported.
The broader resize, camera, and real Chrome hidden-tab checks above were not
repeated for this metadata/focus refinement.

The `Has<FoodPatch>` query used for appearance selection was checked against the
official `bevy_ecs` 0.18.1 crate's bundled `query/fetch.rs` documentation. It tests
component presence without reading or mutating the component.

## Species, geometry, and teaching scaffold — September 20, 2026

Separated `Species` and `EcologicalRole` from individual nicknames. The fixture
is now documented as Hare #1 (Fern), Fox #2 (Flint), and Grass patch #3 (Meadow).
Extracted its authored data to `fixture.rs`; added documented unscheduled
maintenance/food-choice/movement/eating stubs in `lessons.rs`. Their explicit
unimplemented bodies are not registered in `SimTick`. No biology was implemented.

All markers now use one 0.8-cell schematic square size shared by drawing, square
picking, and the selection outline. This replaces the old arbitrary unequal
rectangles. The inspector reports physical size as not modeled.

Actual checks after the code changes:

- `scripts/check.sh`: all 10 native tests, formatting, native/WASM Clippy with
  warnings denied, and JS syntax passed. The existing fixture regression also
  checks the species/role data.
- `scripts/with-toolchain.sh cargo doc -p moss-sim --no-deps --locked`: passed
  without warnings; component and lesson documentation is navigable.
- `scripts/with-toolchain.sh trunk build --locked --release`: passed. Current
  WASM size is 34,695,168 bytes before compression.
- In-app Chromium reloaded the static release. All three species/role/name
  combinations displayed correctly; keyboard selection retained focus. The
  equal-sized squares and matching selection outline rendered, and clicking
  a visible corner of the grass marker selected the patch. Three Steps reached
  tick 3 with the fox still at 60 / 100 energy. Reset returned to tick zero in
  run 2 with Hare #1 selectable. No console errors were reported.

[Updated inspector screenshot](screenshots/taxonomy.png). The earlier Chrome
hidden-tab and resize tests were not repeated for this refinement. Population
scenarios/aggregates, all lesson behavior, plant growth/spread, physical bodies,
daylight, and weather remain unimplemented; the ecology plan describes future
acceptance checks, not passing tests.

## First biological rule — September 21, 2026

Nick implemented the mutable energy loop in `lessons.rs` and corrected the
`creatues` spelling error caught by `cargo check`. The agent preserved the loop,
formatted it, and registered `(lessons::spend_energy, complete_tick).chain()`.
The official Bevy ECS 0.18.1 source documentation for `IntoScheduleConfigs::chain`
confirms ordering constraints between successive systems. The other three
biological stubs remain unscheduled. Updated the browser's obsolete fixed-energy
note and current documentation.

`scripts/check.sh` passed formatting, all 10 existing native tests, native/WASM
Clippy with warnings denied, and JS syntax. The running Trunk release preview
rebuilt successfully. A bounded read-only code review found no maintenance or
scheduling issues. The focused biological regression is still the next paired
edit; passing infrastructure tests do not provide that missing coverage.

In-app Chromium on localhost:8080 showed both creatures at 57 / 100 after three
Steps. Later playback and a further Step reached tick 244; both creatures still
existed at 0 / 100, positions were unchanged, and the grass patch retained 80
biomass. Reset returned to tick zero and 60 / 100. Three more Steps returned to
57 / 100 while paused. No console errors were reported. A selector wait timed
out before depletion; subsequent visible observations established the zero
boundary. These are browser observations, not a new native regression.

Camera/resize/hidden-tab behavior was not rechecked for this rule. No death,
movement, feeding, or other biological rule was added.

## First biological regression — September 21, 2026

Nick completed `maintenance_spends_energy_and_stops_at_zero`. It runs the real
installed schedule, checks both creatures at reserve 60/capacity 100 initially,
57 after three ticks, then zero after 63 total ticks. Matching creature counts
remain two throughout, guarding against an empty assertion loop or adding death.
A bounded read-only review found no blocking issues. Only import formatting
changed during the review; the user's test implementation was preserved.

Actual checks:

- `scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap maintenance_spends_energy_and_stops_at_zero`: passed.
- `scripts/check.sh`: initially stopped at import formatting; after `cargo fmt
  --all`, all 11 native tests, formatting, native/WASM Clippy with warnings denied,
  and JS syntax passed.

The browser smoke test and release build were not repeated for this test-only
change. The preceding browser observations cover the unchanged maintenance rule.
The planned population scenario and later biological rules remain unimplemented.

## Remaining limits

Desktop mouse/trackpad is the initial input target. Mobile touch/pinch,
Safari/Firefox, WebGPU, public deployment, subpath asset hosting, long-run
performance, save/load, and later biological rules have not been validated.
History is memory-only, bounded to 32 events and replaced on Reset.

Playback requests at most four complete ticks per rendered frame. Under load,
excess wall time is discarded and the UI reports slower simulation; no rules
are skipped inside an executed tick. Hidden pages remain paused on return.

## References checked before implementation

- [Bevy v0.18.1 manifest](https://github.com/bevyengine/bevy/blob/v0.18.1/Cargo.toml)
  and [feature closure](https://github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_internal/Cargo.toml).
- [ECS World 0.18.1](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/world/struct.World.html)
  and [single-thread executor](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/schedule/enum.ExecutorKind.html).
- [Window 0.18.1](https://docs.rs/bevy/0.18.1/bevy/window/struct.Window.html),
  [Sprite](https://docs.rs/bevy/0.18.1/bevy/sprite/struct.Sprite.html),
  [orthographic projection](https://docs.rs/bevy/0.18.1/bevy/camera/struct.OrthographicProjection.html).
- [Trunk configuration](https://trunk-rs.github.io/trunk/guide/configuration/index.html)
  and [assets](https://trunk-rs.github.io/trunk/guide/assets/index.html).
- [wasm-bindgen JavaScript namespace imports](https://rustwasm.github.io/docs/wasm-bindgen/reference/attributes/on-js-imports/js_namespace.html).
- [Page Visibility API](https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API)
  and [pointer capture](https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture).

Source compatibility was a hypothesis until the builds above succeeded. Bevy
0.19.1 was available during resolution but requires Rust 1.95; 0.18.1 kept the
already-installed stable toolchain and aligned ECS without an unrelated upgrade.
