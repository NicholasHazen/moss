# Coding style

These are Moss's project conventions, grounded in the current code. Rust enforces
type, borrowing, and visibility rules; it does not require this package layout,
file size, import grouping, or choice of domain types. Follow the conventions for
new work, and explain a concrete reason when changing an established pattern.

Read [the architecture](../design/architecture.md) for ownership and simulation
boundaries, and [development commands](../development/README.md) for verification.
The active learner-owned edit and stopping point remain in [NOW.md](../../NOW.md).

## Modules follow responsibilities

Keep the two Cargo packages: `moss-sim` owns simulation state and rules;
`moss-web` hosts the browser and presentation. Add modules for cohesive existing
responsibilities. A new package needs a concrete dependency or ownership boundary. A new
dependency needs useful functionality the project does not already provide;
file organization alone justifies neither.

Keep crate roots easy to scan. Simulation implementation modules are private,
with explicit root re-exports preserving imports such as `moss_sim::Energy`:

```rust
mod energy;
pub use energy::{Energy, SpeciesEnergyRules};
```

See [lib.rs](../../crates/moss-sim/src/lib.rs). Use `name.rs` with `name/` for
child modules, as in [browser.rs](../../crates/moss-web/src/browser.rs). Keep
related types and their implementations together: reserves and maintenance
settings belong together in [energy.rs](../../crates/moss-sim/src/energy.rs).
There is no line-count maximum or requirement to give each type its own file.
Split when a responsibility becomes independently understandable, not to meet a
quota. Avoid empty future modules, generic registries, and speculative frameworks.

Default to private items. Use `pub(super)` for browser interfaces shared through
their parent; use `pub(crate)` for simulation internals needed across modules,
such as [Journal::record](../../crates/moss-sim/src/journal.rs). Reserve public
exports for the intended caller API. Preserve that API during mechanical moves.
`lessons` remains public because its functions are explicit teaching landmarks.

## Data and borrowing express ownership

Use concrete components, resources, enums, and functions. Group fields by what
owns them and which systems read them, rather than building a universal attribute
map. Different numeric values do not require different component types.

Biological components currently expose mutable fields. The rule that only
simulation-owned code changes them is a project ownership boundary, not a
security guarantee enforced by those fields' visibility. Presentation derives
sprites and transforms from state; it does not own another authoritative position.

Borrow what a system needs and make mutations visible in its query signature.
The current maintenance system uses:

```rust
rules: Res<SpeciesEnergyRules>,
mut creatures: Query<(&Species, &mut Energy), With<Creature>>,
```

This is a parameter excerpt from [lessons.rs](../../crates/moss-sim/src/lessons.rs),
not a proposed replacement for Nick's next exercise. Small `Copy` values can be
copied directly. Keep borrows short; collect temporary entity handles when needed
before structural mutation, as [reset](../../crates/moss-sim/src/simulation.rs)
does before despawning. Do not clone data merely to obscure its ownership.

## Names, defaults, and formatting stay ordinary

Use `UpperCamelCase` for types, `snake_case` for modules, functions, and fields,
and `SCREAMING_SNAKE_CASE` for constants. Name units where they clarify meaning:
`maintenance_units_per_tick`, `units_per_pixel`, `now_ms`, and `TICK_SECONDS`.
Keep species, ecological role, nickname, and stable identity distinct.

Derive traits that the type actually supports and uses. Small component values
commonly derive `Clone`, `Copy`, `Debug`, and equality; there is no mandatory
derive list or ordering. Derive `Default` when its values fit the domain, and
write an explicit implementation for authored defaults such as reserve rates.
Do not change those values as part of a style cleanup.

Use the pinned toolchain's rustfmt output; the repository has no custom formatting
configuration. Group imports as standard library, external crates, then local
items when present. Prefer explicit local imports; the established Bevy
`prelude::*` and test `super::*` imports are fine. Let rustfmt handle layout.
Use readable loops for mutations and iterator chains for straightforward
projection; neither style is required everywhere.

## Absence, errors, and arithmetic carry meaning

Use `Result` for invalid configuration that callers can handle, following
[WorldConfig::new](../../crates/moss-sim/src/config.rs). Keep validated fields
private when direct construction would bypass their invariant. Use `Option` for
meaningful absence: a grass maintenance rate is `None`, while `Some(0)` is an
applicable zero cost. Do not replace unknown inspection data with invented facts.

Use explanatory `expect` messages for internal assumptions, as existing counter
overflow checks do. Handle browser input errors at the boundary. Test fixtures
may use `unwrap` for values they deliberately know are valid; neither `unwrap`
nor `expect` is a substitute for handling expected invalid input in runtime code.

Clamp only intentional boundaries. Maintenance uses `saturating_sub(cost)` because
reserves stop at zero. Tick, run, and journal counters use `checked_add` because
overflow would invalidate history. Document bounds that justify numeric casts,
as [fixture.rs](../../crates/moss-sim/src/fixture.rs) does for world dimensions.
Keep units and balance choices explicit; formatting must not change arithmetic.

## Ticks, tests, and explanations stay aligned

Install one simulation in the browser's ECS world. Keep the simulation schedule
single-threaded and explicitly ordered; [simulation.rs](../../crates/moss-sim/src/simulation.rs)
chains maintenance before completing the tick. Reset replaces entities with
simulation IDs and preserves presentation state. Use stable `SimId` values for
history and deterministic conflict rules, rather than relying on ECS iteration order.
Rendering requests complete ticks; biological rules do not read frame time.

Keep unit tests beside local invariants and integration tests under `tests/` for
installed-world behavior. Shared integration setup lives in `tests/common/mod.rs`.
Use literal expected outcomes and population checks so an empty query cannot pass
silently; [maintenance.rs](../../crates/moss-sim/tests/maintenance.rs) is an example.
Preserve learner-authored assertions during cleanup. Add focused regressions for
meaningful rule changes, not tests that repeat the implementation expression.

Native tests, WASM compilation, and browser smoke tests establish different facts.
Keep browser dependencies target-gated and camera/playback math independently
testable. Report the checks actually run using the development guide.

Use `//!` for a module's purpose and `///` for behavior, units, invariants, and
relevant panic conditions. Explain consequential choices in local comments;
put extended teaching in the guide. Maintain examples and paths according to the
[tutorial authoring guide](../../learning/authoring/README.md). The three documented,
unscheduled lesson stubs are intentional landmarks; do not implement or schedule
them during organization work. New speculative stubs are not the default.

Optional language references: the official Rust Reference on
[modules](https://doc.rust-lang.org/reference/items/modules.html) and
[visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
