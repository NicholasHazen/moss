<a id="3-choose-nearby-food-without-moving-yet"></a>

# 3. Let Fern choose nearby food

[Guide home](README.md) · Lessons: [03, choose food](path/03-finding-food.md) and [04, remember seeking](path/04-when-to-seek.md)

**Future integration companion for sessions 03–04.** Those two
[short sessions](path/03-finding-food.md) own the learner edits and complete
worked answers: first `nearest_food`, then `next_foraging_state`. This page
explains how the agent connects them to existing movement and meals. The helper
and activity data are not installed yet. `FoodTarget` already exists for today's
authored journey and will be reused. [Verification](authoring/verification.md)
separates recorded example checks from future activation.

Fern's first journey follows the target we authored. Movement and meals make
that instruction useful; choice changes where it comes from. The future policy
uses her condition and nearby food to decide where to go and when to stop seeking.

The selection helper itself changes no position or energy. That boundary lets
its tests isolate the choice. In the complete simulation, the chosen target then
flows into the movement and eating rules already running later in the tick.
The path tests ordinary Rust functions first. Agent integration then checks
their ECS adapter and the complete behavior in the browser. These are different
levels of evidence for the same behavior, not another curriculum to complete.

## On this page

- [The small rule](#the-small-rule)
- [A: a test for selection](#checkpoint-a--a-test-for-selection)
- [B: choosing becomes an ECS behavior](#checkpoint-b--choosing-becomes-an-ecs-behavior)
- [Optional: why a tie needs a rule](#optional-why-a-tie-needs-a-rule)

## The small rule

Among nonempty grass patches within a bounded Manhattan distance, choose the
nearest. Manhattan distance counts cardinal grid steps: horizontal distance
plus vertical distance. Equal distance chooses the lowest `SimId`, independent
of spawn or query order. No eligible food returns `None`.

For the first choice-only experiment, use a diagnostic radius of 10 cells:
Fern at (10, 10) is 9 steps from Meadow at (16, 13). A smaller radius would make
that existing patch invisible. This is an authored demonstration setting,
not a claim about real hare vision.

Hunger and eligibility belong in the system around this helper. These are
future policy rules, not prerequisites for today's movement session. Proposed
initial settings are: begin seeking below 45 reserve, stop at 75, and preserve
the previous seeking state between those thresholds. These are energy units
for the current capacity-100 animals. Maintenance runs first, so the decision
uses the reserve after that tick's passive cost. Foxes do not become grass eaters.

Using separate start and stop thresholds gives the animal a reason to continue
an activity. With one threshold, a small meal could make it stop, the next
maintenance cost could make it start, and the activity label could alternate
every tick. The gap gives the previous state a defined role. This technique is
often called *hysteresis*: the same current reserve can produce a different
decision depending on whether the animal was already seeking.

## Checkpoint A — a test for selection

[Session 03](path/03-finding-food.md) contains the candidate trace, complete
selection helper, test and Rust explanation. That is the canonical answer;
there is no second implementation to reconcile here.
Older execution records describe this chapter's former printed examples; the
[path verification](path/verification.md) records checks of the revised answer.

Before that session, the agent prepares the public helper
`moss_sim::lessons::nearest_food` in `lessons.rs` with a `todo!()` body. Its inputs
are `origin: Position`, a borrowed `patches: &[(SimId, Position, u32)]`, and
`radius_cells: u32`; its return type is `Option<SimId>`. The tuple holds stable
identity, position and available biomass. The adapter supplies grass patches
only; the helper filters emptiness and distance and resolves ties.

The agent also prepares `nearest_food_is_local_nonempty_and_stable` in
`crates/moss-sim/tests/foraging.rs`. That live test imports the public helper;
it must not declare a local duplicate. It checks reordered observations,
empty food, an empty candidate list, and the radius boundary. Include the
diagonal counterexample from session 03 so a straight-line metric cannot pass
as Manhattan distance. The planned
focused command, **after preparation**, is:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test foraging nearest_food_is_local_nonempty_and_stable -- --exact
```

Exactly one test should reach the unfinished helper before the learner edit,
then pass after it. Missing imports, a missing target or zero matching tests
mean preparation is incomplete. The independent printed-answer check in the
short-session guide tests the reference, not this live function. Their
[different roles](path/README.md#checking-your-work) matter when deciding whether
there is anything ready to activate.

## Checkpoint B — choosing becomes an ECS behavior

[Session 04](path/04-when-to-seek.md) owns the next paired edit: the body of
`next_foraging_state`. The agent prepares its `ForagingState` enum and focused
test. That helper decides when to start, continue or stop seeking; it changes
neither position nor energy. Its complete answer and state diagram live with
the lesson.

After helper review, the agent prepares the system around it: check grazer
eligibility, update activity, gather eligible patches while seeking, call
`nearest_food`, and write the selected stable `FoodTarget`.
Missing or empty targets must be cleared or replaced. Target IDs resolve within
the current run, and stopping seeking clears the food target so the existing
movement adapter does not continue toward a stale instruction. The diagnostic
fixture moves from an authored initial target to policy-owned target updates.

The completed schedule is maintenance → choice → movement → eating → complete
tick. Choice reads the reserve after maintenance; movement and meals may change
it again later in the same tick. That ordering makes both the decision and the
final reserve explainable.

A focused adapter harness applies maintenance, then choice, with movement and
eating outside that isolated harness. It must prove the following **selection-only**
expectations; they are not end-of-tick values for the full simulation:

| Before a tick, maintenance = 1 | After the tick |
| --- | --- |
| Idle hare at 45, nearby food | Reserve 44; seeking that patch; position unchanged. |
| Idle hare at 60 | Reserve 59; remains idle. |
| Seeking hare at 60 | Reserve 59; remains seeking with valid nearby food. |
| Seeking hare at 76 | Reserve 75; stops seeking. |
| Hungry hare, no eligible food | No target; no food or energy invented. |
| Hungry fox beside grass | No grass target. |

The adapter tests also reverse patch spawn order and confirm the same stable
target. Calling choice directly twice must not spend energy or change position.
Record actual target changes without filling the bounded journal with an
identical event every tick.

A separate full-schedule regression proves the policy reaches the existing
executors: a hungry grazer chooses, takes an affordable step, and eats only when
it reaches the target cell. Assert the maintenance and actual travel charges
alongside any actual meal. A stop decision clears the target before movement;
a missing or empty patch cannot supply an invented destination or meal. The
agent prepares literal values against the then-current fixture before this
checkpoint becomes active.

**Browser:** let the diagnostic hare drop below the seeking threshold. The
inspector should show its activity and target, then its cell should change as
movement executes. Once contact occurs, Meadow loses biomass and Fern gains the
actual meal energy. After she reaches the stop threshold, the next choice pass
stops seeking if her post-maintenance reserve still meets it. User selection
only changes inspection; it does not choose the animal's food.

The agent finishes activation when the helper, isolated adapter and full-schedule
checks agree with the browser sequence. [Session 04's review stop](path/04-when-to-seek.md#turn-the-decision-into-behavior)
owns that handoff. The route then continues to [session 05's population summary](path/05-many-individuals.md),
with scenario and reset details in the [population companion](02-populations.md).

## Optional: why a tie needs a rule

If two patches are equally near, either could make sense for the animal. Why
choose the lower ID? It gives this version a repeatable answer that survives a
change in spawn order. We can later choose randomness deliberately and control
its seed; relying on storage traversal order would make the choice accidental.

The [session 03 worked test](path/03-finding-food.md#a-borrowed-slice-and-a-running-best-candidate)
already gives you a check: reversing the input leaves the
answer at patch 4. If we moved patch 9 onto the origin while leaving it nonempty,
patch 9 should win because distance is compared before ID. You can reason that
out from `(distance, id)` without adding another feature.

[Guide home](README.md) · Lessons: [03, choose food](path/03-finding-food.md) and [04, remember seeking](path/04-when-to-seek.md)
