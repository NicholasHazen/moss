# Moss in RustRover

Open `/Users/nick/Code/moss` as the project. The configured application is
RustRover **2026.3 EAP, build 263.5153.48**. Rust remains pinned to **1.93.1**
by `rust-toolchain.toml`; the IDE toolchain points to `~/.cargo/bin`.

## The run menu

Shared configurations are stored in `.run/` and versioned with the code. Choose
one from the toolbar's run-configuration menu, then press the green Run button.
If the menu shows only recent entries, expand **All Configurations**.

| Configuration | Purpose |
| --- | --- |
| Moss - Maintenance test | Run Nick's first rule regression. Native Debug currently stalls; see below. |
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

The older automatically created `Run moss-web` configuration is a native
`cargo run`, which does not launch the browser application. Use **Moss - Browser
preview** for the application. Only one preview should own port 8080; stop the
existing terminal or IDE preview before starting another. A successful test
run is separate from a browser runtime check.

## This Mac's compiler setting

**Settings → Rust → Environment variables** contains the project-local setting:

```text
DEVELOPER_DIR=/Library/Developer/CommandLineTools
```

This uses the working Command Line Tools without changing macOS's system-wide
Xcode selection. It applies to RustRover's Cargo builds, tests (including gutter
runs), and analysis. It lives in ignored `.idea/workspace.xml`; a fresh checkout
on this Mac needs this setting once. Other machines should use their own working
compiler environment. The shared `.run/` files contain no Mac-specific path.
The shell configurations use `scripts/with-toolchain.sh`, which makes the same
conditional choice and normalizes Trunk's `NO_COLOR` value.

Native Debug was attempted with **Moss - Maintenance test** but remained at
“Instantiating tests…” after LLDB launched. Run mode works; Debug is not verified.
The IDE log shows backend startup and the correct compiler environment, without
a failure reason. The next diagnostic is a single retry with JetBrains' debugger
logging enabled (`com.jetbrains.cidr.execution.debugger` under Help → Diagnostic
Tools → Debug Log Settings); remove the category afterward. No system permission
or security settings were changed. Browser/WASM debugging is a separate workflow.
To inspect browser-only code,
the IDE's target selector can switch analysis to `wasm32-unknown-unknown`; return
to the native target for the simulation test workflow.

## Configuration references

JetBrains documents [Cargo run/test settings](https://www.jetbrains.com/help/rust/cargo-run-debug-configuration.html)
and [Shell Script configurations](https://www.jetbrains.com/help/clion/run-debug-configuration-shell-script.html).
The exact XML fields and project environment inheritance were also checked
against the installed EAP plugin classes. Actual verification and its limits are
recorded in [BUILD_NOTES.md](BUILD_NOTES.md).
