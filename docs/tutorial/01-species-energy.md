# 1. Put passive burn in species settings

[Guide home](README.md) · Next chapter: [populations](02-populations.md)

**Today's smallest useful edit is checkpoint A.** It adds a type and its defaults.
No loop changes yet. The rest of this chapter is here for a later sitting, or
for continuing after review if you want to.

We currently have individual `Energy` components and one literal `1` in
`spend_energy`. We want one shared configuration saying how much a hare or fox
pays per executed tick. Both defaults stay at 1. A test will deliberately choose
Hare = 1 and Fox = 2 to prove the distinction works.

## Checkpoint A — describe the settings

Open `crates/moss-sim/src/lib.rs`. Find `pub struct Energy` and add this **after
its closing brace**, before the next type. The existing Bevy prelude import
already provides `Resource`.

<!-- example: energy-resource -->
```rust
/// Animal maintenance costs in energy units per executed simulation tick.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeciesEnergyRules {
    pub hare_maintenance_units_per_tick: u32,
    pub fox_maintenance_units_per_tick: u32,
}

impl Default for SpeciesEnergyRules {
    fn default() -> Self {
        Self {
            hare_maintenance_units_per_tick: 1,
            fox_maintenance_units_per_tick: 1,
        }
    }
}
```

`#[derive(Resource)]` lets Bevy store this type as shared world data. The other
derives allow copying, printing in failed assertions, and comparing these small
values. `impl Default for ...` supplies the initial configuration; `Self` means
`SpeciesEnergyRules` here. We write the default explicitly because an automatically
derived default would make both `u32` fields zero.

`u32` is an unsigned integer. The field names carry the units: energy per tick.
We are not specifying energy per rendered frame or per real-world second.
`pub` makes the type and fields usable by the browser crate and integration tests.

Run:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked
```

**Expected:** the existing six simulation tests pass: one unit test and five
integration tests. The new data is not installed or used yet, so the browser
still behaves exactly as before. This checkpoint proves the type compiles;
review will inspect its defaults, and the behavior test comes next.

**Stop and send:**

> Chapter 1A is green. Review the resource and defaults, and explain anything surprising in my Rust. Keep the next rule change for pairing.

## Checkpoint B — give the test a different rate

Resume here after A's review. Add this method beneath the default implementation
in `lib.rs`:

<!-- example: energy-lookup -->
```rust
impl SpeciesEnergyRules {
    pub fn maintenance_units_per_tick(&self, species: Species) -> Option<u32> {
        match species {
            Species::Hare => Some(self.hare_maintenance_units_per_tick),
            Species::Fox => Some(self.fox_maintenance_units_per_tick),
            Species::Grass => None,
        }
    }
}
```

`&self` borrows the shared settings without changing them. `match` must cover
every species. `Option<u32>` is either `Some(rate)` or `None`: animal maintenance
does not apply to grass. `None` does not claim that plant metabolism is free.
The [Rust book's match example](https://doc.rust-lang.org/book/ch06-02-match.html)
is an optional reference for this syntax.

**Agent preparation:** in `install`, immediately after
`world.insert_resource(config);`, add:

```rust
world.init_resource::<SpeciesEnergyRules>();
```

I will make this plumbing edit during A's review, before you begin the rate-aware
system. You do not need to track its installation separately. It creates defaults only if the resource
is absent, so a test can insert settings before installation. `insert_resource`
instead would replace an existing value. See the pinned
[Bevy World API](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/world/struct.World.html#method.init_resource).
Keep configuration fixed during each run; the existing `reset` preserves it.

Now open `crates/moss-sim/tests/bootstrap.rs`. Add `SpeciesEnergyRules` to its
existing `use moss_sim::{...};` import. Keep your original maintenance test.
Add this separate test at file scope:

<!-- example: energy-test -->
```rust
#[test]
fn maintenance_uses_species_rates_and_stops_at_zero() {
    let mut world = World::new();
    world.insert_resource(SpeciesEnergyRules {
        hare_maintenance_units_per_tick: 1,
        fox_maintenance_units_per_tick: 2,
    });
    install(&mut world, WorldConfig::default());

    for _ in 0..3 {
        tick(&mut world);
    }

    let mut energies = world.query_filtered::<(&Species, &Energy), With<Creature>>();
    assert_eq!(energies.iter(&world).count(), 2);

    for (species, energy) in energies.iter(&world) {
        let expected = match *species {
            Species::Hare => 57,
            Species::Fox => 54,
            Species::Grass => panic!("the animal fixture contains grass"),
        };
        assert_eq!(energy.reserve, expected);
        assert_eq!(energy.capacity, 100);
    }

    for _ in 0..60 {
        tick(&mut world);
    }

    assert_eq!(energies.iter(&world).count(), 2);
    for (_, energy) in energies.iter(&world) {
        assert_eq!(energy.reserve, 0);
        assert_eq!(energy.capacity, 100);
    }
}
```

The setup is a real ECS world using the same `install` and `tick` as the browser.
`0..3` visits 0, 1 and 2; `_` says we do not need the loop variable. The query
selects animals and returns a tuple of two read-only component references.
`for (species, energy)` unpacks each tuple. `*species` reads the small `Copy` enum
value behind its reference.

The query variable is mutable because Bevy maintains its matching metadata.
That does not make the requested `&Energy` mutable. Reusing the query after more
ticks is fine: each `.iter(&world)` reads current values. Its iterator borrow
ends before the next `tick(&mut world)`.

Run:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap maintenance_uses_species_rates_and_stops_at_zero -- --exact
```

**Expected red checkpoint:** one test runs and fails with Fox actual **57**,
expected **54**. That means the configuration exists, but the old loop still
subtracts 1. A missing import or missing-resource panic is a setup problem to
resolve first; it is not this intended failure.

If the failure is Fox actual 57 versus expected 54, continue to C. For a
different failure, send me the output before changing the rule.

## Checkpoint C — read the setting in the loop

Open `crates/moss-sim/src/lessons.rs`. Extend its `use crate::{...};` import to
include `Species` and `SpeciesEnergyRules`. Replace only `spend_energy`.

The intended edit is: request the resource, query each animal's species alongside
its mutable energy, look up the rate, then keep the saturating subtraction you
already wrote. Here is the complete version to compare with or copy:

<!-- example: energy-system -->
```rust
pub fn spend_energy(
    rules: Res<SpeciesEnergyRules>,
    mut creatures: Query<(&Species, &mut Energy), With<Creature>>,
) {
    for (species, mut energy) in &mut creatures {
        if let Some(cost) = rules.maintenance_units_per_tick(*species) {
            energy.reserve = energy.reserve.saturating_sub(cost);
        }
    }
}
```

Bevy supplies `Res<T>` when it runs the system. It is read-only access to a
resource, while `&mut Energy` grants access to each matched animal's reserve.
`if let Some(cost)` enters the block only when lookup found an applicable rate.
The `mut energy` binding allows mutation through Bevy's change-tracking wrapper.

Run B's focused command again, then the simulation command from A.
**Expected green:** one focused test passes; seven simulation tests pass in
total. Your original default-rate regression still passes. Both animals still
exist at zero. We have not defined starvation.

**Send:**

> Chapter 1C is green. Review the lookup, query and regression. Please verify reset preserves custom rates, finish the inspector plumbing, and smoke-test the browser before we move on.

## Complete the behavior together

I will update stale comments, verify a reset still uses the configured resource,
and make the inspector report the applicable passive rate. The default browser
run remains **57 / 57 after three Steps**. The test override affects only its own
world. For a visible **57 / 54** experiment, I will configure a separate 1/2 run
before installation/reset, demonstrate it, then restore the default configuration.

With the preview running: Reset, inspect both species, Step three times, compare
their reserves with their displayed rates. Check that positions and grass biomass
are unchanged. Pause must freeze energy. I will run the broader checks and record
the result in `NOW.md` before recommending [Chapter 2](02-populations.md).

The consequence of this design is simple: settings belong to a species, while
the remaining reserve belongs to an individual. Travel costs can extend these
settings when travel exists; we do not need an action registry today.
