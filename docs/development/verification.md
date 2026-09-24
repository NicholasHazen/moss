# Verification and tested stack

**Runtime baseline: September 22, 2026, movement preparation and IDE checks.**
The latest focused test check is September 23, described below.
The browser remains maintenance-only. One intentionally red helper test marks
Nick's next edit; two integration tests wait for reviewed activation.
[Development commands](README.md) explain the runnable preview and focused test.

## Current coverage

| Check | Observed result |
| --- | --- |
| Existing live native tests | Seven simulation and five presentation tests passed. |
| New movement exercise | One expected failure at `move_one_cell`'s `todo!()`; two explicit activation tests ignored. Both the terminal command and the native RustRover Run action reached the one named helper test. |
| Native maintenance Debug | After removing a stale project compiler override, RustRover freshly built and ran one passing maintenance test through bundled LLDB, exit 0. Breakpoints, stepping and locals were not exercised. |
| Formatting and lint | rustfmt, native/WASM Clippy with warnings denied, and JS syntax passed. |
| Browser bundle | Release Trunk build passed with the helper unfinished and unscheduled. |
| Live browser baseline | Startup, both animals at 57 after three Steps, unchanged positions, Fern's maintenance rate 1 and authored destination #3, and Reset to reserve 60/target #3 observed in in-app Chromium. |
| Isolated completed example | The exact worked movement helper plus activated schedule passed all 15 native workspace tests, zero ignored, in its own temporary workspace/target directory. No solution installed live. |
| Diagrams | Both tutorial SVGs rendered and inspected in Chromium, then in v2's native RustRover preview. Background scrolling exposed a repaint limitation recorded below. |

[The dated build record](../history/builds/2026-09-22.md#movement-session-preparation--september-22-2026)
contains commands and limitations. [Dated tutorial records](../history/README.md#tutorial-checks) distinguish the
helper, installed-world and future-reference evidence recorded at that time.
[Fieldnotes' reading and model guide](../../learning/content/about.html) explains
the current book's tools and their scope; it is separate from this live-runtime record.

The later [native Run check](../history/builds/2026-09-22.md#movement-action-in-rustrover--september-22-2026)
ended with the expected unfinished-helper panic and exit 101. The
[native reading check](../history/tutorial/2026-09-22.md#native-reading-checkpoint-september-22-2026)
covers v2's diagrams and navigation, including the background repaint limitation.
The later [debugger retry](../history/builds/2026-09-22.md#native-debugger-retry--september-22-2026)
records the obsolete compiler path, its project-local repair and the passing
maintenance Debug execution. Other suite, WASM and browser checks were not rerun
for that repair.

The [September 23 rate check](../history/tutorial/2026-09-23.md#the-supplied-movement-rate-matters)
added two cases to the movement helper test. Its live command compiled and reached
the same expected unfinished-helper failure. In a separate copy, the written
answer passed that stronger test, all five movement/maintenance checks and Clippy.
The earlier whole-workspace, browser, WASM and IDE results above were not rerun.

Live movement/meal browser acceptance remains pending Nick's helper review and
activation. Camera, Play/Pause, resize and hidden-tab checks were not repeated
for this preparation; their earlier evidence remains in dated build records.
Custom-rate browser/reset acceptance also remains pending. No dependency changed.

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

The recorded bootstrap `cargo tree` check found no audio, PBR, UI, or glTF
crates in the active browser dependency graph. Cargo's lockfile
also contains platform/optional resolution entries; those are not a list of
compiled browser features. `Cargo.lock` is tracked; no dependency versions were
changed during either organization pass.

## Remaining limits

Desktop mouse/trackpad is the initial input target. Mobile touch/pinch,
Safari/Firefox, WebGPU, public deployment, subpath asset hosting, long-run
performance, save/load, and later biological rules have not been validated.
History is memory-only, bounded to 32 events and replaced on Reset.

Playback requests at most four complete ticks per rendered frame. Under load,
excess wall time is discarded and the UI reports slower simulation; no rules
are skipped inside an executed tick. Hidden pages remain paused on return.
