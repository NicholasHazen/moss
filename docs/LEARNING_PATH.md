# Learning path — grow a world by changing one rule

**This is a path to choose from, not a backlog to finish or a schedule to meet.**

Only the current card in `NOW.md` is active. Later stops can change order. The agent recommends the next small step and maintains the notes; Nick is not responsible for turning this page into a project-management system.

For the next actual coding sessions, use the [step-by-step tutorial](tutorial/README.md).
Start with its Chapter 1A resource/default edit and stop for review. This page
remains the broader path; the tutorial supplies concrete code and test checkpoints.

`crates/moss-sim/src/lessons.rs` now has the implemented maintenance loop and
three documented, unscheduled stubs for later biological examples. Nick's focused
maintenance regression passes; only this first rule is scheduled. The recommended
next edit makes passive burn configurable by species, then mechanical population
setup precedes pairing on food choice.
[ECOLOGY_PLAN.md](ECOLOGY_PLAN.md) records the selected model defaults and which
parts are still proposals.

## The basic rhythm

See a behavior. Locate its data and rule. Make one change. Predict one result. Run it. Inspect what actually happened. Keep a small regression when the change reveals a useful invariant.

A compile error is part of the work, not a reason for the agent to replace the whole implementation. Help should become more direct when it is useful, not remain trapped in hint mode.

## Stop 0 — a place to build

**Visible result:** A browser scene with world bounds, food and two identifiable creature placeholders, a selected-entity panel, and tick controls. The placeholders are not autonomous yet.

**Agent work:** Toolchain/dependency checks, browser setup, camera pan/zoom, drawing, controls, and basic test wiring.

**Nick's involvement:** A small code tour: where a component lives, where the tick runs, and how an inspector reads state. No prerequisite reading assignment.

**Finish here:** The browser path works or its exact environmental blocker is documented. The agent does not continue by implementing the creature loop.

## Stop 1 — something changes because time passed

**Visible result:** Fern and Flint each lose one energy unit per tick. Watch Fern in the inspector; Pause freezes the reserve and Step changes it once.

**Small implementation:** One mutation through a query for `Energy` on entities with `Creature`, with a clear lower bound. Select by relevant components, not Fern's name or ID. Use an authored initial reserve; zero does not imply a death rule until that rule is introduced.

**Learn:** `struct`, a component derive, a mutable query with a component filter, a bounded arithmetic operation, and a focused test. Explain only what is needed to make this edit.

**Check:** Both reserves go from 60 to 57 after three ticks and never wrap around at zero. Positions and Meadow's biomass remain unchanged.

**Next natural question:** What should the creature do when the reserve is low?

**Mechanical slice before Stop 2:** Keep the diagnostic fixture and add a separate
authored population fixture (6 hares, 2 foxes, 4 grass patches), scenario reset,
and read-only counts/energy summaries by species. This makes multiples visible
without implementing reproduction. Plant summaries count patches and biomass,
not individual blades. The agent can scaffold this after Stop 1; Nick still owns
the following behavior exercises.

## Stop 2 — a grazer earns its meal

**Visible result:** Hunger changes activity. The creature approaches nearby food, consumes available biomass, then stops seeking food after becoming satisfied.

**Small implementation sequence:** First choose an activity and target from nearby food, without moving. Then move within bounds toward that target with an explicit axis/tie rule. Then resolve consumption when actually in range. Split these into separate sessions or diffs rather than writing the entire loop in one answer.

**Learn:** Enums and pattern matching, read versus write queries, target identity, and the difference between a proposal and an actual effect.

**Check:** No food exists → no energy is invented. Food capacity is finite. Already-satisfied creatures do not repeatedly receive an eating reward. Start/stop thresholds can prevent visible twitching.

**Next separate slice — Meadow replenishes:** After the first finite meal works,
add bounded patch renewal with named biomass-per-tick units, a capacity, and an
explicit place in tick order. This is an environmental resource input. Eating
must still consume only what is actually available. Check empty and full patches
before using replenishment to support longer runs or population growth.

Start renewal under constant full light, then introduce tick-driven day/night
as a separate slice. An authored cloudy interval can modify light afterward.
Neighbor-based plant spread (a possible CA) and weather effects on moisture or
animal costs are later independent experiments, not hidden additions to renewal.

Mortality is also a separate rule: decide what happens at zero and whether
maintenance precedes a possible meal. Do not smuggle starvation into the first
decrement, or rest into energy replenishment.

## Stop 3 — rest and reproduction become real

**Visible result:** A tired creature rests; a well-fed, eligible pair can produce a new creature. These are separate small additions, not one exercise.

**Learn:** Explicit activity duration, cooldowns, eligibility checks, deferred commands, and resource costs.

**Check:** Rest reduces fatigue without replacing food. One mating event is resolved once, uses the stated cost, records parentage, and does not let the child act in its birth tick. Repeated ticks during cooldown do not create unlimited offspring.

The exact reproductive model is authored game design. Start with a simple rule; no genetics or elaborate courtship is required.

## Stop 4 — a hunter changes the neighborhood

**Visible result:** Hunters pursue nearby prey; prey flee a visible nearby hunter. Reaching eligible prey may instantly consume it.

**Learn:** Bounded perception, selection criteria, invalid targets, interruption, and stable conflict resolution.

**Check:** Two hunters reaching the same prey receive at most one meal in total. A consumed creature cannot later eat or breed in that tick. Fleeing decisions use creature perception, not camera visibility.

This stop can move earlier when pursuit is the most motivating next behavior. It does not require completed reproduction or a health model.

## Stop 5 — differences have a price

**Visible result:** Creatures with different speed or metabolism behave differently under the same starting conditions. Later health permits injury rather than instant consumption.

**Learn:** Replacing a constant with a component, units, derived rates, parameterized scenarios, and small comparisons.

**Check:** Decide what a speed change costs and test that rule. Moving farther may consume more energy; changing how often a policy thinks must not. Health is implemented with an actual consequence, not added as a decorative bar.

## Stop 6 — populations tell a story

**Visible result:** A population sample or simple graph shows a change; the timeline and creature records explain the relevant interactions.

**Learn:** Aggregation, event indexing, sampling, retention, and distinguishing a count from a rate.

**Check:** Population totals reconcile with births, deaths, and interventions. Small retained histories announce their limits. Do not wait until this stop to log earlier meaningful events; this stop expands analysis, not its foundations.

## Stop 7 — descendants vary

**Visible result:** A child inherits selected bounded attributes with controlled variation. Over runs, inspect distributions and lineage rather than only a maximum statistic.

**Learn:** Reproducible random streams, inheritance versus individual state, bounded mutation, and experiment comparisons.

**Check:** The recorded parent data and mutation rule can explain the child's attributes. Mutation is not always an upgrade. Food reserve and current injury are not automatically inherited as genes. Avoid adding a global optimizer that chooses which creatures are allowed to succeed.

This is later experimentation, not part of the initial bootstrap.

## Stop 8 — a dungeon becomes an ecosystem

**Visible result:** Multiple patches or chambers, different habitats, and several populations create interactions beyond one small room.

**Learn:** World configuration, layout generation, connectivity, spatial indexing, and measured scaling—one actual need at a time.

**Check:** A larger map is a different scenario, not merely a bigger camera. Track density and resource distribution. Offscreen simulation continues by the same rules. Layout generation and ongoing creature behavior remain separate responsibilities.

## A tiny work card

The agent should turn the current step into something like this, with real file paths after bootstrap:

```text
Goal: Make time cost the creature energy.
Why: It creates the pressure that will later motivate eating.
Open: the current Energy component and maintenance system.
Change: decrease the reserve once per complete tick, bounded at zero.
Proof: Step three times; the inspector and a focused test agree.
Stop: the rule works and you can point to the mutation.
Help: ask for a hint, a tiny example, pairing, or direct implementation.
```

Do not add a second task to the card simply because it is related. Maintain a runnable checkpoint. Returning tomorrow should not require remembering today's whole conversation.
