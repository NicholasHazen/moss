# Rust at the point of use

[Guide home](../README.md) · [Return to species energy](../01-species-energy.md)

Fern starts with 60 energy. Three simulation ticks leave her with 57. You can
already follow that rule without remembering every Rust keyword around it. This
page explains the syntax that carries the rule and the completed species-setting
edit. Read the section matching the line in front of you; nothing here is a
prerequisite for starting today's edit. The maintenance examples revisit working
code; the same borrowing and module rules apply to movement and later helpers.

**On this page:** [Modules](#where-a-declaration-lives) · [Values](#values-and-defaults) ·
[Derives](#what-the-derives-provide) · [Borrowing](#three-places-mut-can-appear) ·
[Loops](#following-one-creature-through-the-loop) ·
[Optional values](#reading-an-optional-rate) · [Check](#a-check-you-can-run-now)

## Where a declaration lives

`Energy` is defined in [energy.rs](../../../crates/moss-sim/src/energy.rs), while
callers still import `moss_sim::Energy`. The crate root,
[lib.rs](../../../crates/moss-sim/src/lib.rs), connects those locations with this
**existing excerpt**:

```rust
mod energy;
pub use energy::{Energy, MovementRules, SpeciesEnergyRules};
```

`mod energy;` declares a module and loads its body from `energy.rs`. Other files
refer to that module's items rather than declaring it again. Rust's
[module-file convention][module-files] also supports child modules such as
`browser/scene.rs`, declared by `browser.rs`.

The energy module is private. Its `pub` types can be exposed through a
**re-export**, the `pub use` statement that provides the public root paths.
This keeps callers independent of the file arrangement. Within `lessons.rs`,
`use crate::Energy;` refers to that same root item. See the Rust book's
[re-export explanation][re-exports].

Keep declarations near their related data: energy in `energy.rs`, identity and
position in `components.rs`, and schedule wiring in `simulation.rs`. The crate
boundary still separates simulation from the browser; these modules organize
code inside that boundary.

## Values and defaults

In [crates/moss-sim/src/fixture.rs](../../../crates/moss-sim/src/fixture.rs), this existing **expression excerpt** creates
each animal's starting energy. It belongs inside the tuple passed to
`world.spawn`; it is not an additional edit:

```rust
Energy {
    reserve: 60,
    capacity: 100,
}
```

This is a struct literal: the type name followed by named field values. It
creates data. Creating that value does not itself install a rule that spends it;
`spend_energy` supplies that behavior later.

The completed Chapter 1 describes a different struct, `SpeciesEnergyRules`, for the shared
rates. Its `impl Default for SpeciesEnergyRules` block defines what
`SpeciesEnergyRules::default()` returns. Inside that block, `Self` is shorthand
for `SpeciesEnergyRules`. An explicit default keeps both rates at 1; deriving
`Default` for two `u32` fields would instead choose zero. Default values are a
design decision expressed through an ordinary Rust interface.

## What the derives provide

Above `Energy`, `#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]`
generates standard implementations. `Component` makes the type eligible for
storage on an ECS entity. `Debug` lets failed assertions display its contents;
`PartialEq` and `Eq` support equality comparisons.

`Clone` provides explicit duplication. `Copy` allows small values such as our
integer coordinates and species enum to be copied when used by value. In
Chapter 1, `species` is a reference to a `Species`; `*species` accesses the value
behind it. Because `Species` implements `Copy`, passing that value to the rate
lookup leaves the entity's component in place. This does not clone the world or
the creature.

## Three places `mut` can appear

The preceding version of the rule in [crates/moss-sim/src/lessons.rs](../../../crates/moss-sim/src/lessons.rs)
isolates the three uses of `mut`. This **complete earlier function** is for
comparison, not a replacement for the current species-aware implementation:

```rust
pub fn spend_energy(mut creatures: Query<&mut Energy, With<Creature>>) {
    for mut energy in &mut creatures {
        energy.reserve = energy.reserve.saturating_sub(1);
    }
}
```

`&Energy` requests shared access for reading. `&mut Energy` requests exclusive
access that permits changes. Rust prevents overlapping access that would make
those guarantees invalid. The [borrowing chapter of the Rust book][borrowing]
explains this rule with ordinary functions.

Here `mut creatures` lets us borrow the query mutably. `&mut creatures` asks its
iterator for mutable component access. Each resulting `energy` is Bevy's
change-tracking wrapper around one component, so `mut energy` permits the field
assignment through that wrapper. These spellings each describe a different
part of the access path.

If `mut energy` becomes just `energy`, a useful diagnostic is E0596: a value
cannot be borrowed mutably through that binding. Restore `mut` on this local
binding; changing the whole design is unnecessary. This is a specific instance
of [Rust's E0596 explanation][e0596]. Conversely, adding `mut` to a variable
holding `&Energy` cannot turn a shared reference into permission to edit energy.

## Following one creature through the loop

In the current function, each iteration visits one matched animal. Chapter 1
added its species to the requested data. Its **current loop-header excerpt** is:

```rust
for (species, mut energy) in &mut creatures {
```

The parentheses unpack one tuple into two local bindings. They do not create
an inner loop. The query supplies the pair; the body looks up this species'
rate and updates this individual's reserve.

Tests also use `for _ in 0..3`. The range yields 0, 1 and 2; `_` discards those
values because only the three calls to `tick` matter. A `for` loop consumes an
iterator, whether that iterator comes from a range or an ECS query.

In a test, `let mut energies = world.query_filtered::<&Energy, With<Creature>>()`
creates reusable query state. Its `mut` allows Bevy to update matching metadata;
the requested component access remains read-only. Each `energies.iter(&world)`
reads current data. Finish that read before calling `tick(&mut world)`. Retaining
component references and using them after the tick would keep the earlier world
borrow alive. Collect copied values when you need a before/after snapshot.

## Reading an optional rate

The species lookup returns `Option<u32>`: `Some(cost)` for animal maintenance,
`None` for grass. Its `match` covers every `Species` variant, which makes adding
a species a visible decision point. `None` means this rule does not apply; it
does not mean plants have no metabolism.

The current `if let Some(cost) = ...` enters its body only when a rate exists.
A zero rate is still `Some(0)` and reaches the subtraction. With `None`, the body
is skipped. Both may leave the reserve unchanged, but they communicate different
facts about why.

## A check you can run now

From the workspace root, run the existing regression:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test maintenance maintenance_spends_energy_and_stops_at_zero -- --exact
```

An optional prediction: after 63 ticks, does the unsigned reserve wrap around?
The answer is available in the test: both animals remain present at zero because
`saturating_sub` stops there. Expect **one named test passing**. This checks the
current rule with default rates. The separate
`maintenance_uses_species_rates_and_stops_at_zero` test checks custom rates.
Both pass in the live code; [Chapter 1](../01-species-energy.md) records the
remaining browser acceptance work.

[borrowing]: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
[e0596]: https://doc.rust-lang.org/error_codes/E0596.html
[module-files]: https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html
[re-exports]: https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#re-exporting-names-with-pub-use
