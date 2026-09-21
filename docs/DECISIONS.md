# Decisions and open questions

**Version:** Starter kit v0.1 · September 20, 2026.

This is a small decision log, not an architecture approval process. Record a change when it affects how we build or what we are promising. Do not require a new document for every ordinary code edit.

## Established by Nick's direction

**Independent fork.** Moss is its own learning-led ecosystem project. It need not prove Aftermarket's economic or RPG framework before it is worthwhile.

**Rust and ECS.** The project is an opportunity to regain hands-on fluency in both.

**Browser-first viewing.** The intended experience has familiar map-style pan/zoom and both close inspection and a broader world view.

**Simple life before abstract cognition.** Eating, rest, reproduction, grazing, prey, and predators grow through small iterations. Deeper attributes, energy tradeoffs, and eventual evolution remain part of the vision.

**Participation matters.** Agents should reduce nonvaluable mechanical work while preserving first-hand knowledge and meaningful involvement. Clear communication and a maintained next step are part of the development approach.

## Starting defaults proposed by this kit

| Default | Why start here? | Revisit when… |
|---|---|---|
| Codename Moss | Short, easy to use, appropriate to a growing ecosystem. | Nick prefers another internal name. |
| Bevy ECS + small Bevy 2D web app | Continues the chosen Rust/ECS direction without inventing engine infrastructure. | An actual browser/API/build limitation merits comparison. |
| Two small crates; one browser ECS world | Separates native-testable rules from browser dependencies without a mirrored simulation. | Real coupling or test friction shows a better split. |
| WebGL2-first validation, Trunk | A concrete web bootstrap hypothesis supported by current project references. | The local compatibility check gives a reason to change it. |
| Small bounded grid; new run to resize | Makes early movement and bounds easy to inspect. | Fractional motion or real geography needs more. |
| Conditional activity rules first | Gets a living loop running before a general scorer. | A reproducible case demands competing priorities or planning. |
| Pause on hidden; no catch-up | Keeps the observed experiment under explicit playback control. | Background simulation becomes an intentional feature. |
| Bounded memory history before a database | Keeps observation useful without turning setup into persistence engineering. | Long runs or saved experiments need durable storage. |

The workspace, grid, WebGL2/Trunk path, hidden-page suspension, and bounded journal
are now implemented. Activity rules remain future work. Tested versions and
runtime evidence are in [BUILD_NOTES.md](BUILD_NOTES.md).

## Foundation review — September 20, 2026

**Shared maintenance, individual observation.** The first paired exercise will
apply to entities with `Creature` and `Energy`; Fern is the observation anchor.
Flint pays the same initial one-unit-per-tick cost. IDs serve selection/history,
not eligibility for ordinary physiology. Future differences in rates should be
data introduced alongside their consequences.

**Concrete roles and simulation-owned names.** `CreatureKind::{Grazer, Hunter}`
replaces stringly typed roles, and `FoodPatch` owns Meadow's name. These labels
describe the authored fixture; they do not commit us to final species or a
generic behavior framework.

**Finite food first, explicit renewal next.** Eating must consume real biomass.
Patch replenishment will be its own small rule after the first meal, with units,
a cap, and ordering, before using it to support growing populations. Rest,
starvation, hunting, and reproduction remain separately testable additions.

## Taxonomy and model clarification — September 20, 2026

Nick identified ambiguity between names, species/race, archetypes, and displayed
sizes. `Species::{Hare, Fox, Grass}` and `EcologicalRole::{Grazer, Hunter, Producer}`
now hold separate concepts. Fern/Flint are nicknames; Meadow labels a grass patch.
Species + ID leads the inspector and list. This supersedes `CreatureKind` from
the earlier review. Bevy's “archetype” remains a component-set term.

Uniform schematic markers replace arbitrary unequal rectangles. The renderer
shares a glyph-size constant across drawing, picking and outlines; no physical
size is implied. Four documented, unscheduled lesson stubs were added at Nick's
request; their bodies and focused biological tests remain paired work.

The selected future defaults are a second population fixture, cell-local finite
plant patches before CA spread, constant-light growth then a tick-driven day,
and authored clouds before more complex weather. Full scope, ordering, and
acceptance examples live in [ECOLOGY_PLAN.md](ECOLOGY_PLAN.md).

## Still unresolved

Final species design and detailed balance; reproduction eligibility and costs; local observation rules; geometry beyond initial cell contact; how attributes affect energy; durable storage; inheritance/mutation; mobile input scope; world generation; performance targets; and any commercial game objective.

Choose these at the point of use. Do not block bootstrap to settle them all.

## Provenance

The product direction comes from Nick's conversation culminating in the browser-first ecosystem fork request. Aftermarket's `PROJECT_BRIEF_v0.2.md` supplied the prior context for inspectability and living feedback, not an inherited delivery contract. Moss does not require that source file in its new repository.

Technical facts and contemporary web caveats are separated in [RESEARCH.md](RESEARCH.md). All example creatures, formulas, and architecture defaults in this kit are design proposals rather than external scientific findings.
