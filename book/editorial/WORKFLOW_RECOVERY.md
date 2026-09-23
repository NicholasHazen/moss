# Manual Pages publication — successful recovery

**Status:** published and verified September 23, 2026, 17:47 UTC. Nick explicitly
approved activation after automatic review required confirmation. The new
[run 35897888968](https://github.com/NicholasHazen/moss/actions/runs/35897888968)
succeeded; all 104 public files and the custom 404 match the verified export.

The [workflow](../../.github/workflows/publish-book.yml) was pushed in commit
`3d2b87673526064f3eb745e6c7cd99ce58442264`. It published export
`776dcec5fab903575ad73aacf424b417b24d9111`, without building Rust or installing later
simulation rules. The old queued run's historical record remains unresolved.

## Applied configuration

| Setting or action | Before | Applied |
| --- | --- | --- |
| Pages build type | `legacy`: automatic branch publishing | `workflow`: explicit Actions publication |
| Environment's allowed branches | `gh-pages` | `gh-pages` and `main` |
| Public URL | `https://nicholashazen.github.io/moss/` | Same URL |
| Deployment input | Verified export `776dcec5fab903575ad73aacf424b417b24d9111` | Same exact export |
| Future release trigger | Push to `gh-pages` | Manually dispatch with the verified export SHA |

The workflow runs only when manually dispatched on `main`. The upload job has
read-only repository access, checks out the selected export into `public/`, and
uploads only that directory. The dependent deployment job has `pages: write`
and `id-token: write`, the permissions required by GitHub's deployment action.
Pinned official action revisions, a separate concurrency group and timeouts
keep this attempt bounded. It does not disable the environment's branch gate.

Allowing `main` is a persistent change to deployment authorization. Checking out
an export from `gh-pages` does not change the workflow's identity: its token still
identifies the run as originating from `main`.

## Evidence before activation

- `actionlint .github/workflows/publish-book.yml` passed.
- A fresh clone of remote `gh-pages` resolved to `776dcec` and all 105 file hashes
  matched the previously verified export manifest.
- An independent read-only review found no blocker for that verified input.
- Current Pages and environment settings were saved alongside the manifest in
  ignored `book/artifacts/manual-pages-recovery.json` for rollback.

The input check requires a full commit SHA and static-export landmarks. It does
not prove that an arbitrary future SHA belongs to `gh-pages`; verifying the
export remains a release step.

## Activation and verification

After approval, pushed the workflow, switched Pages to `workflow`, added only the
`main` branch policy, and dispatched once with these inputs:

```sh
gh workflow run publish-book.yml --ref main \
  -f export_commit=776dcec5fab903575ad73aacf424b417b24d9111
```

Both upload and deployment succeeded; the run finished at 17:46:40 UTC. At
17:47:33 UTC, system `curl` retrieved all 104 public files, including both
downloads, and every SHA-256 hash matched the reviewed manifest. A missing URL
returned HTTP 404 with the exact custom page. Pages reports `built` in `workflow`
mode. The ignored recovery record contains the settings backup and HTTP results.

The old run still reports queued. If it wakes now, its artifact is the same export.
Before a newer release, check that old run's status and consider possible late
publication of its earlier content; the new concurrency group does not control it.
The current deployment used the frozen export unchanged. Maintainer documentation
was updated afterward without rebuilding the published packages.

For a later failure, inspect its actual error before another action. If rolling
back this configuration change, restore the saved Pages build type/source and
remove only the `main` branch policy introduced by this attempt. Preserve the
site, branch history, and existing `gh-pages` policy. No site deletion or history
rewrite is needed. No rollback was needed for this successful deployment.

## Why this is different from another branch build

GitHub supports [custom Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
The deployment action sends a new artifact to the Pages deployment API; it does
not invoke the branch-build endpoint whose requests wait behind an existing
build. A fresh workflow may avoid the orphaned run's orchestration, but GitHub
does not guarantee that it bypasses every Pages backend failure.

Workflow lint, documentation links, exact reference-copy checks and whitespace
checks passed. No new native/WASM builds, model tests or browser interaction
checks were run for this deployment; the content had already passed its local
release checks. The fresh verification establishes public artifact delivery.

The [original incident](PUBLICATION_INCIDENT.md) remains recorded. The unsent
[Support request](SUPPORT_REQUEST.md) is superseded; no support contact was needed.
