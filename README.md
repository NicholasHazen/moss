# Moss

**A little life. A world to grow.**

A browser-first Rust/ECS ecosystem workbench. The current 32 × 20 chamber has
Hare #1 (Fern), Fox #2 (Flint), a grass patch, inspection, camera controls and
explicit simulation ticks. Species-configured maintenance spends energy,
bounded at zero. Movement, feeding and death are future work.

Start with [NOW.md](NOW.md) when returning, the
[documentation index](docs/README.md) to browse, or the
[today's v2 tutorial](docs/tutorial/today-v2.md) for the prepared movement session.
[PROJECT_BRIEF.md](PROJECT_BRIEF.md) records the vision.

## Learn with Fieldnotes

[Read Moss Fieldnotes](https://nicholashazen.github.io/moss/fieldnotes/) for the
new Rust and ECS course, or use [the course source and local setup](learning/README.md).
The course develops a separate continuing terrarium through food, population,
movement, rest, hunting and refuge. It does not change the live movement assignment.
The [original Moss book](https://nicholashazen.github.io/moss/) remains available.

## Run locally

From the workspace root (`/Users/nick/Code/moss` on this Mac):

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

Open <http://127.0.0.1:8080>. Stop with Ctrl-C. See the
[development guide](docs/development/README.md) for installation, debug iteration,
static builds and environment notes, or the
[RustRover guide](docs/development/rustrover.md) for IDE actions.

## Use the workbench

- **Play / Pause** advances or freezes the clock at four ticks per second.
- **Step** pauses and completes one tick.
- **Reset** restores the authored fixture in a new run, clears selection and
  the previous journal, and fits the world.
- Drag to pan, wheel to zoom toward the pointer, and click a marker or text-list
  entry to inspect it. A drag preserves selection.
- Hiding the page suspends playback. Returning leaves it paused without catch-up.

The inspector reads ECS state. The journal retains at most 32 actual events
for the current run and shows its coverage. Refreshing starts over; history is
not saved. [Browser design](docs/design/browser-experience.md) describes the
implemented controls and future interaction scope.

## Check and build

The current movement exercise has one intentional failing test until
`move_one_cell` is implemented. Two integration checks wait for reviewed
activation. The preview remains runnable because movement is unscheduled.

```sh
scripts/check.sh
scripts/with-toolchain.sh trunk build --locked --release
```

The [development guide](docs/development/README.md#check-and-build) explains
what these check and how to serve the result. [Verification](docs/development/verification.md)
records tested versions, actual runtime coverage and remaining limitations.

## Code map

`moss-sim` owns biological data and rules; `moss-web` hosts the one ECS world
and its browser presentation. Private simulation modules expose a small root
API through [`lib.rs`](crates/moss-sim/src/lib.rs).

Today's edit is `move_one_cell` in [`lessons.rs`](crates/moss-sim/src/lessons.rs),
beside the implemented maintenance system. Its prepared
[movement test](crates/moss-sim/tests/movement.rs) calls ordinary Rust values.
[`energy.rs`](crates/moss-sim/src/energy.rs) holds reserves and rates; the
[maintenance regressions](crates/moss-sim/tests/maintenance.rs) remain useful.
[`simulation.rs`](crates/moss-sim/src/simulation.rs) wires ticks and reset;
[`browser.rs`](crates/moss-web/src/browser.rs) builds the app.
The [architecture map](docs/design/architecture.md#modules-grow-with-actual-behavior)
lists the focused modules. Agents follow [the coding conventions](docs/agents/coding-style.md).

## Meet the fixture

| Entity | Starting state |
| --- | --- |
| Hare #1, Fern | Cell (10, 10), 60 / 100 energy; grazer. |
| Fox #2, Flint | Cell (21, 6), 60 / 100 energy; hunter. |
| Grass patch #3, Meadow | Cell (16, 13), 80 biomass; producer. |

With default rates, three Steps leave both animals at 57 energy. Positions and
biomass stay unchanged; animals still exist at zero. The diagnostic fixture
keeps one rule easy to inspect. The [ecology plan](docs/design/ecology.md) covers
future populations and interactions; [NOW.md](NOW.md) selects the next edit.
