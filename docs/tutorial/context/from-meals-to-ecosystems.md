# From a meal to an ecosystem

[Context shelf](README.md) · [Guide home](../README.md) · [Finite meals](../05-eating.md)

Meadow begins with 80 biomass. Today it keeps those 80 units forever because
nothing consumes or replenishes it. Once eating exists, the patch becomes a
finite supply shared by nearby grazers. That change makes the next questions
more interesting: where does replacement food come from, and which animals
can reach it before the supply runs out?

This page explains the planned direction. Growth, daylight, weather, death and
reproduction are not implemented. The detailed model decisions remain in
[ecology plan](../../design/ecology.md); reading this page does not activate them.

## On this page

- [Count what moves between the patch and animal](#count-what-moves-between-the-patch-and-animal)
- [Why a patch is enough before a cellular automaton](#why-a-patch-is-enough-before-a-cellular-automaton)
- [Let the simulation own the day](#let-the-simulation-own-the-day)
- [Populations turn examples into experiments](#populations-turn-examples-into-experiments)

## Count what moves between the patch and animal

The first eating example proposes a 1:1 conversion: consuming two biomass units
awards two energy units. Those are still different quantities. Biomass records
available plant material; reserve records an animal's stored energy. Naming both
keeps the model understandable when we later change the conversion.

A meal can add energy while maintenance and travel subtract it in the same tick.
Suppose a hare begins at 60, pays 1 maintenance and 2 for a one-cell step, then
receives a meal worth 4. Its final reserve is 61. The meal was worth 4 even though
the net reserve rose by only 1. Recording actual action outcomes will let the
inspector explain that difference.

This is why finite eating comes before regrowth. If food immediately reappeared,
a duplicated meal could be hard to spot by watching patch totals. First we prove
that two mouths cannot claim the same last unit. Then we add an explicit source
of replacement biomass.

## Why a patch is enough before a cellular automaton

The proposed first renewal rule increases a patch's biomass under constant full
light, bounded by a capacity. With capacity 100 and growth 1 per tick, a patch at
99 becomes 100, and a full patch stays full. A zero-biomass patch can remain as a
cropped stand; we have not modeled seed availability or plant death.

That is local replenishment. A cellular automaton would introduce a different
relationship: what happens at one cell depends on neighboring cells. We may
later use that relationship for grass spreading into adjacent space, but we
do not need it to refill an existing stand. When spread arrives, reading one
old grid and committing the next grid together prevents traversal order from
letting a newly created patch spread again immediately.

The current marker is also not the plant's physical outline. Animals and patches
have cell positions; same-cell contact is the first eating rule. The equally
sized 0.8-cell squares are drawing and selection choices. A future body radius
should arrive with a rule that actually uses physical extent, rather than being
inferred from the picture.

## Let the simulation own the day

Once constant-light renewal is understandable, daylight can change its input.
The proposed cycle has 240 phase values: 0–119 are day, 120–239 are night.
Light belongs to simulation state, so Pause freezes the day
and changing a display tint cannot grow grass.

Under the planned boundary convention, the environment for executing tick
`n = completed + 1` uses phase `n % 240`. Completed tick 120 displays night;
completed tick 240 displays day. Those exact boundaries need their own tests.
Tick 0 initializes state without a growth update, so the first light interval
executes ticks 1–119; later full daylight intervals contain 120 updates.
They are part of our demonstration model, not a mapping to real physiology.

An authored cloudy interval can then reduce the light supplied to the same
growth rule. Weather affecting travel or maintenance should name those effects
separately. “Rain costs energy” is not enough to implement: we would need to say
which animals, how much, when the cost applies, and how inspection explains it.

## Populations turn examples into experiments

Six hares give us multiple outcomes to compare without implementing reproduction.
Some may start nearer a patch, arrive earlier, or lose a tie for the last bite.
A species mean tells us the overall reserve, but it can hide who is struggling.
Two hares at 0 and 100 have the same mean as two at 50; their next affordable
actions can be very different. Counts, minimum and maximum therefore accompany
the first energy mean.

Try a small prediction on paper: if six hares each pay 1 maintenance per tick,
how much animal reserve is spent across ten ticks while every hare can still
pay? The answer is 60 total units. That arithmetic becomes a useful comparison
with future food production, but it does not establish a sustainable population:
food must also be reachable, transfer is bounded, and travel has its own cost.

We are building toward experiments whose outcomes we can explain. Finite meals
account for consumption; renewal accounts for input; daylight and weather vary
that input under named rules. Starvation and reproduction then change who
exists. Each deserves its own edit, test and review rather than being hidden
inside a larger “make it more alive” patch.
