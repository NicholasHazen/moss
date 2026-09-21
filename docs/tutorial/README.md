# Returning to Moss: a few small, testable changes

**Start with [Chapter 1, checkpoint A](01-species-energy.md#checkpoint-a--describe-the-settings). You can stop there today.**
That edit introduces the shared species settings without changing any behavior.
You do not need to read the other chapters first.

You already wrote Moss's first biological rule and its regression. Each animal
starts at 60 energy, loses one per tick, and stays present at zero. This guide
builds from that working code. It is a companion while editing, not a Rust course
you need to finish before contributing.

## The route

| Chapter | Visible result when complete | Your small rule | Agent preparation |
| --- | --- | --- | --- |
| [1. Species energy](01-species-energy.md) | The inspector explains each animal's passive rate. | Replace the shared literal with a species setting. | Install configuration and expose it in the inspector. |
| [2. Populations](02-populations.md) | Six hares, two foxes, four grass patches and species summaries. | Read one aggregate test with me; no biological implementation required. | Scenario setup, deterministic reset, summaries and tests. |
| [3. Choosing food](03-food-choice.md) | A hungry grazer names a nearby target while standing still. | Choose an eligible patch, with a stable tie rule. | Prepare activity data, fixtures, query wiring and inspection. |
| [4. Paying to move](04-movement.md) | A grazer takes affordable steps toward its target. | Change position and charge for actual distance together. | Add species travel settings and schedule/inspector plumbing. |
| [5. A finite meal](05-eating.md) | Grass decreases when an animal actually receives energy. | Bound the transfer by food, bite size and free capacity. | Prepare contact/conflict fixtures and actual-outcome reporting. |

Only Chapter 1 is active. Later chapters are worked previews of the selected
direction, with explicit preparation steps. Their new functions do **not** exist
in the live code yet. Before each chapter, I will reconcile the guide with your
latest implementation, prepare the mechanical pieces, and leave one edit ready.
You do not need to design those prerequisites yourself.

## Where the code lives

All paths below are relative to `/Users/nick/Code/moss`.

| File | Read it as |
| --- | --- |
| `crates/moss-sim/src/lib.rs` | The vocabulary: components, shared configuration, tick order and reset. |
| `crates/moss-sim/src/lessons.rs` | The rules you are changing. Maintenance works; the three later systems are unscheduled stubs. |
| `crates/moss-sim/src/fixture.rs` | The authored starting conditions. |
| `crates/moss-sim/tests/bootstrap.rs` | A renderer-free world exercising the real installed schedule. |
| `crates/moss-web/src/browser.rs` | Browser controls and a read-only view of simulation state. |

An **entity** identifies one thing. A **component** is data attached to it, such
as its `Energy`. A **resource** is one shared value in the world, such as species
settings. A **system** is a function Bevy calls with the components/resources it
requests. The **schedule** says which systems run, and in which order.

Think of a query as selecting rows by the data they carry. Fern is a hare because
her `Species` value is `Hare`, not because her nickname says anything biological.
Meadow is a grass patch with biomass. It is not an animal with an energy reserve.

## The working rhythm

1. Open the one file named by the checkpoint. Make that edit only.
2. Run its focused test. A red test is useful when its failure is the intended missing behavior.
3. Read the result: **`0 passed` / zero tests run is not a checkpoint**. Check the test name/filter.
4. When green, send the review request at the checkpoint. Green is a place to review and stop, not an instruction to start the next chapter.

Cargo may also report zero **doc-tests** after the real tests; that is normal.
The focused test named in the chapter must actually appear and run.

Use RustRover's **Run** triangle, including the gutter beside an individual
`#[test]`. Native Debug currently stalls on this machine; [the IDE guide](../RUSTROVER.md)
records that limitation. Terminal commands in these chapters are the dependable
fallback; run them from the workspace root.

The agent runs the broader checks, handles formatting/build/browser plumbing,
reviews correctness and Rust usage, and updates `NOW.md`. For a completed behavior,
we also check it through `install` + `tick` and in the browser. A helper test alone
cannot prove that a system is scheduled, filters the right entities, or resolves
competition correctly.

If a test fails unexpectedly, send its output and your current edit. You do not
need to diagnose the whole project before asking. Complete worked examples are
included; copying one and walking through it together is a valid way to resume.

## What comes after these chapters

First finish finite eating. Then the recommended environmental sequence is
bounded grass renewal under constant light, followed by a tick-driven day/night
cycle, then an authored cloudy interval. Neighbor-based plant spread, broader
weather, starvation, fatigue and reproduction remain separate decisions and
exercises. Zero energy continues to mean zero reserve; we have not defined death.

The [ecology plan](../ECOLOGY_PLAN.md) holds those details. You do not need it open
for Chapter 1.

## Evidence for this guide

The guide targets this repository's pinned Rust 1.93.1 and Bevy ECS 0.18.1.
Worked code is checked in an isolated copy; it is not installed into the live
simulation by writing this guide. The validation record below distinguishes
tested examples from the future integration/browser acceptance checks.

<!-- tutorial-validation -->
Verified September 21, 2026:

- Chapter 1A's resource/default example compiles with the existing six simulation tests passing.
- Chapter 1B's new regression fails for the intended reason: Fox actual 57 versus expected 54. Chapter 1C's worked system makes it pass; all seven simulation tests then pass.
- Each helper test in Chapters 3–5 fails against its unfinished stub and passes against the exact worked function. The food-choice test also checks that a closer patch beats a lower-ID, farther patch.
- A separate temporary regression confirms reset preserves custom species rates. With all examples and that check installed in the temporary copy, 11 native simulation tests pass. Formatting and simulation Clippy with warnings denied also pass.
- The live repository's six simulation tests still pass. No live Rust source, dependency or biological behavior was changed for this guide.

Commands used in the temporary copy: the focused `cargo test` commands printed
in the chapters, `cargo test -p moss-sim --locked`, `cargo fmt --all`, and
`cargo clippy -p moss-sim --all-targets --locked -- -D warnings`, through the
repository's toolchain wrapper. Two read-only subagent reviews checked API
accuracy, test coverage and the size of the learning steps.

**Not yet implemented or verified:** population scaffolding, the installed
choice/movement/eating systems, their future integration tests, and the browser
acceptance checks described in these chapters. The helper examples do not claim
those later behaviors work. No new WASM/browser smoke test was run for this
documentation-only change; the current foundation's evidence remains in
[build notes](../BUILD_NOTES.md).
