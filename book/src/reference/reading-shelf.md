<p class="eyebrow">Reference shelf · Open at the point of need</p>

# Good company for the next question

You do not need to finish a reading list before changing Moss. These sources
are useful when a question has become specific enough to carry into another
explanation. Each invitation says what to look for and what to bring back.
The book's examples and diagrams are original; the links below do not import
another author's code, artwork or simulation into Moss.

## “What is the thing behind this reference?”

The official Rust book's
[References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
chapter is a good companion to the movement proposal and bounded meal. Follow
one caller-owned value through a function. Then return to `*position = next`
and identify which place that assignment changes. Its neighboring
[ownership chapter](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
is useful when moving a value and copying a `Copy` value start to blur together.

These are living documentation pages. Moss's executable examples are checked
separately with Rust 1.93.1 and edition 2024; a newer page's surrounding examples
do not silently change the project's compiler.

## “Which candidate wins, or what if there is none?”

The standard-library reference for
[`Iterator::min_by_key`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.min_by_key)
states the tie and empty-input behavior of that method. Read it beside the
food-selection chapter and compare a distance-only key with a distance-and-ID
key. The key encodes a policy; a concise iterator expression still needs one.

[`Option::unwrap_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or)
becomes useful in the individual-defaults chapter. It supplies the fallback for
`None`, while `Some(0)` preserves zero. Return with the distinction between an
absent instruction and a present instruction whose value happens to be zero.

## “Why did this entity disappear from the loop?”

Bevy ECS's [0.18.1 Query documentation](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html)
is the primary reference for the access declarations used here. Inspect
required components, optional access and disjoint queries. Then read the
maintenance query as a membership rule: which exact component sets can match?
An animal missing a required cost component has not proved its upkeep is zero;
it may have missed the system entirely.

## “When does the next system see a change?”

The versioned
[`chain()` documentation](https://docs.rs/bevy/0.18.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html#method.chain)
and [`Commands` documentation](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Commands.html)
explain ordering and deferred changes. In this Bevy version, a chain can add
deferred-application points between systems. “Queued” therefore does not mean
“invisible until the entire tick ends” in every schedule.

Bring that distinction back to target creation, removal and newborn eligibility.
Bevy's visibility mechanics tell us when data can be observed. Moss still needs
a policy for which animal may act and which claimant receives a resource.

## “Why should the world use a different clock from the pictures?”

Glenn Fiedler's [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/)
develops the accumulator idea through physics examples. It is useful background
after one Moss tick already makes sense. Follow the separate jobs of simulation
time and rendering time, then notice what happens when catching up requires
more work than the machine can finish.

Moss makes its own policy choices, including bounded tick requests and pausing
hidden tabs without catch-up. The
[Page Visibility API documentation](https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API)
explains the browser signals and background constraints. It does not choose
Moss's policy for us, and a physics essay does not prove browser determinism.

## “Am I looking at a reserve, a rate or a transfer?”

MIT OpenCourseWare's
[stock-and-flow mapping assignment](https://ocw.mit.edu/courses/15-871-introduction-to-system-dynamics-fall-2013/resources/mit15_871f13_ass3/)
offers useful questions about stored quantities, rates, units and the boundaries
of an account. There is no worksheet requirement here. Return to Meadow with
three labels: biomass stored, biomass actually produced this tick, and biomass
actually consumed. The labels reveal errors that a graph of one final number
can conceal.

## “What would make this a fair experiment?”

The [NetLogo BehaviorSpace manual](https://docs.netlogo.org/behaviorspace.html)
shows how a modeling tool describes repeated experiments, inputs and
measurements. Read it after the two-hare comparison. Bring back an experiment
card with a starting state, one deliberate change, a tick horizon and the
outcomes to record. If randomness is introduced later, seeds and repetitions
join that card; they are not prerequisites for the current authored comparison.

For an optional excursion, Uri Wilensky's
[Wolf Sheep Predation model](https://ccl.northwestern.edu/netlogo/models/WolfSheepPredation)
contrasts implicit food with explicit grass consumption and renewal. Look at
the assumptions before comparing the population curves. A collapse can expose
a model's supply limit without demonstrating a programming error. The linked
model has its own rules; its behavior is not evidence about Moss.

## “Does changing a default change an animal that already exists?”

Flecs documents explicit copy-versus-inherit policies under
[OnInstantiate component traits](https://www.flecs.dev/flecs/ComponentTraits.html).
That comparison helps make the word *default* precise. An initialized owned
value and a live lookup into a shared base respond differently when the base
changes. Bring the observable difference back to the two-hare example before
choosing a representation. Flecs's mechanism is an engine comparison, not a
claim that Bevy supplies the same inheritance API.

When a temporary effect becomes relevant, Epic's
[Gameplay Attributes and Attribute Sets](https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-attributes-and-attribute-sets-for-the-gameplay-ability-system-in-unreal-engine?application_version=5.6)
offers another useful distinction: baseline and current effective values.
Return to three separate numbers—an individual's baseline rate, the effective
rate under a condition, and its reserve remaining. Moss does not need to adopt
an ability-system framework to preserve those different meanings.

## “Surely this layout must be faster?”

The Factorio developers' [Friday Facts #204](https://www.factorio.com/blog/post/fff-204)
reports a substantial data-layout change that produced no measurable improvement,
alongside more specific optimization experiments. Its value here is the reason
to measure. It is not a benchmark of Bevy, WebAssembly or Moss's populations.
Bring back a workload and a correctness comparison before drawing a performance
conclusion from a pleasing arrangement of structs.

These sources were selected and their primary pages inspected on September 23,
2026. Living manuals can change. Versioned Bevy links describe 0.18.1; comparisons
with other engines are explanatory choices rather than dependency proposals.
