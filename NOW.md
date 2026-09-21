# Now

**Status:** Nick's first biological rule and focused regression are complete. Hare #1 (Fern) and Fox #2
(Flint) each spend one energy unit per executed tick, bounded at zero. The
single-threaded schedule explicitly runs maintenance before completing the tick.
Movement, feeding, and death remain unimplemented. Later scope is in the
[ecology plan](docs/ECOLOGY_PLAN.md).

## One active task — configurable passive burn (paired; not started)

**Mode:** Pair. The first maintenance exercise is finished: Nick wrote the
rule and its regression, which checks both creatures at 60 initially, 57 after
three ticks, and zero after 63 ticks while both still exist. The test exercises
the installed schedule. Its commented API reference remains for returning to Rust.

**Start here:** [Tutorial Chapter 1, checkpoint A](docs/tutorial/01-species-energy.md#checkpoint-a--describe-the-settings).
It has the exact code, a short Rust refresher, test command and copyable review
request. The [TutorBro guide home](docs/tutorial/README.md) maps the next five
chapters; only checkpoint A is active today. In RustRover, open that README's
Markdown preview. The [context shelf](docs/tutorial/context/README.md) is optional
background, and [authoring notes](docs/tutorial/authoring/README.md) tell the agent
how to keep the guide aligned as plans and code change.

**Open:** `crates/moss-sim/src/lib.rs` beside the existing `Energy` component.

**One concrete next edit:** Define a small `SpeciesEnergyRules` resource with
named Hare and Fox passive rates, both initially 1 energy unit per tick. A
resource is shared world configuration; `Energy` is each individual's reserve.
Add the resource and its explicit `Default` implementation only, then run
`scripts/with-toolchain.sh cargo test -p moss-sim --locked`. Expect the existing
six simulation tests to pass. Stop and send “Chapter 1A is green” for review.
This first checkpoint changes no behavior. The agent handles installation at
review; reading `Species` in the loop and inspector plumbing follow afterward.

Keep the existing default-rate regression. Add a focused example with Hare=1,
Fox=2: three ticks from 60 yield 57 and 54, and additional ticks clamp both at
zero without despawning either animal. Rates stay fixed for a run; reset when
configuration changes. This plan is recorded, not implemented yet.

**Expected browser observation:** Reset, Step three times, and inspect either
animal: 57 / 100. Positions and grass biomass remain unchanged. Zero causes no
death yet; Reset restores 60 / 100.

**Today's stop:** The resource/default edit compiles and has been reviewed.
**Chapter completion:** Species rates work in the native test and are visible in the browser.
Then return to the planned mechanical population scenario: 6 hares, 2 foxes,
4 patches, deterministic reset, and read-only species summaries. Food choice,
movement, and eating follow as paired exercises. Death remains separate.

**Energy direction:** Baseline maintenance now runs. When movement is implemented,
add species-configured cost per cell actually traveled, with an affordability
check after maintenance. Calculate distance where authoritative movement occurs;
no permanent distance component is needed merely to charge a step. Add other
named costs with completed actions; choosing an activity does not incur travel.

## Verified run path

Working directory: `/Users/nick/Code/moss`.

RustRover EAP now has shared **Moss - Maintenance test**, **Simulation tests**,
**All tests**, **Full checks**, **Browser build**, and **Browser preview** actions.
The IDE owns the active localhost:8080 preview; stop it with the red Stop button
before launching a terminal preview. See [the IDE guide](docs/RUSTROVER.md) for
the local Mac environment setting and the shared configurations.

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

Open <http://127.0.0.1:8080>. The wrapper handles this Mac's Command Line Tools selection and Trunk's `NO_COLOR` parsing. Omit `--release` for later debug iteration after its first compile.

```sh
scripts/check.sh
scripts/with-toolchain.sh trunk build --locked --release
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

For the September 21 coding session, RustRover's live-reloading Trunk preview is
running at <http://127.0.0.1:8080>. Use that while editing; the optional static server at
8081 serves only the most recently built bundle. Restart Trunk with the first
command above when needed. Stop a terminal server with Ctrl-C.

## Evidence and code map

The [new tutorial](docs/tutorial/README.md) has checked examples for species
rates and the later choice/movement/eating helpers. Its native red/green checks
ran in a temporary copy; no future rule was installed in the live project.
The live six simulation tests also pass. Future browser and full-schedule checks
are explicitly acceptance steps, not claimed results.

`scripts/check.sh` passes all 11 tests, formatting, native/WASM Clippy with warnings
denied, and JS syntax. RustRover's **All tests** also reports 11 passed; **Full
checks** and **Browser build** finish with exit code 0. The IDE preview serves
the browser, where three Steps again showed Hare #1 at 57 / 100. The focused test
passes separately. Exact evidence and limitations are in [build notes](docs/BUILD_NOTES.md).

**IDE limitation:** native Debug stalled while instantiating the maintenance
test; Run mode works. The IDE guide records the next debugger diagnostic. This
does not block the species-rate edit or normal test/build/preview workflow.

**Git checkpoint:** local `main` starts with `9469f3d`, the browser foundation and
tested maintenance rule. The IDE setup and energy plan are a separate follow-up
commit. No remote or public deployment was created.

To revisit just this exercise:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap maintenance_spends_energy_and_stops_at_zero
```
