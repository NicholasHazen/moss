# Now

**Status:** Nick's first biological rule and focused regression are complete. Hare #1 (Fern) and Fox #2
(Flint) each spend one energy unit per executed tick, bounded at zero. The
single-threaded schedule explicitly runs maintenance before completing the tick.
Movement, feeding, and death remain unimplemented. Later scope is in the
[ecology plan](docs/ECOLOGY_PLAN.md).

## One next task — authored population scenario (not started)

**Mode:** Scaffold. The paired maintenance exercise is finished: Nick wrote the
rule and its regression, which checks both creatures at 60 initially, 57 after
three ticks, and zero after 63 ticks while both still exist. The test exercises
the installed schedule. Its commented API reference remains for returning to Rust.

**Open:** `crates/moss-sim/src/fixture.rs` for the next mechanical slice.

**One concrete next edit:** Extract the creature-spawn block into a small helper
that accepts identity, species/role, and position, keeping the diagnostic scene
unchanged. Reuse it for the planned separate scenario with 6 hares, 2 foxes, and
4 grass patches. The full slice adds scenario selection, deterministic reset,
and read-only species counts and energy summaries; see the ecology plan.

**Expected browser observation:** Reset, Step three times, and inspect either
animal: 57 / 100. Positions and grass biomass remain unchanged. Zero causes no
death yet; Reset restores 60 / 100.

**Stop:** This session ends with the tested maintenance rule. For the next
mechanical slice, stop when both scenarios reset reproducibly and their displayed
summaries match individual inspection. Food choice remains the next paired
biological exercise; movement, eating, and starvation remain separate work.

**Energy direction:** Baseline maintenance now runs. When movement is implemented,
add an extra cost per cell actually traveled, with an affordability check after
maintenance. Other completed actions can add their own costs later; choosing an
activity does not itself incur movement cost.

## Verified run path

Working directory: `/Users/nick/Code/moss`.

```sh
scripts/with-toolchain.sh trunk serve --locked --release
```

Open <http://127.0.0.1:8080>. The wrapper handles this Mac's Command Line Tools selection and Trunk's `NO_COLOR` parsing. Omit `--release` for later debug iteration after its first compile.

```sh
scripts/check.sh
scripts/with-toolchain.sh trunk build --locked --release
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

For the September 21 coding session, the live-reloading Trunk preview is running
at <http://127.0.0.1:8080>. Use that while editing; the optional static server at
8081 serves only the most recently built bundle. Restart Trunk with the first
command above when needed. Stop a terminal server with Ctrl-C.

## Evidence and code map

`scripts/check.sh` passes all 11 tests, formatting, native/WASM Clippy with warnings
denied, and JS syntax, including Nick's completed maintenance regression. The
focused test also passes on its own. Trunk and browser behavior were verified
when the rule was wired; the test-only edit did not repeat the browser smoke test.
Exact evidence and limitations are recorded in [build notes](docs/BUILD_NOTES.md).

To revisit just this exercise:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap maintenance_spends_energy_and_stops_at_zero
```
