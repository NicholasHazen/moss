# Short-session path: review and executable evidence

[Path home](README.md) · [Tutorial authoring](../authoring/README.md)

**Evidence through September 23, 2026.** These records cover the module
reorganization and movement preparation now committed in `83a72a5`, plus later
tutorial refinements. Their passing references do not install behavior or establish
browser acceptance. Today's live helper remains `todo!()` and movement stays
unscheduled. [Current guide evidence](../authoring/verification.md) covers v2;
[development verification](../../development/verification.md) covers the runtime.

## Commands and observed results

The printed references were extracted into isolated workspaces with their own
`CARGO_TARGET_DIR`, using pinned Rust 1.93.1, edition 2024, and Bevy ECS 0.18.1
from the existing lockfile. The latest evidence is a sequence of checks:

| Checkpoint | Observed result |
| --- | --- |
| Last complete reference target | All 16 tests were discovered and passed, none ignored, with the execution guard. This included the strengthened session 01 assertions; sessions 06 and 07 were extended afterward and checked separately below. |
| Session 06 refinement | Its extended reference passed one discovered test, Clippy and pinned formatting. It distinguishes absent from explicit same-as-default input when constructing values under a later template. No installed Reset or provenance wiring was tested. |
| Session 07 membership example | Both discovered reference tests passed, with Clippy and pinned formatting. Removing the animal filter passed the old test but failed the new non-animal control. No live query or spawn wiring changed. |
| Runner execution guard | Seventeen Python fixtures passed. Native ignored and conditionally excluded reference probes were rejected; an expected-panic test was accepted. These checks do not prove assertion quality. |
| Session 01 refinement | Its revised reference test and Clippy passed. Two faulty variants that passed the old assertions fail the stronger checks for finite supply and a configured positive bite limit. |
| Earlier temporal checkpoint | All 16 tests, three additional Rust explanation tests and Clippy passed after revising sessions 11 and 13. |
| Session 13 clarification | A subsequent explicit fixture reset passed its focused rerun: one test, 15 filtered out. |
| Session 03 revision | The diagonal distance case passed its focused rerun: one test, 15 filtered out; the whole reference target passed Clippy. |
| Session 02 revision | The depleted shared-patch case passed its focused rerun: one test, 15 filtered out; the whole reference target passed Clippy. |
| Deliberately faulty variants | Seven initial variants and later distance/shared-food variants compiled, then failed their intended assertions. The correct references were restored and temporary mutant targets removed. This is targeted evidence, not general mutation coverage. |

To reproduce the **printed references**, from the repository root:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --keep-workspace
```

This copies the workspace and runs the extracted `month_guide` target offline;
a fresh machine first needs the normal toolchain and dependencies. It does not
test whether Nick's live edit is finished. Each selected session must have a
compiled test, and every discovered test must report passing. An ignored test
does not count. [The latest guard check](../../history/tutorial/2026-09-22-path.md#requiring-reference-tests-to-execute-september-22)
records the fresh full and single-session runs; path Clippy and browser checks
were not repeated. [Exact commands and earlier runs](../../history/tutorial/2026-09-22-path.md#commands-and-observed-results)
and [the later temporal run](../../history/tutorial/2026-09-22-path.md#rules-that-meet-at-a-tick-boundary)
retain the earlier execution history. [The runner check](../../history/tutorial/2026-09-22.md#reproducible-movement-answer-september-22-2026)
records the fresh full pass; it did not rerun path Clippy or scratch diagnostic
targets. [The later meal check](../../history/tutorial/2026-09-22-path.md#three-meal-limits-checkpoint-september-22)
records the focused session 01 revision; the other fifteen reference bodies
were unchanged at that checkpoint. [The later ownership check](../../history/tutorial/2026-09-22-path.md#equal-values-different-authored-inputs-september-22)
records session 06's new case without claiming a repeated full suite.
The [query-membership check](../../history/tutorial/2026-09-22-path.md#query-membership-checkpoint-september-22)
records session 07's added diagnostic and temporary filter-removal experiment.
Temporary scratch targets named in history are not checked-in
commands.

## What each reference establishes

| Session | Tested scope, and where the claim stops |
| --- | --- |
| 01 | Room, finite supply and a configured positive bite each limit an actual transfer; full capacity, empty supply and zero bite grant nothing. No eligibility or ECS adapter. |
| 02 | Stable-ID ordering of already eligible eaters, shared remainder and actual meal versus net change. No ECS eligibility query. |
| 03 | Local, nonempty nearest-food selection with stable ties, meaningful absence, an inclusive radius boundary and a diagonal metric discriminator. No installed target mutation. |
| 04 | Idle/Seeking transitions and threshold boundaries using post-maintenance reserves. No deferred target visibility check. |
| 05 | Count, minimum, maximum and mean, including a fractional mean; empty input stays absent. Species filtering and browser aggregates remain future checks. |
| 06 | Copy ownership, default/override resolution, explicit zero and equal initial values from distinct authored inputs. New construction under a changed template preserves the explicit override. No live spawn, Reset or origin-recording wiring. |
| 07 | Valid-fixture component attachment, individual upkeep and clamping; a separate synthetic fixture shows required-data and animal-filter exclusion with a positive control. No future complete-world reset acceptance. |
| 08 | Accounting for supplied affordable movement traces, equal travel and unequal upkeep. The reference does not move creatures. |
| 09 | Growth reports the actual amount, handles partial capacity, honors a request that fits and allows cropped patches to regrow. No installed growth system. |
| 10 | Growth-before-meal arithmetic and full-patch ordering. No autonomous-choice or live-schedule proof. |
| 11 | Day/night boundaries, executing versus completed tick, and initial versus full daylight opportunity counts. No environment resource or browser time check. |
| 12 | Light changes production, capacity still limits it, and stored food remains edible at night. No live environment adapter. |
| 13 | Proposed after-meal starvation with stored night food, an empty night, dawn growth, and repeated underpaid upkeep through daylight. Contact is assumed; no installed choice or accepted death policy. |
| 14 | Actual Bevy collection/despawn, ordered removed IDs, repeat-call behavior and an unchanged surviving control. No journal or full later-action schedule check. |
| 15 | Count-account consistency and rejection of impossible deaths or counter overflow. Supplied samples do not establish real population dynamics or supply. |
| 16 | Canonical component snapshots match across insertion orders and detect changed reserves and targets. No tick execution, browser repeatability or full future-state comparison. |

## What remains for future activation

Before offering a later session, the agent inspects current code, prepares the
named plumbing and focused live test, then identifies one learner edit. After
review, verify installed ordering, Reset and the named browser observation.
The [activation contract](../authoring/README.md#prepare-a-future-session-for-actual-use)
keeps that work outside the learner's focus window. A passing reference does not
settle design questions: after-meal starvation is still an unaccepted proposal.

No later behavior was installed by writing these guides. Live movement/browser
acceptance, future integration and native RustRover preview of these sixteen
guides remain pending.
Reading times and learning outcomes have not been measured.

## Navigation, visuals and preservation

The dated records describe visual inspections in Chromium and XML checks of
local SVGs. These are illustrations of references, not screenshots of activated
biology. V2's two diagrams and navigation now have a separate
[native preview check](../../history/tutorial/2026-09-22.md#native-reading-checkpoint-september-22-2026);
the sixteen future guides have not received that IDE inspection. The current
[document checker](../authoring/README.md#run-the-document-checks) verifies local
navigation and fences; exact historical counts stay in their dated checkpoints.
The [September 23 visual check](../../history/tutorial/2026-09-23.md#choice-before-the-meal)
covers session 04's two-tick illustration in Chromium. Its numbers are an
expected trace of the proposed order; no live foraging system or native IDE
rendering was tested in that pass.

The [comparison observation review](../../history/tutorial/2026-09-23.md#inspecting-overlapping-individuals)
checked session 08's inputs and the existing stable-ID selection controls in
source. Its revised instructions use separate entity buttons while overlapping
hares stay paused at the same tick. No reference code or live behavior changed;
the future scenario's browser acceptance remains pending.

The path's source/configuration preservation checks passed. The later v2 work
changed only NOW's two lesson links among those protected files; its
[separate record](../../history/tutorial/2026-09-22.md#todays-lesson-v2-september-22-2026)
explains that exception. No dependency or live rule changed in these guide passes.

## Detailed review records

Earlier section destinations remain below. Each link opens the full dated
record, including its commands, review findings and limits.

<a id="history-coverage-checkpoint-september-22"></a>

- [History coverage checkpoint, September 22](../../history/tutorial/2026-09-22-path.md#history-coverage-checkpoint-september-22)

<a id="shared-outcomes-checkpoint-september-22"></a>

- [Shared outcomes checkpoint, September 22](../../history/tutorial/2026-09-22-path.md#shared-outcomes-checkpoint-september-22)

<a id="spatial-assumptions-checkpoint-september-22"></a>

- [Spatial assumptions checkpoint, September 22](../../history/tutorial/2026-09-22-path.md#spatial-assumptions-checkpoint-september-22)

<a id="ongoing-teaching-development-checkpoint-september-22"></a>

- [Ongoing teaching development checkpoint, September 22](../../history/tutorial/2026-09-22-path.md#ongoing-teaching-development-checkpoint-september-22)

<a id="rules-that-meet-at-a-tick-boundary"></a>

- [Rules that meet at a tick boundary](../../history/tutorial/2026-09-22-path.md#rules-that-meet-at-a-tick-boundary)

<a id="investigating-an-apparently-wrong-inspector-result"></a>

- [Investigating an apparently wrong inspector result](../../history/tutorial/2026-09-22-path.md#investigating-an-apparently-wrong-inspector-result)

<a id="review-and-revision"></a>

- [Review and revision](../../history/tutorial/2026-09-22-path.md#review-and-revision)

<a id="further-teaching-refinement-september-22"></a>

- [Further teaching refinement, September 22](../../history/tutorial/2026-09-22-path.md#further-teaching-refinement-september-22)
