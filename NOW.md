# Now

**Active session: help Fern reach Meadow.** Start with
[today's v2 lesson](docs/tutorial/today-v2.md), designed for roughly 60–90 minutes.
Maintenance works. Movement's target, rate, ECS adapter, tests and inspector
are prepared; the movement rule is still your edit and is not scheduled.

For an illustrated companion, the [published Moss book](https://nicholashazen.github.io/moss/)
opens with this same affordable-step exercise. It adds experiments and the longer
story; it does not add another prerequisite or change today's assignment.

## One next edit — an affordable step

In [`lessons.rs`](crates/moss-sim/src/lessons.rs), replace only the body of
`move_one_cell`. [The v2 lesson](docs/tutorial/today-v2.md#3-write-one-affordable-step)
explains the rule, mutable borrowing and the complete worked answer. Use
**Moss - Movement test** in RustRover's Run menu, or:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test movement movement_charges_only_an_affordable_actual_step -- --exact
```

**Checkpoint:** this one test intentionally fails at `todo!()` before your edit.
Aim for one passing test, then send it for review. The agent reviews the rule,
enables the prepared system and two integration tests, rebuilds the browser and
checks the journey. Full checks currently stop at that same unfinished exercise;
the runnable browser remains maintenance-only.

**Core stopping point:** nine Steps take Fern to Meadow with reserve 33; the
tenth stays there with 32. These are expected results after activation, not live
browser observations yet. A finite meal is the optional stretch, prepared after
review. Choice, population scenarios, individual-cost refactoring and death
are outside today's first edit. Animals still remain alive at zero.

## Verified run path

From `/Users/nick/Code/moss`:

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

Open <http://127.0.0.1:8080>, or use **Moss - Browser preview**. Keep one preview
on that port. Stop with Ctrl-C or the IDE Stop button. The wrapper handles this
Mac's compiler environment; [IDE notes](docs/development/rustrover.md) describe
Run actions and the native debugger's verified coverage.

[Current verification](docs/development/verification.md) separates the intentional
red exercise, isolated worked-answer checks and browser evidence. The
[ecology plan](docs/design/ecology.md) keeps the longer direction; the
[session log](docs/history/sessions/2026-09-22.md) records why today's route changed.
