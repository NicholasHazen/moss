# Moss in RustRover

Open `/Users/nick/Code/moss` as the project. The configured application is
RustRover **2026.3 EAP, build 263.5153.48**. Rust remains pinned to **1.93.1**
by `rust-toolchain.toml`; the IDE toolchain points to `~/.cargo/bin`.

The organization pass moved the focused maintenance regression into
`crates/moss-sim/tests/maintenance.rs`. The shared **Moss - Maintenance test**
action now uses `--test maintenance`; its exact Cargo command was verified from
the terminal on September 22. The test's name is unchanged.

## Read Fieldnotes alongside the editor

Open the [Fieldnotes movement chapter](https://nicholashazen.github.io/moss/14-movement.html)
in a browser and `crates/moss-sim/src/lessons.rs` in RustRover. The chapter contains
the full assignment, answer and review stop. Keep the browser alongside the
editor when comparing prose with the function; the Run menu below checks your
saved source. The [local course guide](../../learning/README.md#run) explains how
to serve Fieldnotes from this checkout for offline reading.

## The run menu

Shared configurations are stored in `.run/` and versioned with the code. Choose
one from the toolbar's run-configuration menu, then press the green Run button.
If the menu shows only recent entries, expand **All Configurations**.

| Configuration | Purpose |
| --- | --- |
| Moss - Maintenance test | Run Nick's first rule regression. Native Debug completed this test; see verification below. |
| Moss - Movement test | Run the single prepared helper test. It intentionally fails until today's rule is implemented. |
| Moss - Simulation tests | Run `moss-sim` tests without the browser or renderer. |
| Moss - All tests | Run all workspace tests with the IDE's test-results tree. |
| Moss - Full checks | Run formatting, tests, native/WASM Clippy, and JS syntax checks. |
| Moss - Browser build | Produce the release WebAssembly bundle in `dist/`. |
| Moss - Browser preview | Build and serve the live-reloading browser app at http://127.0.0.1:8080. Stop with the IDE's red Stop button. |

Cargo test configurations honor the pinned toolchain, use `--locked`, and
retain Cargo's build-before-run step. Shell configurations invoke the existing
scripts from the project root and show output in the Run window. They do not
require a global WASM build target: native tests stay native, and Trunk selects
WebAssembly for the browser build.

The movement configuration uses the exact focused command from the guide. On
September 22, its normal Run action reached the one named test and failed at the
unfinished helper's `todo!()`, as intended. The [IDE execution record](../history/builds/2026-09-22.md#movement-action-in-rustrover--september-22-2026)
keeps the result separate from a completed solution. Normal Run is enough for
today's checkpoint; movement Debug has not been checked.

The older automatically created `Run moss-web` configuration is a native
`cargo run`, which does not launch the browser application. Use **Moss - Browser
preview** for the application. Only one preview should own port 8080; stop the
existing terminal or IDE preview before starting another. A successful test
run is separate from a browser runtime check.

## This Mac's compiler setting

**Settings → Rust → Environment variables** now has no `DEVELOPER_DIR` override.
This Mac uses its selected Xcode at `/Applications/Xcode.app/Contents/Developer`.
The earlier project-local override pointed at `/Library/Developer/CommandLineTools`,
which no longer exists; a fresh native link failed before the debugger could start.
Removing that stale entry repaired the build without changing macOS's Xcode
selection. The change lives in ignored `.idea/workspace.xml`; do not recreate the
old override in a fresh checkout. Other machines should use their own working
compiler environment. Shared `.run/` files contain no Mac-specific path.

The shell configurations use `scripts/with-toolchain.sh`. It selects standalone
Command Line Tools only when that directory exists and no explicit override is
set; otherwise it leaves compiler selection alone. It also normalizes Trunk's
`NO_COLOR` value. No wrapper change was needed for this repair.

After the repair, **Moss - Maintenance test** freshly compiled, launched the
bundled LLDB and completed **one test passed, exit code 0**. This verifies native
Debug execution of that test, not breakpoints, stepping, locals or browser/WASM
debugging. The earlier “Instantiating tests…” stall did not recur; its original
cause remains unproven. Temporary trace logging was cleared after the retry.
The [dated debugger record](../history/builds/2026-09-22.md#native-debugger-retry--september-22-2026)
contains the failure and successful execution evidence. No restart, installation,
system permission or security change was needed. To inspect browser-only code,
the IDE's target selector can switch analysis to `wasm32-unknown-unknown`; return
to the native target for the simulation test workflow.

## False errors and stalled analysis — September 21, 2026

This EAP's JetBrains Academy plugin caused false analysis errors. Disabling
Academy and restarting restored healthy analysis; normal Run mode passed the
maintenance test. The [incident record](../history/rustrover-2026-09-21.md)
contains the exact plugin version, log evidence and verification limits.

If false errors return, compare Cargo's result and the current IDE log before
changing correct source or disabling inspections. Recheck the same failure
before applying an old workaround. Native Debug's breakpoint, stepping and locals
coverage remains untested; custom-rate browser acceptance is recorded separately
in verification.

## Configuration references

JetBrains documents [Cargo run/test settings](https://www.jetbrains.com/help/rust/cargo-run-debug-configuration.html)
and [Shell Script configurations](https://www.jetbrains.com/help/clion/run-debug-configuration-shell-script.html).
The exact XML fields and project environment inheritance were also checked
against the installed EAP plugin classes. Actual verification and its limits are
recorded in [verification](verification.md).
