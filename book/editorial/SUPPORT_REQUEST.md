# GitHub Support draft — stuck Pages rerun

**Superseded; not submitted.** This draft preserves observations from September
23, 2026, 17:12 UTC. Publication subsequently succeeded through the
[manual workflow](WORKFLOW_RECOVERY.md), verified at 17:47 UTC. The settings and
publication state described below are historical; update them before any future
submission about the old run, which still reports queued.

**Subject:** Pages rerun stuck queued; cancellation and rerun APIs disagree

Repository: <https://github.com/NicholasHazen/moss>

Affected run: <https://github.com/NicholasHazen/moss/actions/runs/35879572240>

Please reconcile or terminally cancel this stuck rerun so we can retry the
existing GitHub Pages release. Please preserve the live site and its configuration.

Pages uses the managed `pages-build-deployment` workflow, publishing `gh-pages`
at `/` (`build_type: legacy`). Actions is enabled and the workflow is active.
The deployment of commit `776dcec5fab903575ad73aacf424b417b24d9111` failed
on September 23 at 15:10 UTC when `actions/deploy-pages@v5` timed out requesting
an OIDC identity token. Build and artifact upload had succeeded.

A failed-job rerun was accepted at about 15:11:58 UTC. At 17:12 UTC:

- Run summary still says `queued`, attempt 1, updated at 15:11:58 UTC.
- Attempt 1 says `completed` / `failure`, updated at 15:10:46 UTC.
- Latest jobs endpoint returns `total_count: 0`; `filter=all` shows only the
  original completed jobs, including failed deploy job `107244323958`.
- Pages build `1234464369` still says `building` for `776dcec`, without an error.
- Earlier normal cancellation returned “Cannot cancel a workflow run that is completed.”
- Earlier full rerun returned “This workflow is already running.”
- Force-cancel returned HTTP 409, “Cannot cancel a workflow re-run that has not
  yet queued.” Request ID: `DDAB:CD4CB:2996406:2B33406:6AB3F3F3`.

No successful cancellation or second attempt has occurred. We have not deleted
the run, changed publishing settings or permissions, or queued additional builds.
The previous successful release remains available at
<https://nicholashazen.github.io/moss/>; the refinement is not live.

Could you clear or reconcile the stale run and Pages build state, or identify a
supported recovery that preserves the published site and branch history?

## Submission boundary

Send only the subject and report above after Nick authorizes contacting Support.
It contains public repository/run identifiers and a GitHub request ID; no tokens,
credentials, local paths or full logs are needed. The fuller local investigation
is in [the incident record](PUBLICATION_INCIDENT.md).
