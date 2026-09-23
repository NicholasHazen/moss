# First technical review

**Date:** September 23, 2026

**Scope:** the first textbook draft, chapters 1–7, against the current source,
canonical worked references and pinned Bevy ECS 0.18.1 API. This is a source
review, not fresh runtime verification or publication approval.

## Result

**No actionable correctness findings in the reviewed draft.** I found no
substantive mismatch in the selected Rust/ECS claims or the worked numerical
traces. No P1/P2/P3 correction is requested. Future behavior is labeled at its
point of use, and the unfinished live exercise is not presented as installed.

The review deliberately challenged the following claims rather than treating a
correct final number as sufficient evidence:

| Area and location in the reviewed source | Counterexample considered and result |
| --- | --- |
| Chapter 1, `01-one-affordable-step.md:105`, `:119`, `:220` | Changing a copied proposal must leave the caller unchanged; rejecting a step must also preserve reserve. The distinction between copying, borrowing and write-back is correct. Exact payment and a supplied rate different from the default are explained. |
| Chapter 1, `01-one-affordable-step.md:286` | A targetless creature must not match the prepared movement query. Read/write access to `Position` needs the stated disjoint filters. The prose agrees with `lessons.rs` and the pinned query API. |
| Chapter 2, `02-a-meal-has-two-sides.md:240`, `:395`, `:445` | Sorting stale awards would still overdraw a patch; copying an initial `Energy` value does not share future reserves. The chapter correctly requires the current remainder and distinguishes a four-unit meal from a three-unit net tick gain. Arrival at 37 versus the separate same-cell case at 36 is reconciled. |
| Chapter 3, `03-let-the-animal-choose.md:398`, `:430`, `:468` | A 74 start gives 73 at choice and 77 after eating, while remaining Seeking. The next choice sees 76 and stops. The nearby 72 start instead ends at 75, then maintenance produces 74 and allows another meal. Both traces agree with the transition function and target-gated action policy. |
| Chapter 4, `04-two-hares-one-fair-comparison.md:194`, `:279`, `:322` | `None`, `Some(0)` and an override equal to the default retain different authoring meanings. Already constructed costs are independent values. The required query components and the non-animal positive data control distinguish missing attachment from wrong arithmetic. |
| Chapter 4, `04-two-hares-one-fair-comparison.md:518`, `:580` | Nested fields do not create separately attached components or separate ordinary query accesses. Different field values do not create new archetypes. The discussion makes no unsupported throughput claim and preserves the single-threaded context. |
| Chapter 5, `05-a-world-that-feeds.md:216`, `:346`, `:365` | Full and empty patches distinguish growth/eating order differently. Tick zero is initialization; executing ticks 1–119 give 119 initial daylight updates, while 1–240 still contain 120 daylight updates. The proposed executing-tick convention and ranges agree. |
| Chapter 5, `05-a-world-that-feeds.md:619`, `:639` | Starting the night at reserve 20 exhausts reserve on tick 139, the twentieth night tick. Night requests 120 units but deducts only 20. Dawn can supply one to a same-cell animal because this chapter has no death rule. The two-unit biomass account across ticks 119–240 also reconciles. |
| Chapter 6, `06-a-disappearance-explained.md:154`, `:389` | Reserve one, cost two and a one-unit meal can sustain survival under the stated no-debt rule. The chapter explicitly exposes that consequence. Its separate six-hare scarcity argument uses cost one, for which each surviving animal can pay the next tick's unit, so it does not confuse requested costs with deductions. |
| Chapter 6, `06-a-disappearance-explained.md:223`, `:270`, `:350` | Collected handles and copied IDs outlive the query without retaining component borrows. Removal reports only success. An oldest-retained tick cannot imply complete coverage of that boundary tick; zero evictions cannot imply collection of unrecorded event types. These boundaries are correctly stated. |
| Chapter 6, `06-a-disappearance-explained.md:591`, `:643`, `:660` | Optional target access preserves targetless animals, and copied snapshots can be retained while the world changes. Capacity is intentionally omitted and its consequent limit is disclosed. Snapshot equality is not promoted to replay, save/resume or installed browser evidence. |
| Chapter 7, `07-another-kind-of-life.md:29`, `:102`, `:122` | A consumed actor must become unavailable before later actions; a queued despawn alone is insufficient. Newborn first action and baseline inheritance remain explicit future model choices, not implications of cloning ECS components. |

Locations identify the source lines read during this pass; later inserted labs
or prose can shift them. Chapter 2's narrative and Rust reference were read
before its parallel interactive-lab addition. That new lab is outside this
review.

## Teaching improvements, separate from correctness

These are optional editorial improvements, not defects or requirements to
change the worked rules.

1. **Give the disjoint-query explanation its failure moment.** At
   `01-one-affordable-step.md:292`, add one short sentence that Rust can compile
   the two query parameter types while Bevy rejects conflicting access when it
   initializes the system. This would connect the existing filters to the
   actual diagnostic a returning programmer might see. Keep the detailed
   explanation in an optional aside; the current API explanation is correct.
2. **Join history coverage and population accounting in one static spread.** At
   `06-a-disappearance-explained.md:505`, a clearly illustrative record for one
   terminal `SimId`, a six-to-five living-count change, and a visibly partial
   retained-history boundary would give the reader one small investigation
   spanning the chapter's otherwise separate mechanisms. Include an independent
   whole-run death counter if the example claims a total; do not derive that
   total from retained details. This follows the editorial plan's proposed
   Chapter 6 evidence spread without adding another simulation.

## Checks and sources

Read the project brief, active task, repository instructions, helper contract,
coding conventions, architecture, editorial plan and existing evidence record.
Read chapters 1–7 and relevant canonical guide passages. Compared their claims
with `components.rs`, `energy.rs`, `config.rs`, `lessons.rs`, `simulation.rs`,
`fixture.rs`, `journal.rs`, and the pinned manifests/toolchain. Inspected the
existing worktree status before writing. Numerical traces above were checked by
following the printed operations; no executable test or build was run.

The pinned primary documentation confirms the reviewed scheduling and query
claims: [`chain()` and deferred application](https://docs.rs/bevy/0.18.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html#method.chain),
[query data, optional access and disjoint filters](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html),
and [`World` query creation/removal APIs](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/world/struct.World.html).
These references establish API contracts, not that this draft's examples were
newly compiled during the audit.

Only this report was written. No live biology, existing guide, `NOW.md`,
dependency, Git state, GUI, hosting configuration or deployment was changed.
No additional agents were used. The existing reference-test evidence remains
the evidence recorded in `EVIDENCE.md`; this pass does not upgrade it. Rendered
layout, accessibility, interactive-model fidelity, links, copied-block identity
and final public-site behavior remain the publishing lead's separate checks.
