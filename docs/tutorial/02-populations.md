# 2. See several individuals of each species

[Guide home](README.md) · Lesson: [session 05, read several individuals](path/05-many-individuals.md) · [Short-session path](path/README.md)

**Future integration companion for session 05.** The
[short-session guide](path/05-many-individuals.md) owns your edit: implement
`summarize_energy` from supplied reserves. It includes the complete worked answer
and test. This page explains the authored scenario, reset and browser checks the
agent prepares around that small change. Those APIs and tests do not exist yet;
[verification](authoring/verification.md) keeps that boundary explicit.

One hare and one fox make a good diagnostic: every reserve is easy to inspect.
They cannot show what “the hares are doing” means when one hare reaches food
first and another waits. Once one hare can reach and eat food, several authored
individuals let us compare their opportunities and account for all of them.

The scenario supplies starting conditions, not reproduction: these animals
exist because we authored them. Movement, meals and autonomous choice come first
in the path, so different starting opportunities can produce something worth
comparing. The agent prepares scenario plumbing before session 05; you do not
have to build a scene selector before writing the summary function.

## On this page

- [What we are adding](#what-we-are-adding)
- [The test we will walk through](#the-test-we-will-walk-through)
- [Browser checkpoint](#browser-checkpoint)
- [Optional: a mean can hide a hungry hare](#optional-a-mean-can-hide-a-hungry-hare)

## What we are adding

The current chamber is a useful diagnostic: one hare, one fox, one patch. It
stays available. A separate population scenario gives us several individuals
without needing reproduction first. Choosing a scenario starts a fresh run.
There is no random placement in this step.

The agent will put authored placement beside the existing fixture, keep settings
in the simulation crate, wire the browser selector, and add a species summary.
Exact scenario API names will be documented when that scaffolding exists; this
chapter does not ask you to call nonexistent APIs.

Counts and energy summaries group by `Species`, not nickname or ECS archetype.
Each animal still has its own `Energy`. The first comparison uses species-default
maintenance rates without individual overrides. When later sessions introduce
animal cost components, this scenario will initialize them with those same
defaults. Grass gets a patch count and total biomass: four patches are not four
blades of grass.

## The test we will walk through

The agent's planned `population_reset_restores_authored_state` regression belongs
in the bootstrap integration tests. It protects the scene around your summary
function: create a scenario, disturb it by ticking, then reset and compare the
observable starting state. This is integration preparation, not a second
learner assignment after `summarize_energy`.

| Checkpoint | Expected |
| --- | --- |
| At reset, default rates | 6 hares, 2 foxes, 4 patches; 12 unique IDs. |
| Animal energy at reset | Every reserve 60, capacity 100; mean/min/max all 60 per species. |
| Patch biomass at reset | Four patches at 80 each; total 320. |
| Disturb the world with ticks | Existing movement, meals and choice may change positions, reserves and biomass; record those outcomes before Reset. |
| Reset again | Same species, IDs, positions and initial reserves/biomass; clock 0; run number advances. |

Compare sorted simulation records, rather than the order an ECS query happens
to return. `SimId` is the stable application ID within a run. The internal Bevy
`Entity` handles can change when reset despawns and recreates entities; that is
expected. Camera and selection are presentation state, not biology.

The agent separately checks maintenance in a no-food population scenario, using
the installed schedule without disabling its systems. With no food, animals
cannot pay travel or gain meal energy: all eight should end at 57 after three
default-rate ticks. At Hare = 1 / Fox = 2, the six hares should end at 57 and the
two foxes at 54. Those literals are not acceptance values for the food-containing
browser scenario. A mixed-reserve fixture should check the summary arithmetic
too—for example hares at 20 and 60 give count 2, min 20, max 60 and mean 40.
An empty species group displays no energy mean, rather than inventing a zero mean.

The planned focused command for this **agent-owned reset check**, after
preparation, is:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap population_reset_restores_authored_state -- --exact
```

**Expected then:** one passing test. It does not exist yet, so running that filter
today and seeing zero tests is not evidence that populations work.

## Browser checkpoint

During review, select the prepared population scenario, inspect two different
hares, and Step three times. Individual reserves may differ because actual travel
and food opportunities differ. The species summary must agree with the current individuals, not a
maintenance-only prediction. Reset restores the authored data and layout. Switch
back to the diagnostic scene and confirm it still has three simulation entities.

Two inspected hares make the summary concrete, but do not establish a six-hare
mean. The agent also compares the summary with every member of that species,
including the count and sum. A correct mean formula cannot compensate for an
adapter that supplied fox reserves to the hare summary.

[Session 05](path/05-many-individuals.md#check-the-view-against-its-members) owns
the learner checkpoint and review stop. This preparation makes several animals'
outcomes inspectable; it does not claim the population is balanced or capable
of surviving indefinitely.

## Optional: a mean can hide a hungry hare

Two hares at 0 and 100 have the same mean reserve as two at 50: both averages
are 50. Their minimum and maximum tell different stories. That is why the first
summary includes all three values, with a count to say how many animals they
describe. This is an observation tool, not yet a population-behavior model.

The previously reviewed [food-choice rule](path/04-when-to-seek.md) can expose
different local opportunities across this population. Next,
[session 06](path/06-owned-costs.md) gives an individual a configured cost before
sessions 07–08 connect that value to upkeep and a controlled comparison.
[The ecosystem context](context/from-meals-to-ecosystems.md) follows how individual
opportunities affect population experiments.

[Guide home](README.md) · Lesson: [session 05](path/05-many-individuals.md) · [Short-session path](path/README.md)
