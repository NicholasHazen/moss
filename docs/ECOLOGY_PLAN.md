# Ecology model — individuals, populations, plants, and environment

**Status:** Selected reversible design defaults, September 20, 2026. Only the
taxonomy labels, schematic view, diagnostic fixture, and baseline maintenance
are implemented. Nick's maintenance regression passes; later
biological rules below remain planned work. `NOW.md` is the one active task.

## Identity is not species, and species is not a role

| Concept | Example | Responsibility |
| --- | --- | --- |
| Stable identity | Run 1, `SimId(1)` | Selection, event participants, later lifetime records. |
| Individual nickname / patch label | Fern / Meadow | A readable label; never rule eligibility or grouping. |
| Species | Hare, Fox, Grass | Group like organisms for species counts and later authored defaults. |
| Ecological role | Grazer, Hunter, Producer | A current food-web role; multiple species can share it. |
| ECS archetype | Entities with the same component types | Bevy storage/query terminology; not a race, species, or personality. |

Use **species** in code and UI. A later fantasy setting can display “race” for
appropriate groups without merging species, ancestry, and behavior into one field.
Hare/Fox/Grass are provisional game species, not calibrated biological models.

The browser now reads **Hare #1 — nicknamed Fern**, **Fox #2 — nicknamed Flint**,
and **Grass patch #3 — labeled Meadow**. Each has separate `Species` and
`EcologicalRole` components. The two creatures share the same component set
even though their species/role values differ. `Creature` marks animals for
shared maintenance. Neither the name nor a species-specific switch decides
who pays that maintenance. Nicknames may eventually be optional or duplicate;
run-scoped IDs remain unique.

## Energy rates belong to species settings; reserves belong to individuals

**Next paired refinement, not implemented:** introduce a concrete
`SpeciesEnergyRules` resource with named Hare and Fox settings. A resource is one
shared value in the ECS world; `Energy` remains a component on each creature.
Begin with only `maintenance_units_per_tick`, defaulting to 1 for both species.
Maintenance reads each creature's `Species` and looks up that setting. Grass
does not acquire animal maintenance merely because it has a species label.

The [tutorial](tutorial/01-species-energy.md) starts with one flat resource:
`hare_maintenance_units_per_tick` and `fox_maintenance_units_per_tick`, plus a
`maintenance_units_per_tick(species)` lookup. This keeps the first edit small;
grouping settings into a plain per-species struct can follow when more fields
make that clearer. Neither form requires a registry or per-individual copies.

Keep these settings fixed during a run; changing them starts a new run. Initially
configure them in Rust, with a test override; a browser settings editor is a
separate convenience. The inspector should report the applicable passive rate.
Do not copy shared defaults into each animal until individual metabolic traits
or inheritance give those copies a distinct meaning.

The regression experiment sets Hare to 1 and Fox to 2 units per tick. Starting
at 60, three ticks must leave 57 and 54. Both still clamp at zero and remain
present; the existing default-rate regression remains useful. Rates are per
executed simulation tick (currently 0.25 simulated seconds), never per frame.

When movement arrives, extend the same species settings with
`movement_units_per_cell`. Add other explicitly named action costs when their
actions are implemented. These are incremental costs above passive maintenance.
There is no generic action-cost registry or unused field inventory to build now.

The movement system knows the old position and the accepted destination. For
the first cardinal step, actual distance is the sum of absolute x/y changes in
cells: zero or one. Check affordability after maintenance, then apply position
and its checked distance-times-rate cost together, once. Rejected movement,
missing targets, and repeated decisions incur no movement charge. Clamping a
charge is not permission to move farther than the available energy buys.

No persistent distance component is needed just to charge a move. If we later
want travel history or aggregate distance, record the actual movement outcome.
For multi-segment travel, sum segment lengths; an out-and-back journey has zero
net displacement but nonzero distance. Camera motion and visual interpolation
never enter this calculation. Fractional rates, activity-duration costs, and
individual modifiers wait until a concrete rule needs them.

The later tutorial chapters use **proposed demonstration settings**, still
unimplemented: start seeking below 45 energy, stop at 75, preserve seeking state
between those thresholds, perception radius 10 cells, movement x before y,
travel cost 2 per cell in examples, and a bite limit of 4 biomass with a 1:1
biomass-to-energy conversion. These make the examples reproducible; they are not
balance claims. Review them when preparing each paired chapter. Maintenance
precedes choice, so threshold checks use post-maintenance reserves.

## Multiples follow the energy refinement

Keep the three-entity **diagnostic fixture** for isolating a rule. Immediately
after the small species-rate refinement, add a separate authored **population
fixture**: **6 hares, 2 foxes, 4 grass patches** in the same 32 × 20 world.
The counts are a small experiment, not a claim of sustainable balance.

That mechanical slice will add a scenario choice that starts a new run, configured
counts, deterministic placements/ID assignment, and a read-only population
summary. No breeding, randomness, or procedural generation is needed to obtain
multiple individuals. Fern/Flint can remain the first named individuals; labels
for the others should use species + ID rather than requiring invented names.

First aggregates: creature counts and mean/minimum/maximum energy **by species**;
plant patch count and total biomass separately. A patch is a stand of plants,
so four grass patches must not be described as four individual grass plants.
Later activity proportions, births, deaths, and distributions use actual data
when their rules exist. Group by role separately when answering food-web questions.

Acceptance: reset reproduces counts, positions and IDs; all six hares receive
the shared maintenance rule; totals equal the individually inspected values.
The population fixture can expose aggregate energy before it has group behavior.
After foraging, it can reveal crowding and competition for finite food.

## Meadow is a resource patch; growth and spread are different rules

Today Meadow is one entity at cell (16, 13) with 80 biomass units. It neither
grows nor spreads. The selected first model keeps one patch per occupied plant
cell, with available biomass and, when renewal is added, capacity and a growth
rate. Animals may share its cell; occupying a plant cell is not a collision.

First implement finite eating. Then add local replenishment, initially under
constant full light so its arithmetic is easy to inspect. Start with a proposed
capacity of 100 and full-light growth of 1 biomass unit per tick; these are
editable demonstration rates. Zero biomass means a cropped stand that can regrow,
not an eradicated species. Plant death/seed availability is a later separate rule.

A cellular automaton (CA) would make the next state of a cell depend on its
neighbors. We do **not** need one just to increase biomass locally. If plant
colonization becomes interesting, introduce a separate spread step: e.g. grass
establishes in eligible neighboring cells. Read the old grid and commit the next
grid together so traversal order cannot make newly grown plants spread again
in the same tick. Specify neighborhood, boundaries, contested cells, and resource
costs then. Do not build a general CA engine now.

Acceptance for renewal: growth respects capacity, eating never exceeds available
biomass, and camera changes cannot affect either. Plant biomass and animal energy
have different units; eating gets an explicit conversion and records actual
transfers. Sunlight supports plant production; animals obtain energy through food.

## Geometry has three separate meanings

**Location now:** each simulation entity has an integer cell position. There is
no body mass, radius, collision, or physical footprint. Sharing a cell is allowed.
The first movement rule uses one cell per tick with a documented axis/tie choice;
the first eating rule requires same-cell contact with the accepted target.

**Drawing and selection now:** all markers are equal 0.8-cell squares, colored
by role. One renderer constant controls glyph size, square hit testing, and the
selection outline. A minimum click target helps at distant zoom. These symbols
and click targets have no physical effect. The previous different rectangle
sizes were arbitrary presentation choices and have been removed.

**Physical extent later:** when body size changes an interaction, introduce a
simulation-owned circular radius in cell units and define contact/bounds against
it. A plant stand can use a separate cell footprint. Add mass or size-dependent
costs only alongside a rule that uses them; do not infer them from sprite area.
Detailed silhouettes and animation remain presentation. Cell sharing can remain
allowed until avoidance/collision becomes an intentional feature.

Acceptance: sprite style, zoom, and click tolerance never change meal eligibility.
When radii arrive, test exact contact and edge cases before using them for costs
or movement. A change in units/geometry is a deliberate model change.

## Day/night drives light, and weather later modifies it

After constant-light renewal works, introduce **one simulation-owned environment
resource**. Derive daylight from executed ticks and configured ticks per day;
never use the computer's clock or rendering brightness as biological input.
Suggested demonstration: 240 ticks per day, first half day and second half night.
At four ticks per second this gives a one-minute cycle at normal playback.
It is a useful viewing speed, not a mapping to real-world physiology.

Start with full daylight / zero daylight. At future tick `n = completed + 1`,
compute phase `n % ticks_per_day`, then use that light for that tick's growth.
Thus completed tick 120 displays night and tick 240 displays day. Update the
environment, replenish plants, resolve creature maintenance/activities/eating,
then commit the clock and post-tick observations. Growth is available to eat in
the same tick. These ordering choices become explicit tests when implemented.

Pause and hidden-tab suspension freeze the day along with the rest of the world.
Smooth dawn/dusk may follow; fractional growth must carry remainders so a low
rate does not truncate permanently to zero. The inspector should expose day,
phase, effective light, and the growth actually produced before visual tinting
tries to communicate them.

**Weather follows the light loop.** Begin with an authored clear → cloudy → clear
sequence, with clouds reducing effective light. Add seeded weather only after
that deterministic example works. Rain may change soil moisture and temperature
may modify maintenance later, each through a named state, units, and one actual
rule. Rain does not automatically grant animal energy, and an unused weather
label must not imply effects. Shelter/local conditions can follow real need.

Acceptance: Step crosses daylight boundaries at exact ticks; equal executed
ticks and inputs give equal light/biomass in native tests and the browser;
cloudy conditions affect growth through the displayed light value. Test weather
effects individually before combining them.

## The first code landmarks

[`src/lessons.rs`](../crates/moss-sim/src/lessons.rs) contains four documented,
compiled rule landmarks: `spend_energy`, `choose_food`, `move_to_food`, and
`eat_food`. Maintenance is now implemented and scheduled; the remaining three
are **unimplemented, unscheduled** stubs. Their signatures grow with the paired
data model. They fail explicitly if called prematurely rather than masquerading
as completed rules. No stub is registered in the running schedule. Nick's focused
maintenance regression now passes beside the existing bootstrap tests.

[`src/fixture.rs`](../crates/moss-sim/src/fixture.rs) owns the starting individuals.
[`src/lib.rs`](../crates/moss-sim/src/lib.rs) owns components and schedule wiring.
The first rule and its regression were Nick's paired implementation work.
Population setup is a mechanical slice between lessons 1 and 2; constant-light
growth, day/night, and weather follow lesson 4 as separate future slices.
