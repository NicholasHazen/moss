# Decisions and open questions

**Record:** decisions beginning September 20, 2026. Dated entries describe their
checkpoint and may be superseded below. Current boundaries live in
[architecture](architecture.md), the [ecology plan](ecology.md) and [NOW.md](../../NOW.md).

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
runtime evidence are in [verification](../development/verification.md).

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
acceptance examples live in [ecology plan](ecology.md).

## Species energy settings and development checkpoint — September 21, 2026

Nick requested configurable passive burn and a place for later species/action
costs. The selected next paired edit uses a shared concrete species-settings
resource; reserves remain individual components. Only passive units per tick
are introduced first. Movement adds a cost per actual cell traveled when its
executor exists, and other actions add named costs at their point of use.
Settings remain fixed within a run. This is a planned refinement, not implemented
biology. The original population → food choice → movement → eating route follows.

Initialized local Git and committed the browser foundation, lockfile, and first
maintenance rule/test as `9469f3d` before the IDE/planning changes. Personal
`.idea/` state is ignored; reusable RustRover configurations live in `.run/`.
The Mac's compiler environment belongs in local project settings, preserving
portable shared configurations and leaving the system Xcode selection alone.

## Group energy settings before adding action costs — September 21, 2026

**Superseded by the individual-attributes decision below.** This records the
earlier shared-profile proposal; do not use it as the current implementation task.

The flat species-rate resource and maintenance loop now pass their native
regressions. Nick raised concern about how this representation scales. Recommend
one paired refactor: plain `AnimalEnergyCosts` values nested under `hare` and
`fox` in `SpeciesEnergyRules`. Keep the current maintenance lookup signature,
system behavior, defaults and test assertions. Chapter 1D supplies the edit.
The grouping is planned, not implemented in the live simulation.

This introduces no new biological rule. Per-animal reserves remain components;
shared species configuration remains a resource. Movement later adds its named
cost per accepted cell traveled. A catalog loaded from data, individual traits,
environmental modifiers and fractional costs wait for their concrete use cases.
Complete the grouping before inspector plumbing, then finish Chapter 1 acceptance
and resume populations → food choice → movement → eating.

## Species defaults initialize individual attributes — September 21, 2026

Nick clarified that individual overrides and many future attributes are expected.
Revise Chapter 1D: species configuration supplies spawn defaults, authored
individual overrides take precedence, and each animal owns the resolved values
as a typed component. Begin with `AnimalEnergyCosts`; its first field remains
maintenance units per tick. It can also be the value type in shared templates.
The same type need not be a resource: only the outer collection is shared.

Treat that component as authoritative individual baseline data, not a cache of
the species profile. The future maintenance query reads costs and energy directly.
Shared descriptions and larger definitions still use species/asset keys. Group
components by access and ownership as concrete systems grow. Measure memory and
tick time before claiming any layout is fastest or adding a derived-value cache.

Defaults remain fixed during a run. Reset rebuilds the authored scenario,
reapplying its stored overrides and discarding runtime-only instance changes.
An explicit zero override is valid. Future live retuning needs an explicit policy
for existing individuals, and future inheritance must select which values come
from parents. Temporary effects preserve the baseline and define ordering,
rounding, checked arithmetic and bounds when implemented.

The active paired edit only declares the component and default, leaving the
seven existing tests green. Agent spawn/test preparation then precedes the
paired query change. Its decisive new regression uses two hares with rates 1/2
and expects 57/54 after three ticks, without changing the shared default. Add
reset/override and browser inspection acceptance before population scaffolding.
No live component, override behavior, cache or attribute registry was installed
for this design discussion.

Follow-up [applied research](../research/attributes-and-energy.md) supports this as
an ownership choice, not a universal performance result. Flecs explicitly
supports both shared inheritance and owned instance values; Veloren demonstrates
individual energy state and recomputed temporary modifiers. Keep the proposed
scope and measure before optimizing the layout.

## A visible journey before more architecture — September 22, 2026

Nick asked to restore momentum and learn through a 60–90 minute session.
**This supersedes the earlier immediate implementation order**, while retaining
species defaults → individual typed values as the direction when variation is
introduced. The next edit is one ordinary Rust movement helper; a finite meal
is the stretch. Autonomous choice, populations and individual costs follow
observable behavior rather than blocking it.

Prepare an authored `FoodTarget` on Fern and a concrete `MovementRules` resource
at 2 units per cell. Keep maintenance's current species lookup. These reversible
choices avoid needing activity state or an attribute refactor for one journey.
The existing renderer reads simulation positions; no new rendering engine is needed.

The agent prepares the adapter, tests, inspector and IDE action, but leaves the
helper unfinished and unscheduled. Its normal test is intentionally red; both
installed-movement regressions remain explicitly ignored until reviewed activation.
Maintenance tests retain their assertions using a no-food setup. After review,
activate maintenance → movement → complete tick and verify the browser. Eating
is a separate learner-owned rule; zero energy still causes no automatic death.

The teaching sequence is one behavior, one focused edit, review, then observable
feedback. Keep full worked answers and small diagrams available. Native helper,
installed schedule and browser evidence stay distinct. Stop at a reviewed journey
or meal, with one next action in `NOW.md`.

## Learning in short sessions — September 22, 2026

Nick requested a week-to-month horizon while leaving today's movement session
unchanged. The [new path](../tutorial/path/README.md) contains sixteen independent
20–30 minute focus blocks, grouped by visible outcomes: foraging, individual
variation, renewable supply/light, and explainable scarcity. These are flexible
learning arcs, not promised calendar delivery dates or another active backlog.

Each guide revisits a prior concept, gives one small edit or investigation with
a complete worked reference, and connects it to the next behavior. Agent-owned
preparation happens before the focus block. Reference examples run in a temporary
workspace and never activate live rules. `NOW.md`, today's guide and its movement/
meal chapters remain unchanged during this authoring work.

Keep individual maintenance costs concrete and leave shared travel settings in
place for the first controlled comparison. Growth precedes consumption; binary
daylight precedes fractional weather. The later starvation lesson recommends an
after-meal zero check for explicit pairing; the policy is proposed, not accepted
or installed. First hunting, separate fatigue/rest and funded reproduction are
follow-on arcs rather than oversized final lessons in this month.

## Still unresolved

Final species design and detailed balance; reproduction eligibility and costs; local observation rules; geometry beyond initial cell contact; how attributes affect energy; durable storage; inheritance/mutation; mobile input scope; world generation; performance targets; and any commercial game objective.

Choose these at the point of use. Choose them only when the active work needs them.

## Provenance

The product direction comes from Nick's conversation culminating in the browser-first ecosystem fork request. Aftermarket's `PROJECT_BRIEF_v0.2.md` supplied the prior context for inspectability and living feedback, not an inherited delivery contract. Moss does not require that source file in its new repository.

Technical facts and contemporary web caveats are separated in [research index](../research/README.md). All example creatures, formulas, and architecture defaults in this kit are design proposals rather than external scientific findings.
