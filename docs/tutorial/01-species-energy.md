# 1. Put passive burn in species settings

[Guide home](README.md) · Next chapter: [populations](02-populations.md)

**Ready:** checkpoint A matches the current code. It adds a type and its defaults;
the loop change follows after review. The [verification record](authoring/verification.md)
names the source revision and separates tested examples from future browser work.

Fern and Flint both reach 57 energy after three ticks. The shared maintenance
loop makes that result easy to explain: each animal started at 60 and paid 1
three times. But suppose a fox should cost twice as much to sustain. Changing
the literal in that loop would make the hare pay twice as much too.

We need two facts to meet at the subtraction: which species this animal belongs
to, and the passive rate configured for that species. The individual already
carries `Species` and `Energy`. What is missing is a shared place for the rates.
That is the resource we will add first. Both defaults stay at 1; a separate test
will choose Hare = 1 and Fox = 2 and expect 57 and 54.

**Today's edit:** add the resource and its defaults in checkpoint A, run the
existing tests, then stop. The remaining checkpoints are available when you
want the next session; no preparation reading is required.

## On this page

- [A: describe the settings](#checkpoint-a--describe-the-settings)
- [B: give the test a different rate](#checkpoint-b--give-the-test-a-different-rate)
- [C: read the setting in the loop](#checkpoint-c--read-the-setting-in-the-loop)
- [Complete the behavior together](#complete-the-behavior-together)
- [Optional: one setting, several animals](#optional-one-setting-several-animals)

## Checkpoint A — describe the settings

Open [lib.rs](../../crates/moss-sim/src/lib.rs). Find `pub struct Energy` and add this **after
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

We could put a rate on every `Energy` component, but then changing the hare
default would mean tracking several copies. For now all hares share a species
setting, so one resource expresses the intended relationship. Later, an
individual inherited metabolism could justify per-animal data. That would be a
new fact about the animal, rather than another copy of today's shared default.

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

Now open [bootstrap.rs](../../crates/moss-sim/tests/bootstrap.rs). Add `SpeciesEnergyRules` to its
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

Open [lessons.rs](../../crates/moss-sim/src/lessons.rs). Extend its `use crate::{...};` import to
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

## Optional: one setting, several animals

Imagine three hares with starting reserves 60, 40 and 1, sharing a rate of 2.
After one tick they should have 58, 38 and 0. The shared setting does not require
equal reserves, and the last hare still uses the lower bound you already wrote.
This is a paper prediction, not an additional implementation assignment.

That same distinction is what makes the next population chapter useful: six
hares can share the rule while their individual state diverges. If the borrowing
syntax is the unfamiliar part, [the Rust companion](context/rust-at-point-of-use.md)
explains it using this loop; [the ECS companion](context/ecs-in-moss.md) follows
how the resource reaches the system.

[Guide home](README.md) · Next after review: [populations](02-populations.md)
