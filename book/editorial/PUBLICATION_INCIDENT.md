# Pages deployment investigation — September 23, 2026

**Publication recovered at 17:47 UTC.** With Nick's approval, a fresh manual
workflow deployed the refined export `776dcec`. All 104 served files match the
reviewed export, including both downloads; the custom 404 also matches. The
old rerun still reports queued, but no longer prevents publication. See the
[recovery record](WORKFLOW_RECOVERY.md) for the successful run and configuration.

## What failed

Repository: `NicholasHazen/moss`. At the time of failure, Pages was configured to publish the root of
`gh-pages` using GitHub's managed branch-publishing workflow (`build_type: legacy`).

- [First release, run 35841617425](https://github.com/NicholasHazen/moss/actions/runs/35841617425):
  successful, commit `14b93fab06513aa6440150816e94044ae7936dd9`.
- [Refinement, run 35879572240](https://github.com/NicholasHazen/moss/actions/runs/35879572240):
  commit `776dcec5fab903575ad73aacf424b417b24d9111`.
  Attempt 1's build and artifact upload succeeded. Deploy job `107244323958`
  failed while `actions/deploy-pages@v5` requested an OIDC identity token.

The deployment log's specific error is an HTTP request timeout at the Actions
identity-token endpoint. It occurs before a successful Pages deployment request.
The subsequent `id-token: write` hint is generic:
[deploy-pages v5 emits it for every `getIDToken()` failure](https://github.com/actions/deploy-pages/blob/v5/src/index.js#L18-L25).
It does not establish that repository permissions are incorrect. The
[toolkit's token client](https://github.com/actions/toolkit/blob/main/packages/core/src/oidc-utils.ts)
distinguishes missing environment values from a failed HTTP request.

This evidence supports a runner/service request failure; it does not identify
the exact network fault or prove a platform-wide outage. Rust compilation and
simulation tests were not jobs in this publishing run.

## Why the accepted retry did not recover it

A failed-job retry was accepted at about 15:11:58 UTC. At 15:43 UTC, fresh reads
showed the following inconsistent state:

| Read or recovery operation | Observed result |
| --- | --- |
| Current workflow-run summary | `queued`, attempt 1, last update 15:11:58 UTC. |
| Attempt-specific endpoint for attempt 1 | `completed`, `failure`, updated 15:10:46 UTC. |
| Jobs with `filter=all` | The original successful build/report jobs and failed deploy job. |
| Latest-job listing | No new jobs listed. |
| Pages/latest build | `building` for commit `776dcec`, with no error message. |
| Earlier normal cancellation | Refused because the workflow was completed. |
| Earlier full rerun | Refused because the workflow was already running. |

After normal cancellation had failed, one request used GitHub's documented
[force-cancel endpoint](https://docs.github.com/en/rest/actions/workflow-runs#force-cancel-a-workflow-run).
It returned **HTTP 409**:

```text
Cannot cancel a workflow re-run that has not yet queued.
```

GitHub request ID: `DDAB:CD4CB:2996406:2B33406:6AB3F3F3`.
No cancellation succeeded. No additional retry, trigger commit, settings change,
permission expansion, credential rotation or deletion followed this conflict.

## What is preserved and verified

At 15:45 UTC, system `curl` retrieved all **104 publicly served files** from the
first release and compared their SHA-256 hashes with its recorded manifest.
Every file matched, including both downloads. Eighty-five unchanged files also
match the refined export; the changed files still belong to the earlier release.
This verifies the public release is intact, not that the refinement has deployed.

The new export and its downloadable source were already verified locally; no
runtime or source tests were rerun during this incident investigation. The
expected refinement hashes remain in the ignored
`book/artifacts/review-v2-release-record.json`. The fresh public check is in
`book/artifacts/actions-investigation-public-check.json`.

The active learner exercise remains `move_one_cell`. Local browser development
and the local book preview do not depend on the stalled GitHub run.

## Recovering the original run

GitHub must queue or clear the accepted rerun. Once it has a consistent terminal
state, inspect the result before taking another action. If it succeeded, verify
the public files against the refinement manifest. If it failed, inspect the new
attempt's error before deciding whether one normal full rerun is appropriate.

If the contradictory state persists, the run URL, attempt/job IDs, timestamps,
409 response and request ID above form a concise GitHub Support report. This
document is prepared locally; no support message has been sent. Keep the working
public site and publishing configuration in place while GitHub resolves the run.

An independent read-only helper checked the official action source and recovery
documentation and agreed that the permission hint is not a diagnosis. The lead
retrieved the run/attempt/job state, inspected the deployment log, attempted the
single force-cancel recovery and verified the public artifact hashes.

## Follow-up at 17:12 UTC

Fresh API reads found the same run timestamps and contradictory states. The
latest jobs endpoint returned zero jobs; Actions is enabled and the managed
workflow is active. Pushing checkpoint `6c610af` to `main` did not trigger a
Pages release: the configured publishing source remains `gh-pages` at `776dcec`.

The live GitHub status feed marked Actions and Pages operational. Its remaining
September 23 incident update concerns delayed Projects indexing, so it does not
establish an explanation for this deployment failure.

A second bounded helper independently checked recovery documentation. GitHub's
[Pages build API](https://docs.github.com/en/rest/pages/pages#request-a-github-pages-build)
states that a new build request waits for an existing build to complete; it is
not a documented bypass. No further cancellation or rerun was attempted against
the unchanged state. Publishing settings and history remain intact.

Prepared a short [Support request](SUPPORT_REQUEST.md) asking GitHub to reconcile
the run and Pages build state. Submitting it would send a message on Nick's behalf
and requires his authorization; it has not been sent. The earlier full public
file verification remains separate from these fresh API observations.

## Alternative publication route prepared

Nick asked whether publication could proceed without Support. Prepared and
reviewed a [manual workflow recovery](WORKFLOW_RECOVERY.md) that deploys the exact
verified export through a fresh workflow and deployment request. This changes
the publishing route rather than fixing the old rerun.

Automatic approval review rejected pushing and activating that change before
execution because it changes the persistent publishing mode and adds `main` to
the allowed deployment branches. Explicit approval was requested. No remote
settings changed during that rejected operation. Nick subsequently approved activation.

## Successful publication at 17:47 UTC

Pushed workflow commit `3d2b87673526064f3eb745e6c7cd99ce58442264`, switched Pages
from `legacy` to `workflow`, and added `main` to the deployment environment while
preserving the existing `gh-pages` rule. Dispatched the verified export
`776dcec5fab903575ad73aacf424b417b24d9111` once.

[Run 35897888968](https://github.com/NicholasHazen/moss/actions/runs/35897888968)
completed successfully at 17:46:40 UTC. At 17:47:33 UTC, all 104 public files
matched the refined release manifest and a missing URL returned the exact custom
404. Pages reports `built` with build type `workflow`. The original run remains
queued with its unchanged timestamp; this recovery does not claim to have fixed
that historical record. No Support request was sent.
