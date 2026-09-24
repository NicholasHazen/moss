# Run and develop Moss

Run commands from the workspace root (`/Users/nick/Code/moss` on this Mac).
See [verified coverage](verification.md) for the tested stack and limitations,
and [RustRover](rustrover.md) for shared IDE actions.

## Run the browser app

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

Open <http://127.0.0.1:8080>. The server binds to loopback; stop it with Ctrl-C.
Only one terminal or IDE preview can own port 8080. Omit `--release` for debug
iteration after its first compile. `cargo run -p moss-web` prints a reminder;
the native binary does not open the browser app.

## Check and build

**Current teaching checkpoint:** full tests intentionally fail at the unfinished
`move_one_cell` helper. Its adapter is unscheduled, so the browser can still build
and run maintenance. Two movement integration tests are explicitly ignored until
reviewed activation; [the movement chapter](../../learning/content/14-movement.html) explains the red-to-green edit.

```sh
scripts/check.sh
scripts/with-toolchain.sh trunk build --locked --release
```

The check script runs rustfmt, native workspace tests, native/WASM Clippy with
warnings denied, and JavaScript syntax validation. Trunk produces `dist/`.
Neither command establishes browser runtime behavior. For a static bundle check:

```sh
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

Open <http://127.0.0.1:8081>; it serves the latest built bundle without live
reload. Stop it with Ctrl-C. Public deployment requires a separate request.

## Focused commands

```sh
scripts/with-toolchain.sh cargo fmt --all
scripts/with-toolchain.sh cargo test -p moss-sim --locked
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test movement movement_charges_only_an_affordable_actual_step -- --exact
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test maintenance maintenance_spends_energy_and_stops_at_zero -- --exact
scripts/with-toolchain.sh cargo doc -p moss-sim --no-deps --locked
```

The focused test must report one matching test, not zero. Generated API docs
start at `target/doc/moss_sim/index.html`. Tutorial experiments use a separate
workspace and `CARGO_TARGET_DIR` so they cannot replace live test artifacts.

## Set up another machine

`rust-toolchain.toml` selects Rust 1.93.1 and the WebAssembly target. The tested
Trunk and wasm-bindgen CLI versions are installed separately:

```sh
rustup toolchain install 1.93.1 --component rustfmt --component clippy --target wasm32-unknown-unknown
cargo +1.93.1 install trunk --version 0.21.14 --locked
cargo +1.93.1 install wasm-bindgen-cli --version 0.2.123 --locked
```

These are setup instructions, not commands executed during the documentation
cleanup. Use trusted official sources and the environment's installation
permissions. Node is used only by `node --check` in the check script; it is not
an application dependency. macOS builds also need a working Apple compiler and
SDK, supplied by Xcode or the standalone Command Line Tools.

The wrapper selects standalone Command Line Tools only when that directory exists
and no explicit `DEVELOPER_DIR` is set. On this Mac the directory is now absent,
so builds use the selected Xcode. It normalizes `NO_COLOR=1` for Trunk and changes
neither the system Xcode selection nor its license state.
See the [build history](../history/README.md#build-records) for original diagnostics.

## Browser smoke checks

After rebuilding, reload the served bundle. Check startup and the grid,
selection/inspection, three Steps, Play/Pause and Reset. Default rates should
leave both animals at 57 / 100 after three Steps; zero energy does not cause
death. Camera actions while paused must preserve biological state.

That is the current maintenance-only baseline. After movement is reviewed and
activated, use [the movement acceptance table](../../learning/content/14-movement.html#account-for-the-journey)
instead: Fern pays maintenance plus travel while Flint remains still.

When input or lifecycle behavior changes, also check drag versus click,
pointer-anchored zoom, resize, panel input isolation and real hidden-tab return.
Record the browser/version and exactly which checks ran. Keep unavailable
checks explicit in the verification summary.
