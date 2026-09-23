# 1. Put passive burn in species settings

[Guide home](README.md) · Next session: [Fern reaches Meadow](today-v2.md)

**Completed walkthrough, not a new assignment.** The settings, lookup and loop
below are implemented. This page explains the current source; its checkpoint
headings remain so existing bookmarks still work. Recorded native checks cover
Hare 57 / Fox 54 with custom rates after three ticks. Default-rate browser/Reset
checks passed; custom-rate browser/Reset acceptance remains pending agent work.
The [verification record](authoring/verification.md) dates that evidence. Today's
movement helper remains intentionally unfinished and is a separate task.

Fern and Flint both reach 57 energy after three ticks. The shared maintenance
loop makes that result easy to explain: each animal started at 60 and paid 1
three times. But suppose a fox should cost twice as much to sustain. Changing
the literal in that loop would make the hare pay twice as much too.

Two facts meet at the subtraction: which species this animal belongs to, and
the passive rate configured for that species. The individual carries `Species`
and `Energy`; the shared `SpeciesEnergyRules` resource supplies the rates.
Both defaults are 1. A separate test configures Hare = 1 and Fox = 2 to make
the lookup's effect visible as 57 and 54.

**Current stop:** the rate-aware loop is green and reviewed. Continue with
[today's movement helper](today-v2.md). [Individual costs](path/06-owned-costs.md)
remain a later variation experiment in the short-session path, after motion and
food make the difference observable.

## On this page

- [A: describe the settings](#checkpoint-a--describe-the-settings)
- [B: give the test a different rate](#checkpoint-b--give-the-test-a-different-rate)
- [C: read the setting in the loop](#checkpoint-c--read-the-setting-in-the-loop)
- [Complete the behavior together](#complete-the-behavior-together)
- [Optional: one setting, several animals](#optional-one-setting-several-animals)

## Checkpoint A — describe the settings

[energy.rs](../../crates/moss-sim/src/energy.rs) contains this existing declaration
and implementation beside `Energy`. This is a **source excerpt**, with `Resource`
provided by the file's Bevy import; it is not an insertion instruction.

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
`pub` makes the declaration available for export. The current complete energy
re-export in [lib.rs](../../crates/moss-sim/src/lib.rs) also includes the travel
setting prepared for today's movement:

<!-- example: energy-export -->
```rust
pub use energy::{Energy, MovementRules, SpeciesEnergyRules};
```

The browser and integration tests can name `moss_sim::SpeciesEnergyRules`.
The energy module stays private; the re-export exposes the selected type. The
[module explanation](context/rust-at-point-of-use.md#where-a-declaration-lives)
describes that boundary.

All hares currently share a species setting, so one resource expresses that
relationship directly. The later [individual-cost lesson](path/06-owned-costs.md)
will resolve a species default and optional override into an animal-owned
baseline. The distinction matters: a shared setting is read during each tick;
an owned baseline is decided during construction and then read from the animal.

Declaring a resource type alone does not put a value into the world. The next
connection is installation, followed by a system that requests the installed
value. That is why a successful compile was only the first checkpoint when this
feature was originally built.

## Checkpoint B — give the test a different rate

The existing lookup in `energy.rs` connects the species enum to the resource.
This **source excerpt** uses the file's `use crate::Species;` import:

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

`install` in [simulation.rs](../../crates/moss-sim/src/simulation.rs) already
contains this **single-statement excerpt**:

```rust
world.init_resource::<SpeciesEnergyRules>();
```

It creates defaults only if the resource is absent, so a test can insert settings
before installation. `insert_resource`
instead would replace an existing value. See the pinned
[Bevy World API](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/world/struct.World.html#method.init_resource).
Configuration stays fixed during each run; the existing `reset` preserves it.

The following is the **existing complete test function** from
[maintenance.rs](../../crates/moss-sim/tests/maintenance.rs), using that file's
imports and `common::remove_food` helper. It is not a standalone file. Removing
food isolates maintenance so future movement and eating cannot obscure these
expected reserves. The original default-rate regression remains alongside it.

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
    remove_food(&mut world);

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

To revisit this completed behavior, run its focused test from the repository root:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test maintenance maintenance_uses_species_rates_and_stops_at_zero -- --exact
```

**Expected now:** one passing test. Historically, the Fox assertion failed with
57 rather than 54 while the loop still subtracted a hardcoded 1. That failure
showed exactly what the settings declaration had not yet accomplished. The
next section explains the implemented connection. A zero-test result is not
evidence of either behavior; the optional [test-reading guide](context/reading-a-test.md)
walks through the neighboring default-rate regression and explains how to read
its result.

## Checkpoint C — read the setting in the loop

The current `spend_energy` in [lessons.rs](../../crates/moss-sim/src/lessons.rs)
requests the resource, queries species beside mutable energy, and supplies the
looked-up rate to the saturating subtraction. This is the **existing complete
function**, using that file's Bevy and crate imports:

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

For the configured fox, the lookup returns `Some(2)`, so the same loop computes
`60 → 58 → 56 → 54`. For the hare it returns `Some(1)` and computes
`60 → 59 → 58 → 57`. The remaining assertions prove reserve stops at zero and
both animals still exist. Subtraction and death are different rules; this
completed feature implements only the former.

## Complete the behavior together

The inspector reports the applicable passive rate. In the current maintenance-only
browser, Reset followed by three Steps gives **57 / 57**, with positions and
biomass unchanged. The test's 1/2 override affects only its own world; it does not
retune the browser. A visible 57/54 experiment requires a separate configured run.
Custom-rate browser/Reset acceptance remains pending agent work in the
[verification record](authoring/verification.md), not a prerequisite for today's
movement edit or evidence implied by the native test.

Once travel and eating are activated, a no-food scenario will keep this
maintenance experiment isolated, as the native test already does. Shared species
settings supply upkeep; the individual owns its remaining reserve. The separate
`MovementRules` resource supplies the incremental cost of a traveled cell.

## Optional: one setting, several animals

Imagine three hares with starting reserves 60, 40 and 1, sharing a rate of 2.
After one tick they should have 58, 38 and 0. The shared setting does not require
equal reserves, and the last hare still uses the lower bound you already wrote.
This is a paper prediction, not an additional implementation assignment.

That same distinction is what makes the later [population session](path/05-many-individuals.md) useful: six
hares can share the rule while their individual state diverges. If the borrowing
syntax is the unfamiliar part, [the Rust companion](context/rust-at-point-of-use.md)
explains it using this loop; [the ECS companion](context/ecs-in-moss.md) follows
how the resource reaches the system.

[Guide home](README.md) · Current edit: [today's movement](today-v2.md) · After today: [short-session path](path/README.md)
