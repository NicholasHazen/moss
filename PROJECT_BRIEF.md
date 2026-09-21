# Project Moss
## A little life. A world to grow.

**Vision brief · v0.1 · September 20, 2026**  
**Owner:** Nick  
**Status:** Verified browser foundation and first paired maintenance rule with its focused regression. See `NOW.md` for the next edit.  
**Codename:** Moss. An internal name, not a cleared commercial title.

## 1. Why this project exists

Moss is a browser-based ecosystem sandbox built with Rust and an entity-component-system architecture. Small creatures move through a world, find food, rest, reproduce, compete, and die. Interacting rules should produce behavior worth watching and investigating.

It is equally a place to regain coding fluency. Nick is an experienced software engineer returning to hands-on Rust after completing Rustlings, roughly seventy percent of the Rust book, and earlier reading about Bevy ECS. The aim is not to repeat a curriculum before making anything. It is to learn by building a visible world and understanding how it works.

Success has two parts: **the world becomes interesting, and its creator becomes more capable and invested.** A technically impressive simulation that Nick does not understand is not the desired outcome. Neither is a course of disconnected exercises without a living result.

## 2. A fork, not an obligation to another game

Moss grows out of Aftermarket's interest in autonomous inhabitants, ecology, inspectable consequences, and persistent history. It deliberately changes the starting point and the medium.

This is its own project. It does not inherit a requirement to build merchants, sword trading, a character utility framework, SQLite storage, or a desktop client before its creatures become worthwhile. Code and lessons may later inform Aftermarket, but compatibility or eventual merger is not a condition of success.

The imagined progression from ant farm to livestock, pets, people, and a world is a metaphor for increasing behavioral richness. It is not a required sequence of literal species or a claim that one mind architecture will automatically scale through every stage. A compelling dungeon ecology could remain Moss's focus indefinitely.

## 3. The experience

Open a browser and find a small living place. Pan across it, zoom toward something interesting, select a creature, and understand what it is doing. Pull back to see patterns across populations. Pause, step through a surprising interaction, change a rule or starting condition, and try again.

The view should feel familiar to someone using an online map: drag to pan, zoom toward the pointer, select without accidentally dragging, and return easily to the whole world. This is an interaction reference—not an integration with Google Maps or a need for geographic data, tiles, or API keys.

Simple squares, pixels, or modest sprites are enough. Clear motion, distinct activities, readable selection, and useful inspection matter more than art production. The browser is the primary way to view and interact with the simulation; a native headless test path supports development rather than replacing it.

## 4. Start with small, complete lives

Begin with a couple of simple creatures and renewable food. Eating, resting, and breeding form an early direction, introduced in understandable increments rather than delivered as one opaque package. Grazers, prey, and predators should arrive soon enough to make the world rewarding to watch.

A basic rule such as “a hunter in range consumes one available prey” is acceptable. Detailed combat is not a prerequisite. The rule still needs an unambiguous outcome: one prey cannot feed two hunters, and a creature that has died cannot act afterward.

The smallest living loop is already the important loop:

**Condition changes → perceive an opportunity → choose or continue an activity → act → experience a consequence → choose differently.**

Early policies can be direct conditionals. We will discover the need for richer scoring, intent, or memory through behavior that exposes a limitation. There is no requirement to design a universal needs API before a grazer can eat.

## 5. Layer depth and breadth through experience

Each new feature should make the world more interesting, teach something useful, or test a specific design assumption. Prefer additions that do more than enlarge a list of statistics.

Speed should eventually affect the movement rule and its costs. Health should matter when an encounter can wound rather than instantly consume. Energy consumption can depend on time, activity, distance, and attributes under explicit game rules. Rest and nutrition should not be conflated into a single source of free survival.

Variation should become visible in outcomes: one creature may reach food sooner but spend more energy getting there. Another may be slower but cheaper to sustain. Tradeoffs are more interesting than every new attribute being an upgrade.

Later inheritance, bounded mutation, and changing population traits are exciting directions. They should grow from an inspectable lifecycle, not from a hidden script that makes each generation “better.” Individual growth, learned behavior, and inherited variation are different mechanisms and should be named separately.

The aspiration is a world or dungeon ecosystem: patches, chambers, routes, habitats, and interacting populations. Procedural layouts, multiple kinds of creatures, and richer relationships can follow demonstrated needs. They do not all belong in the first chamber.

The selected initial species labels are Hare, Fox, and Grass, separate from
individual nicknames and grazer/hunter/producer roles. Multiple members of a
species should be available through authored population scenarios before
reproduction is implemented. The current design also includes local plant
renewal followed by a simulation-tick day/night cycle supplying light; weather
may later affect light, moisture, and energy costs through explicit rules.
[The ecology plan](docs/ECOLOGY_PLAN.md) distinguishes these future steps from
the working foundation and explains sizes, shapes, and optional plant spread.

## 6. A bounded world that can grow

The first world is finite and small enough to understand. Its dimensions and starting conditions should be configuration, not scattered constants. Creating a larger world should eventually be a deliberate experiment, with known effects on density, resources, and performance.

Changing world size may initially require a new run. Live resizing, infinite terrain, chunk streaming, and distributed simulation are not implied by configurability.

The camera observes the world; it does not determine which creatures get to live. Offscreen inhabitants use the same rules as onscreen ones. Visual detail may change with zoom, but physical truth must not.

## 7. A world that explains itself

Observability is a core part of the experience and the learning process. We want to connect a specific life to the larger patterns it participates in.

At the **individual level**, inspect identity, current condition, activity, target, relevant local information, and recent meaningful events. Later this includes parentage, attribute origins, and changes over a lifetime. Selection should still lead to a historical record after death, within the stated retention policy.

At the **population level**, compare creature kinds or species: counts, births, deaths and recorded causes, age or energy distributions, and activity patterns. “Race” can remain a fantasy-facing label where appropriate; technical grouping should distinguish kind, species, lineage, and temporary cohort rather than treating them as interchangeable.

At the **world level**, examine resource supply, population changes, interventions, and run configuration. A macro view should lead back to the relevant people-free creature stories, not just display an unexplained chart.

Maintain one shared semantic history with participant links. Do not create contradictory copies of an interaction in each creature's biography. Separate present state, remembered observations, event history, and aggregate measurements. Record what happened and the explicit rule or evidence used; do not turn nearby timestamps into claims of proven causation.

Browser memory is a design constraint. Start with bounded retained history, small samples, and explicit coverage. Long-run export, local persistence, and deeper historical analysis can grow later. Missing or evicted history must be visible, not silently presented as a complete lifetime.

## 8. The technical direction

Rust and ECS are chosen. Bevy ECS, a small Bevy 2D browser client, and WebAssembly are the recommended starting implementation. Specific compatible versions and web features must be verified by the bootstrap agent and recorded after actual builds; this brief does not pin an untested stack.

Prefer ordinary structs, enums, functions, queries, and ordered simulation steps. Keep biological rules separate from camera, browser, and presentation code. Keep one authoritative simulation, even when the interface projects it into many views.

Use stable application identities for history, controlled randomness when randomness is introduced, and a clear relationship between elapsed simulation ticks and state changes. Basic repeatability and focused tests should grow with the features they protect. Do not require an enterprise persistence or replay platform before the first creature works.

No custom ECS, general rule engine, universal planner, backend service, or gameplay LLM is needed at the start. Development agents are tools for building Moss, not a requirement for simulating its inhabitants.

## 9. How we will work

Nick should have first-hand knowledge of the important code and decisions without having to perform every mechanical task.

Agents can handle dependency resolution, build wiring, browser setup, repetitive glue, and narrowly scoped investigation. They should act as thought partners, reviewers, teachers, and practical sources of momentum on the simulation rules.

Communication should make returning and starting easier: short explanations, a concrete example, one recommended next action, and an obvious stopping point. Explain the Rust/ECS concept needed for the current change rather than front-loading an entire theory. Record parked ideas so they do not disappear or derail today's step.

Do not make participation a rigid purity test. Nick may implement, pair, ask for a worked example, or delegate a slice. The default is to preserve meaningful hands-on work; explicit requests for more direct help override that default.

The agent maintains the working notes and path. Nick should not have to become the project manager in order to learn and build.

## 10. What makes an iteration worthwhile

A good iteration changes something visible, gives us a way to inspect it, and leaves the code more understood. A bug can be valuable when it reveals a mistaken assumption. A rewrite can be worthwhile when it replaces a design that encountered real friction.

Keep reliable pieces when they earn their place. Discard experiments without treating them as failures. Avoid premature abstractions, but do not use “prototype” to excuse invisible errors or claims of tests that never ran.

Stable populations are not the only success criterion. Scarcity, collapse, and uneven success may be interesting outcomes if their causes are legible. Distinguish a broken implementation, a harsh scenario, and a model that is working but not yet enjoyable.

Moss should regularly reward the thought: **“I changed that rule, I understand why this happened, and I want to see what happens next.”**

## 11. What remains open

Exact species, need categories, equations, spatial representation beyond the first prototype, inheritance rules, balance values, and eventual game objectives are open design work. Browser deployment, durable storage, and performance targets should be selected from actual use rather than assumed scale.

The supporting documents propose a small technical starting point and a learning path. They are not a fixed delivery schedule. This brief preserves the direction while leaving room for the project to teach us what it needs.
