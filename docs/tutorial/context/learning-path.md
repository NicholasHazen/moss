# Learning path — grow a world by changing one rule

[Guide home](../README.md) · [Context shelf](README.md) · [Current task](../../../NOW.md)

This page connects Moss's behavior to reusable programming ideas. Use
[the 16 short-session guides](../path/README.md) for the actual learning sequence,
and [NOW.md](../../../NOW.md) for the one active edit. The topics below are
connections to revisit when useful, rather than additional lessons to complete.

The recommended route is foraging → populations and individual costs → growth
and daylight → scarcity and lifecycle. A first hunt then follows in the current
recommendation, with its timing revisited after the first live foraging observation;
fatigue/rest and reproduction remain separate later arcs. The short-session
guides own the checkpoints; the [ecology plan](../../design/ecology.md) owns model
choices. This companion explains why the concepts recur along that route.

## The basic rhythm

Observe a behavior, locate its data and rule, change one thing, predict a result
and check it. Keep a focused regression for the useful invariant. The agent
maintains the route and helps directly when needed; compiler errors do not
authorize replacing Nick's implementation with an unexplained solution.

<a id="stop-0--a-place-to-build"></a>

## A foundation makes the source of truth visible

The implemented foundation supplies a visible chamber, identifiable entities
and tick controls. It makes the connection between a component, a tick and the
inspector concrete. That connection remains useful when later examples add
more rules: the source of truth stays in the simulation, while presentation
shows its result. Agent scaffolding maintains the tools, browser glue and test
wiring that make each observation possible.

<a id="stop-1--something-changes-because-time-passed"></a>

## Maintenance connects time, data and a rule

Maintenance is implemented. At default rates, Fern and Flint go from 60 to 57
after three ticks; zero causes no wraparound or death. A mutable query connects
the component to that visible change, without selecting an animal by name.

[Species rates](../01-species-energy.md) established shared configuration and
individual reserves. [Sessions 06–07](../path/06-owned-costs.md) revisit that same
rule with individual costs. Its subtraction stays familiar while the source of
the rate changes. We do not need that refactor before observing movement.

<a id="stop-2--a-grazer-earns-its-meal"></a>

## Foraging makes small rules cooperate

Begin with [today's authored journey](../today-v2.md): Fern has a stable target,
and Nick writes one [movement helper](../today-v2.md#3-write-one-affordable-step).
Learn mutable borrowing by proposing a cell, checking its cost and committing
position and energy together.
The agent connects it to ECS and verifies the browser. That visible journey is
a complete session, with a [finite meal](../05-eating.md) as a separate stretch.

Then replace the authored target with [food selection](../path/03-finding-food.md)
and [a decision about when to seek](../path/04-when-to-seek.md).
`Option`, selection loops and activity state now explain an observable difference:
Fern finds food rather than following the fixture's instruction. Each layer gets
one focused regression and an installed-world/browser check before the next.

Two animals reaching one patch cannot receive the same biomass twice. Missing
food must not create energy. [Authored populations](../path/05-many-individuals.md)
then expose competition and different costs before reproduction exists. Once
those comparisons are understandable, [renewal and daylight](from-meals-to-ecosystems.md)
add a source to the finite food account. Agent scaffolding prepares the scenario
and inspector; Nick owns the rule being compared.

<a id="stop-3--rest-and-reproduction-become-real"></a>

## Rest and reproduction need separate resource rules

These are later arcs in the current recommendation, after foraging and Flint's first hunt.
Rest can reduce fatigue while food supplies nutrition. Reproduction introduces
eligibility, resource costs, cooldowns and deferred entity creation. Add these
as separate increments. One mating event must resolve once, and a newborn
must wait until its defined first eligible tick. Genetics is a later concern.

<a id="stop-4--a-hunter-changes-the-neighborhood"></a>

## Hunting makes a terminal outcome matter immediately

Flint's hunt is [proposed after the current route, with its timing revisited at
the first live foraging review](../path/README.md#what-comes-after-this-path).
Pursuit reuses bounded perception, targets, movement and terminal outcomes.
Flight and combat attributes can follow when that first interaction makes them
useful; neither is a prerequisite for instant-contact predation.
Check that two hunters receive at most one meal from one prey and that a
consumed animal cannot act afterward. Perception uses the world, not the camera.

<a id="stop-5--differences-have-a-price"></a>

## Individual variation makes controlled comparisons useful

Different speed or metabolism should change an observable outcome. The
[individual-cost comparison in sessions 06–08](../path/06-owned-costs.md) already
uses typed attributes, named units and a controlled scenario. Equal travel with
different upkeep makes the changed input's consequence visible. Policy
evaluation itself spends no travel energy; accepted movement does.

Later speed or injury rules can reuse that experimental method. Research about
levels, wealth or personality stays optional until one affects a concrete rule.
The question is the same in each case: which value changes, who owns it, and
which observation would show that the change matters?

<a id="stop-6--populations-tell-a-story"></a>

## Population history connects accounts to evidence

Counts and samples connect individual events to population changes.
[Session 05](../path/05-many-individuals.md) introduces summaries;
[sessions 14–15](../path/14-removal-and-memory.md) connect removal, retained
evidence and living counts. Later births and interventions extend that account.
Missing history stays visible. [Observability](../../design/observability.md)
owns the distinction between present state, retained events and aggregate samples.

<a id="stop-7--descendants-vary"></a>

## Inheritance initializes a new individual

Inheritance and bounded mutation introduce reproducible randomness and lineage.
Explain a child's attributes from recorded parents and the mutation rule.
Variation is not always an upgrade, and current reserves or injuries are not
automatically inherited. Compare distributions across runs, not just maxima.

<a id="stop-8--a-dungeon-becomes-an-ecosystem"></a>

## World structure tests rules beyond the camera

Several patches or chambers introduce layout generation, connectivity and
measured scaling. A larger world changes density and resource distribution;
it is more than a camera change. Keep layout generation separate from creature
rules and verify that offscreen inhabitants follow the same simulation.

## A tiny work card

The agent turns the next selected checkpoint into [NOW.md](../../../NOW.md): one
purpose, the actual file/symbol, one edit, its observable check and a stopping point.
Later ideas belong in the [parking lot](../../design/parking-lot.md).
Guide changes follow the [authoring contract](../authoring/README.md); maintaining
this route is agent work, not an extra assignment for Nick.
