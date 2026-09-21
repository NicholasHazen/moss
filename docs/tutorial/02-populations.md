# 2. See several individuals of each species

[Guide home](README.md) · Previous: [species energy](01-species-energy.md) · Next: [food choice](03-food-choice.md)

**Future; agent scaffolding after Chapter 1.** The population API and its test
do not exist yet. The [verification record](authoring/verification.md) keeps
that boundary explicit.

One hare and one fox make a good diagnostic: every reserve is easy to inspect.
They cannot show what “the hares are doing” means when one hare reaches food
first and another waits. Before we introduce those interactions, we can create
several individuals and make sure the inspector accounts for all of them.

This chapter adds starting conditions and ways to observe them. It does not
need reproduction to explain how the animals appeared: we authored the scene.
After Chapter 1 is reviewed, send this request and I will prepare it:

> Prepare Chapter 2: keep the diagnostic scene, add the authored 6-hare / 2-fox / 4-patch population scenario and read-only summaries. Verify deterministic reset and maintenance across all animals. Give me one short test walkthrough; leave food choice unimplemented.

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
Each animal still has its own `Energy`. All six hares read the same hare settings.
Grass gets a patch count and total biomass: four patches are not four blades of grass.

## The test we will walk through

The agent will add `population_reset_restores_authored_state` to the bootstrap
integration tests. We will read it in three parts: create a scenario, disturb it
by ticking, reset and compare the observable starting state.

| Checkpoint | Expected |
| --- | --- |
| At reset, default rates | 6 hares, 2 foxes, 4 patches; 12 unique IDs. |
| Animal energy at reset | Every reserve 60, capacity 100; mean/min/max all 60 per species. |
| Patch biomass at reset | Four patches at 80 each; total 320. |
| Three ticks at default rates | All eight animals at 57; species mean/min/max all 57; biomass still 320. |
| Reset again | Same species, IDs, positions and initial reserves/biomass; clock 0; run number advances. |

Compare sorted simulation records, rather than the order an ECS query happens
to return. `SimId` is the stable application ID within a run. The internal Bevy
`Entity` handles can change when reset despawns and recreates entities; that is
expected. Camera and selection are presentation state, not biology.

The agent will also run a 1/2-rate example: all six hares at 57, both foxes at 54
after three ticks. A mixed-reserve fixture should check the summary arithmetic
too—for example hares at 20 and 60 give count 2, min 20, max 60 and mean 40.
An empty species group displays no energy mean, rather than inventing a zero mean.

After preparation, the focused command will be:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap population_reset_restores_authored_state -- --exact
```

**Expected then:** one passing test. It does not exist yet, so running that filter
today and seeing zero tests is not evidence that populations work.

## Browser checkpoint

Select the population scenario, inspect two different hares, and Step three
times. Each should agree with the hare summary. Reset should restore their
starting data and the authored layout. Switch back to the diagnostic scene and
confirm it still has only three simulation entities.

After the agent demonstrates those checks, send:

> Chapter 2 is green. Walk me through the reset test's arrange, act and assert sections, then prepare just the food-choice exercise.

**Stop here.** Multiple individuals are useful even before they interact. We
have not claimed the population is balanced or capable of surviving indefinitely.

## Optional: a mean can hide a hungry hare

Two hares at 0 and 100 have the same mean reserve as two at 50: both averages
are 50. Their minimum and maximum tell different stories. That is why the first
summary includes all three values, with a count to say how many animals they
describe. This is an observation tool, not yet a population-behavior model.

Once several animals share a world, [food choice](03-food-choice.md) can give
them different local opportunities. [The ecosystem context](context/from-meals-to-ecosystems.md)
explains how those individual opportunities later affect population experiments.

[Guide home](README.md) · Previous: [species energy](01-species-energy.md) · Next after review: [food choice](03-food-choice.md)
