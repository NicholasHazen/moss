<p class="eyebrow">Concept index · Find the explanation, then the example</p>

# A few words with precise jobs

Use this page when a term interrupts your reading. Each entry gives the working
meaning used by this edition and a place to see it matter. The source snapshots
and versioned documentation provide deeper detail when the immediate example
is no longer enough.

| Word or phrase | Meaning in the book |
| --- | --- |
| **Action** | An attempted change whose actual result can differ from the intention. [An affordable step](../chapters/01-one-affordable-step.md#think-about-a-step-before-taking-it) may be rejected. |
| **Activity** | Remembered behavior such as Idle or Seeking, separate from a currently valid target. See [choice](../chapters/03-let-the-animal-choose.md). |
| **Archetype** | A group sharing a set of component types. Different energy values do not alone create different archetypes. See [two hares](../chapters/04-two-hares-one-fair-comparison.md). |
| **Borrow** | Temporary access through a reference, with the original value still owned elsewhere. See [the movement parameters](../chapters/01-one-affordable-step.md#what-the-references-let-us-change). |
| **Component** | Typed data attached to an entity, also used by queries to select and declare access. See [the code map](code-map.md). |
| **Copy** | A trait permitting an implicit independent value copy for a type that supports it. Fern's proposed position uses it; see [the proposal](../chapters/01-one-affordable-step.md#what-the-references-let-us-change). |
| **Default** | An input used when a more specific authored input is absent. Whether it is copied or looked up live is an additional policy; see [individual differences](../chapters/04-two-hares-one-fair-comparison.md). |
| **Entity** | An ECS identity carrying components. Species, nickname and stable history identity answer different questions about it. See [the adapter](../chapters/01-one-affordable-step.md#how-the-little-function-reaches-an-ecs-world). |
| **Invariant** | A condition the model must preserve, such as total food awarded not exceeding supply. See [the shared meal](../chapters/02-a-meal-has-two-sides.md). |
| **Outcome** | What actually happened, such as biomass consumed. A requested bite and a tick's net reserve change are different quantities. See [meal accounting](../chapters/02-a-meal-has-two-sides.md). |
| **Override** | A present, more specific authored instruction. Zero can be an override, and a value equal to the default can still have a different origin. See [individual differences](../chapters/04-two-hares-one-fair-comparison.md). |
| **Provenance** | Recorded information about where a value came from. A resolved number alone cannot recover the authored instruction. See [individual differences](../chapters/04-two-hares-one-fair-comparison.md). |
| **Query** | A declaration of component data and access, optionally narrowed by filters. Missing required components exclude an entity. See [the adapter](../chapters/01-one-affordable-step.md#how-the-little-function-reaches-an-ecs-world). |
| **Resource** | Shared ECS data used by systems, such as world configuration or shared species settings. See [the code map](code-map.md). |
| **Schedule** | The ordered execution of systems and relevant deferred-application boundaries. See [one visible tick](../chapters/01-one-affordable-step.md#from-one-correct-call-to-one-visible-tick). |
| **Stable ID** | Moss's application identity used to name participants across inspection and history; distinct from a storage handle. See [target resolution](../chapters/01-one-affordable-step.md#how-the-little-function-reaches-an-ecs-world). |
| **System** | A function that declares the ECS data it accesses and can run in a schedule. `move_to_food` is the prepared system adapter; `move_one_cell` is the ordinary helper it calls. See [the adapter](../chapters/01-one-affordable-step.md#how-the-little-function-reaches-an-ecs-world). |
| **Tick** | One executed simulation step. Its rules do not depend on how many frames draw it. See [one visible tick](../chapters/01-one-affordable-step.md#from-one-correct-call-to-one-visible-tick). |
| **World** | The ECS store holding entities, their components and shared resources. Moss uses one authoritative world; presentation derives its view from that state. See [the code map](code-map.md). |

An ECS organizes access and storage. It does not by itself choose the game's
allocation, starvation or inheritance policy. Those choices become understandable
when they appear as explicit rules with an observable consequence.
