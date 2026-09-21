# Moss

**A little life. A world to grow.**

A browser-first Rust/ECS ecosystem workbench. The foundation has a finite 32 × 20 chamber, Fern and Flint, a food patch, inspection, camera controls, and explicit simulation ticks. Creatures now spend one energy unit per tick, bounded at zero, starting from 60 / 100. Movement, feeding, and death are not implemented yet.

Start with [NOW.md](NOW.md) when returning. The vision remains in [PROJECT_BRIEF.md](PROJECT_BRIEF.md); exact evidence and limitations are in [docs/BUILD_NOTES.md](docs/BUILD_NOTES.md).

For the configured RustRover EAP build/test actions, see [the IDE guide](docs/RUSTROVER.md).
For a guided coding session, start with [the tutorial's first small checkpoint](docs/tutorial/01-species-energy.md#checkpoint-a--describe-the-settings).
The [TutorBro guide](docs/tutorial/README.md) includes five coding chapters,
worked examples, test checkpoints and review stops, plus optional Rust/ECS and
project context. Its linked Markdown pages are designed for RustRover; the
authoring notes and chapter template keep it expandable as Moss changes.

## Run locally

From `/Users/nick/Code/moss`:

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

Open <http://127.0.0.1:8080>. Stop the server with Ctrl-C. It binds only to loopback. For faster iteration after the first debug compile, omit `--release`. No backend service or external assets are needed.

The checked-in `rust-toolchain.toml` selects Rust 1.93.1 and the WebAssembly target. This machine already has Trunk 0.21.14 and wasm-bindgen-cli 0.2.123. For a new machine, install those versions from their official sources:

```sh
rustup toolchain install 1.93.1 --component rustfmt --component clippy --target wasm32-unknown-unknown
cargo +1.93.1 install trunk --version 0.21.14 --locked
cargo +1.93.1 install wasm-bindgen-cli --version 0.2.123 --locked
```

These installation commands are setup guidance; no tool installation was needed or executed during this bootstrap. On macOS, working Apple Command Line Tools are also required. The small `with-toolchain.sh` wrapper selects the existing Command Line Tools for that process, respecting an explicit `DEVELOPER_DIR`. It does not change the system-wide Xcode selection.

## Use the workbench

- **Play / Pause** advances or freezes the clock at four ticks per second.
- **Step** pauses playback and completes exactly one tick.
- **Reset** starts a fresh run at tick zero, restores the fixture, clears selection, and fits the world. The previous run's in-memory journal is discarded.
- Drag the map to pan; wheel to zoom toward the pointer. **Fit world** restores the chamber. A click selects; a drag preserves the existing selection.
- Select from the text list as an alternative to clicking a shape. The inspector reads current ECS components; it does not maintain another simulation.
- Hiding the page suspends playback. Returning leaves it paused, without offline catch-up. Resume explicitly with Play.

The journal retains at most 32 actual events for the current run and shows its coverage. Bootstrap records initialization and authored placement, not births or invented decisions. Refreshing the page starts over; there is no saved history.

## Check and build

All commands below run from the workspace root:

```sh
scripts/check.sh
scripts/with-toolchain.sh trunk build --locked --release
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

The check script runs formatting, 11 native tests, native and WebAssembly Clippy with warnings denied, and JavaScript syntax validation (`node --check`). Node is needed only for the syntax check, not to run Moss. Visit <http://127.0.0.1:8081> for the separately served static release bundle. Public deployment and subpath hosting are outside this bootstrap.

To run only the renderer-free simulation tests:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked
```

## Five places to understand

1. [`crates/moss-sim/src/lessons.rs`](crates/moss-sim/src/lessons.rs): start here. Nick's maintenance loop is implemented; the other three rule stubs remain unimplemented and unscheduled. Later function parameters are deliberately provisional.
2. [`crates/moss-sim/src/lib.rs`](crates/moss-sim/src/lib.rs): components, validated dimensions, journal, and `SimTick`. `install()` explicitly chains maintenance before `complete_tick` on a single-threaded executor.
3. [`crates/moss-sim/src/fixture.rs`](crates/moss-sim/src/fixture.rs): who exists initially, their species/roles, positions, reserves, and IDs.
4. [`crates/moss-sim/tests/bootstrap.rs`](crates/moss-sim/tests/bootstrap.rs): the same simulation runs in a plain ECS world without a renderer or browser. Nick's maintenance regression checks both creatures at 60 → 57 → 0 energy and confirms they still exist at zero.
5. [`crates/moss-web/src/browser.rs`](crates/moss-web/src/browser.rs): `frame()` requests ticks, `snapshot()` reads inspection data, and `sync_view()` draws it. Adjacent `playback.rs`/`view.rs` handle time and camera math; `shell.js` handles the DOM.

Generate navigable API documentation with `scripts/with-toolchain.sh cargo doc -p moss-sim --no-deps --locked`; open `target/doc/moss_sim/index.html`. This command has been verified.

## Meet the fixture

Species and stable ID lead the labels; nicknames remain secondary. The provisional game species are Hare, Fox, and Grass. Species groups individuals; ecological role describes their place in the food web. “Archetype” retains Bevy's technical meaning of a shared set of component types.

| Entity | Data today | Ecological role |
| --- | --- | --- |
| Hare #1, nicknamed Fern | Position (10, 10), energy 60 / 100 | Grazer |
| Fox #2, nicknamed Flint | Position (21, 6), energy 60 / 100 | Hunter |
| Grass patch #3, labeled Meadow | Position (16, 13), biomass 80 | Producer |

Step now advances the clock and spends one energy unit on each creature;
positions and biomass remain unchanged. All names, roles, positions, and reserves originate
in `moss-sim`; the browser chooses shapes/colors and displays that data.

One entity can carry several components: Fern has `SimId`, `Position`, `Creature`,
`Energy`, `Species`, and `EcologicalRole`. A system operates on entities with the data it needs. Fern and
Flint can therefore share maintenance while differing in later food-seeking
rules. Neither needs its own class or a name-specific policy.

The first paired edit makes time cost energy; its focused regression passes.
The next paired refinement makes passive burn configurable by species. Then the
next mechanical slice is a separate
population scenario with 6 hares, 2 foxes, 4 patches, and species totals. It does
not require reproduction. [The ecology plan](docs/ECOLOGY_PLAN.md) defines that
scenario, local plant growth before any CA spread, physical extent versus the
equal-sized schematic markers, tick-driven sunlight, and later weather.

The current three-entity chamber remains the diagnostic scene. No population
scenario, plant growth, day/night, weather, or physical body model is implemented.
Balance becomes a question after individual interactions can be explained.

Next: pair on configurable passive burn, then scaffold the authored population
scenario described in [NOW.md](NOW.md).
The maintenance loop is written, scheduled, and tested; food choice, movement,
eating, and starvation remain separate paired edits.
