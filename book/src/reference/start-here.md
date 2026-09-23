# Put the book beside a running world

The first useful sight is modest: Fern stays in place while three presses of
**Step** lower her reserve from 60 to 57. That tells you the browser can ask the
simulation to execute ticks and inspect their result. Giving those ticks a
journey is the first coding task.

If you already have the Moss project used by this edition, keep your checkout
and begin with [one affordable step](../chapters/01-one-affordable-step.md#your-first-edit).
For a fresh start, use the edition's small workspace:

**[Download the September 23 starter ZIP](../downloads/moss-starter-2026-09-23.zip)**
· [SHA-256 checksum](../downloads/moss-starter-2026-09-23.zip.sha256)

Extract it into a new directory and open its `Cargo.toml` in RustRover or your
preferred editor. Run the commands below from the extracted folder. In
RustRover, its terminal is enough; the archive does not include personal IDE
settings or preconfigured Run actions.

## What you have opened

The workspace contains two crates. `moss-sim` owns the ECS world and simulation
rules. `moss-web` turns that state into a browser view and submits control
requests. A native simulation test can exercise an entire tick without opening
a window, while Trunk builds the browser's Rust into WebAssembly.

```text
Cargo.toml                  workspace and shared dependencies
crates/moss-sim/src/         data, tick order and your first rule
crates/moss-sim/tests/       observable simulation checkpoints
crates/moss-web/             browser shell and presentation
scripts/                    build-environment wrapper and checks
docs/tutorial/              extracted Rust references and their runner
```

This download captures the edition's actual working-tree source, including local
changes based on commit `83a72a5abd46339e5448076fc183bfe33cd54029`. It is not an
archive of that clean Git commit. `STARTER-MANIFEST.json` records every other
included file, its origin and its SHA-256 hash. The
[Moss repository](https://github.com/NicholasHazen/moss) may continue past this
checkpoint; use the download when you want the exact starting exercise described
here.

## Bring up the tools once

The commands use a Unix shell on macOS or Linux. Install Rust through the
[official rustup instructions](https://rust-lang.github.io/rustup/installation/index.html)
if `rustup` is not available. A native compiler/linker is also needed; on macOS,
that comes with Apple's Command Line Tools or Xcode. Native Windows setup has
not been verified for this edition, and the supplied shell wrappers need a
Unix-like shell.

The project selects Rust 1.93.1 in `rust-toolchain.toml`. Install its browser
target and the separately versioned build tools:

```sh
rustup toolchain install 1.93.1 --component rustfmt --component clippy --target wasm32-unknown-unknown
cargo +1.93.1 install trunk --version 0.21.14 --locked
cargo +1.93.1 install wasm-bindgen-cli --version 0.2.123 --locked
scripts/with-toolchain.sh cargo fetch --locked
```

Trunk packages the HTML, styles, JavaScript and compiled Rust. Its
[official setup guide](https://trunk-rs.github.io/trunk/guide/getting-started/index.html)
explains that build path; the
[wasm-bindgen CLI reference](https://rustwasm.github.io/docs/wasm-bindgen/reference/cli.html)
describes the WebAssembly bindings tool. These are the edition's pinned versions,
so avoid substituting whichever versions happen to be newest while establishing
the baseline.

Fetching dependencies once matters because the worked-reference runner later
uses Cargo offline. `--locked` keeps Cargo aligned with the supplied dependency
lockfile; it does not itself mean “no network.” See
[Cargo's fetch reference](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)
for that distinction. The initial installs and compilation can take considerably
longer than a subsequent edit.

Python 3 is needed for the optional reference runner. Node is used by the full
check script's JavaScript syntax check; the application itself has no Node
runtime dependency.

## Establish one green result

Start with the implemented simulation rules:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap --test maintenance
```

Expect **five passing tests**. They check the authored fixture, stable identities,
tick/reset behavior and upkeep. A passing result establishes that baseline
before your movement edit adds another behavior.

Now start the browser build and local server:

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

When the server reports readiness, open <http://127.0.0.1:8080/>. Select Fern
and press **Step** three times. Her reserve should read **57 / 100**, and her
position should be unchanged. **Reset** restores the starting fixture. These are
the expected observations to make on your machine; a successful compilation
alone does not prove that the browser loaded it. Stop the server with Ctrl-C.

If port 8080 is already occupied, stop the other preview terminal before
starting another. If the page still says Loading after a successful build,
inspect the browser console before changing simulation code: the helper you
are about to write is not scheduled yet.

## The intentional red result is your doorway

Open `crates/moss-sim/src/lessons.rs` and find `move_one_cell`. Its body contains
`todo!()`. Run the one prepared test that calls it:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test movement movement_charges_only_an_affordable_actual_step -- --exact
```

One selected test should fail at the unfinished helper. That failure is useful:
the code compiled, the test found the intended function, and it reached the
place you will change. Zero matching tests or a compiler error is a different
result. Full workspace tests also reach this deliberate failure, so the green
baseline command above selects only the completed rules.

The exercise has a short handoff:

```text
One helper body → one named test passes → review → activate the prepared system
```

Chapter 1 explains the rule and gives a complete worked answer. Its first
stopping point is **the named movement test passing**. The browser remains
maintenance-only until the adapter is deliberately scheduled and its two
integration tests are enabled. Animals remain alive at zero energy; meals,
autonomous choice and death are later chapters.

## Run an answer without replacing your exercise

When a later chapter gives an isolated reference, these commands work from the
same extracted folder after dependencies are fetched:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 01
python3 docs/tutorial/authoring/check_path_examples.py
python3 docs/tutorial/authoring/check_path_examples.py --movement
```

The first checks one meal example. The second checks all sixteen future
reference modules. The third substitutes and activates the movement answer
**inside a temporary copy**, leaving your real helper and schedule alone.
The runner creates a separate build directory and removes the temporary copy
afterward.

The Markdown files in this download are clearly labeled **extracted worked
references**. They retain the exact Rust fences needed by the runner, with their
source hashes, while the explanations remain in this book. The download omits
Git history, collaboration notes, editorial records, dependency caches and
compiled output. Original Rust comments may still name fuller repository docs;
use the [code map](code-map.md) to navigate the edition.

You now have a baseline you can distinguish from your next change. Open
[one step that Fern can afford](../chapters/01-one-affordable-step.md#your-first-edit)
and make the copied proposal become a paid, accepted step.
