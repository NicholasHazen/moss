<p class="eyebrow">Chapter 7 · Future relationships to explore</p>

<a id="7-another-kind-of-life"></a>

# Another kind of life

Flint has been present from the beginning: a fox, a hunter, an individual with
a reserve that maintenance can spend. Those labels give him a place in the
world, but they do not feed him. A hunter role becomes meaningful when a rule
lets Flint do something that changes another creature's life.

The same is true of a resting activity, a parent link or a weather label.
Each earns its place through an observable consequence. The questions below
extend the [ecology model](../../../docs/design/ecology.md); none describes
behavior already installed or selects another active task. They show how the
small rules developed in this book can support a richer world without requiring
all of that world to be designed first.

## One prey can feed one hunter

A first hunt can be as direct as “an eligible hunter in range consumes one
available prey.” Pursuit, escape and injuries would create other interesting
interactions, but they are not prerequisites for testing that first terminal
outcome. If pursuit becomes the chosen model, target selection and movement
already offer useful mechanisms to extend.

The difficult boundary appears when two hunters want the same prey. Both might
observe it while it is alive and submit valid-looking requests. As with two
hares reaching the last biomass, an earlier observation is not permission to
spend a resource later. Resolution must inspect current eligibility in a
defined order.

This time, however, the resource is also an actor. The first successful claim
must make the prey unavailable before a later hunter can receive a reward,
and before the prey can perform a later action. The following is an
illustrative transition, not a proposed data type:

```text
Prey is available
        |
        | First eligible claim resolves
        v
Prey is unavailable ──> One actual consumption outcome
        |
        | Later requests check current eligibility
        v
No second reward; no later action by the consumed prey
```

Queuing a despawn without changing eligibility leaves a gap: the entity may
still exist when the next request runs. An end-of-tick starvation check leaves
the same gap for a different reason—it runs too late to govern an earlier
predation interaction. Immediate removal or an explicit unavailable state can
close it, provided every later resolver respects the committed result.

One shared record should identify the actual participants and outcome. If
consumption and death use separate linked records, their meanings must be
distinct so the population account counts one death. Flint's reward also needs
its own stated rule; the prey's last observed reserve is not automatically the
amount the hunter receives.

That gives a first hunt a small, strong test: two requests, one prey, at most
one reward, and no subsequent action by the consumed animal. A richer combat
system can change how a claim succeeds while preserving that boundary.

## A rested animal can still be hungry

Rest introduces a different resource story. Energy in these examples represents
nutrition that maintenance and movement spend and meals replenish. Fatigue
would describe a condition that rest can reduce. Letting rest manufacture
food energy would erase much of the pressure that made Meadow worth reaching.

Consider an illustrative rest action that reduces fatigue from six to two.
During the same tick, ordinary upkeep could reduce reserve from ten to nine.
The animal becomes less tired while becoming slightly hungrier. Both changes
can be correct because they answer different questions. These numbers merely
expose the distinction; they are not selected rest settings.

The interesting choice then comes from competing conditions. Should a tired
grazer continue toward nearby food or stop to recover? Begin with an explicit
conditional whose inputs can be inspected. A threshold or short commitment may
be enough to reveal the tradeoff. A universal needs score is unnecessary until
concrete behavior shows why a simpler rule cannot express the desired choice.

Rest also deserves an outcome boundary. A completed recovery action can reduce
fatigue once. Re-evaluating whether to rest should not repeatedly award recovery
or charge an action that has not happened. That is the same separation between
decision and accepted effect that kept a proposed movement from spending energy.

## A birth needs a budget and a first tick

Creating another animal is mechanically possible before reproduction is a
meaningful rule. A birth becomes part of this ecosystem when eligibility,
parental costs and the child's starting resources are explicit enough to
account for.

For an illustrative funded transfer, suppose the participating parents together
provide twelve energy units and the child begins with twelve. Those values make
the transfer visible; they do not settle how many parents the rule requires,
how costs are divided, or whether an additional reproductive cost should be
spent. Whatever policy is chosen, the resolver must recheck eligibility and
affordability before committing the costs and birth together. Two requests
must not charge the same mating event twice or create two children from one
accepted event.

The child also needs a stable identity and a defined first opportunity to act.
The architectural direction is that a child created during tick *n* first acts
on tick *n + 1*. Otherwise, its first maintenance charge, meal or movement could
depend on where creation happened relative to queries in the current tick.
A newborn might even reproduce in its creation tick if eligibility were left
implicit.

The birth record links the actual parents and child, and population accounting
gains one birth. Authored placement remains seeded population. A useful
checkpoint would inspect the parents' actual costs, the child's starting state,
one birth outcome, and the absence of child actions until its defined first
tick. Genetics can wait until that funded lifecycle is understandable.

## Descendants inherit a baseline, not a finished life

Inheritance asks where a new individual's initial attributes come from. That
question fits the initialization boundary developed for species defaults and
authored overrides. A reproductive rule can derive a child's baseline from
recorded parent values instead of using only a species template.

A parent's current reserve is a different kind of value. It reflects meals and
costs already experienced. So do fatigue, an injury and a currently selected
target. Copying all of the parent's components would silently answer several
unrelated questions: whether the child begins tired, inherits damage, remembers
food, or receives stored energy without anyone paying for it.

Name the intended sources individually. A child might receive a derived upkeep
baseline, a separately funded reserve, and no current target. Those are three
initialization decisions, not one instruction to clone a parent. A learned
behavior or a change acquired during life also needs an explicit inheritance
policy before it can be treated as heritable.

If mutation arrives, bound it and make its randomness reproducible enough for
the comparisons being claimed. Record the relevant parentage and rule context.
A lower upkeep cost is not proof that every generation improved: another
attribute may impose a tradeoff, and different habitats can favor different
combinations. Compare populations and conditions, not only the largest number
found in one descendant.

## Let the environment change a known relationship

Weather has a clear first entry point in the supply account. An authored
clear → cloudy → clear interval can change effective light supplied to growth.
The plant rule then reports the biomass actually retained. This gives clouds
a consequence before adding random weather, soil moisture or temperature.

Fractional production makes arithmetic policy visible. If cloudy conditions
permit half a biomass unit per tick, rounding every request down to zero would
prevent growth forever. A defined remainder can carry progress across ticks.
That would be a new numerical rule to check, not something a darker background
can implement. Rain or cold should likewise name which input they change;
an unused weather label has no biological effect.

### When a larger picture changes contact

Geometry asks a related question: what does a larger picture mean? The live
markers are schematic squares, and the first meal model uses simulation cells
for contact. Enlarging a sprite or its click target does not enlarge an
animal's reach. A future body radius would need simulation-owned units and
explicit contact and boundary rules. A plant stand might instead occupy a
footprint of cells. Shape matters when a rule uses it, while visual detail can
remain a presentation choice.

### Refill a patch or spread to a neighbor

Meadow's refill and grass spreading into a new cell are also separate
relationships. Refill changes a stored quantity at an existing patch. A
cellular-automaton spread rule would make a cell's next state depend on its
neighbors. Reading the old grid before committing the next one prevents a
newly established patch from spreading again merely because traversal reaches
it later in the same tick. Neighborhoods, contested cells and establishment
costs would then become the specific decisions to inspect.

Each extension changes one relationship already visible in the world: a hunter
claims an actor, rest changes fatigue, a birth funds another participant,
inheritance supplies a baseline, or an environment changes production. They
need not arrive together. A small ecosystem can reveal a worthwhile question
long before it has every possible kind of life.

Keep those possible lives in view, then return to the one ready to begin.
[Make Fern's first affordable step](01-one-affordable-step.md#your-first-edit),
get its single named test passing, and ask for review. That small result gives
the next biological change a world you can already explain.
