# Textbook build and review evidence

## September 23, 2026 — first publishing slice

The existing worktree was read before writing. Its HEAD was
`83a72a5abd46339e5448076fc183bfe33cd54029`; existing local modifications were
preserved. The textbook is a new edition under `book/`. This record describes
the edition's checks, not a new live Moss browser acceptance.

### Local publishing tool and build

Installed the official crates.io `mdbook` package with:

```sh
scripts/with-toolchain.sh cargo install mdbook --version 0.5.3 --locked --root .tools/mdbook-0.5.3
```

The release compilation and installation passed. Moss's Cargo manifests and
lockfile were not changed. `sh book/scripts/book.sh build` successfully generated
HTML with the custom CSS/JS, search and ordinary print view. Explicit repository
citations are packaged by `finish_site.py` as inert, hashed source snapshots.
The latest first-slice build contains the opening and chapters 1–4; chapters
5–6 were drafted but not yet in that build.

`python3 -B book/scripts/check_book.py` passed on 11 Markdown files, 102 local
source destinations, 13 exact reference copies, 25 generated HTML pages and
371 local output destinations. Counts are a snapshot, not a fixed acceptance
target. The checker excludes external HTTP availability and does not assess
rendering, prose or accessibility. `git diff --check` also passed.

### Executable references

```sh
python3 docs/tutorial/authoring/check_path_examples.py
```

In a temporary workspace and independent target directory, the runner discovered
and executed **17 named tests across all sixteen reference modules**. All passed,
zero ignored or filtered. This run verified the canonical reference blocks;
the book checker separately verifies that copied blocks are identical.
It did not install those rules in the live world or run the full live suite.

```sh
python3 docs/tutorial/authoring/check_path_examples.py --movement
```

The exact movement answer passed the focused helper test, then all **five**
movement/maintenance tests after activation only in the temporary copy. Native
Clippy for those two integration targets passed with `-D warnings`. This used
the supplied-rate regression as currently prepared. Live `lessons.rs` remained
unfinished and unscheduled. No new live WASM build or Moss browser journey ran.

### Teaching model

```sh
node --check book/theme/moss.js
node --test book/scripts/test_labs.cjs
```

Syntax passed. Three Node tests passed: the printed reserve ledger including
rate 3 and explicit zero cost, arrival/invalid coordinates, and both directions
with x-before-y. Inputs were checked to remain unchanged. These are tests of
an independent JavaScript illustration; they do not establish general
equivalence with Rust's integer semantics.

In in-app Chromium, the chapter link reached the intended edit. The model
showed the copied proposal changing while the caller stayed unchanged, then an
accepted commit `(3, 2)` / 57 from reserve 59. Exact reserve 2 committed to zero;
reserve 1 rejected and stayed `(2, 2)` / 1. Reset restored the attempt under the
selected input values. An invalid negative reserve disabled the attempt and
displayed its valid input range.

Browser testing found that listening only for a committed input change left the
display stale while editing a number. The model now resets on input, and the
updated display/rejection/exact-payment path was rechecked. mdBook's live reload
also reset an in-progress experiment when a helper wrote a chapter; the wrapper
now builds once and serves fixed files, with explicit rebuild/reload during QA.

### Reading surface

Inspected the opening at the normal 1280×720 viewport in dark and light themes.
The first illustration, type hierarchy and primary edit link rendered. At a
375×812 responsive viewport the lab stacked its controls/cards and the document
width remained 375 pixels without page overflow. This was an initial layout
check, not a whole-book mobile audit. The temporary viewport was reset.

Search for `affordable` returned 19 results and a selected result navigated to
the nearby variation. mdBook starts searching on keyboard `keyup`; filling the
field alone did not trigger it. A pre-rebuild page logged a missing old hashed
search asset; refreshing the rebuilt page and using keyboard search succeeded.
Further testing must check the final, stable publication artifact.

### Public hosting inspection

Nick selected web only and a public site linked to the Moss repository.
Read-only GitHub checks confirmed `NicholasHazen/moss` is public, its default
branch is `main`, and the already signed-in owner has admin/write access. The
Pages endpoint returned 404 under that owner, so no existing Pages site was
found. The active global CLI account was not switched. No repository setting,
remote branch or deployment has been changed in this pass.

Official workflow guidance consulted:
[GitHub's custom Pages workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
It documents configure-pages v5, upload-pages-artifact v4, deploy-pages v4 and
checkout v6 at this date. A publication method has not yet been installed.

## Open verification work

- Review the full six-chapter argument and each newly introduced claim/trace.
- Complete and verify the remaining original interactive models and figures.
- Test publication tooling's failure cases and the final source allowlist.
- Inspect all generated chapter figures, code blocks and source-snapshot links.
- Verify keyboard, narrow-screen, light/dark and no-JavaScript reading paths.
- Exercise the final build under the repository's public URL subpath.
- Publish the reviewed artifact, then verify the real public URL and repository link.

Future checks should append their observations rather than silently upgrading
these first-slice results into whole-book coverage.

## First full narrative and five models

The opening, chapters 1–7, code map, concept index and reference shelf now build
together. `check_book.py` passed on 18 Markdown files, 145 source destinations,
**17 exact Rust reference copies**, 49 generated HTML pages and 797 local output
destinations. The build packaged 43 explicitly cited source/figure files.
All source chapter tables now use at most two columns for IDE/narrow reading.

`node --test book/scripts/test_*lab*.cjs` passed **27 tests** across five models:
3 movement, 5 meal, 4 tick-phase, 7 defaults and 8 daylight tests. The new models
are independent JavaScript teaching implementations. Their Node checks establish
the selected traces and input contracts; their browser controls, keyboard
behavior and rendered layout still require inspection. No new Rust reference
changed, so the earlier exact-copy comparison and native execution remain the
relevant Rust evidence.

The [independent technical review](TECHNICAL_REVIEW.md) found no actionable
correctness findings in chapters 1–7. It checked against current source and
pinned primary APIs, with numerical traces reasoned through rather than freshly
executed. It excluded the newly inserted labs. Its disjoint-query diagnostic
suggestion was integrated; the suggested Chapter 6 history/accounting figure
remains an editorial task.

A preservation check compared all 135 pre-existing files against their initial
hashes and found no changes outside `book/`. This is separate from the pre-existing
uncommitted work, which remains present.

## Full browser, editorial and publication-tooling pass

All five labs were exercised in in-app Chromium. In the meal model, reversing
input order to 7→1 retained resolution 1→7: a five-unit patch awarded four then
one; a four-unit patch awarded four then zero. Invalid negative supply blocked
resolution; keyboard Reset restored the authored example. The tick model showed
74→73→77 while Seeking, then the next tick cleared the target at 76 and preserved
the remaining four biomass. The empty-patch case retained Seeking without a target.

The defaults model retained owned costs 1/1/0 while the draft changed to nine.
A runtime edit changed Fern to five; Reset restored 1/1/0 with draft nine still
pending; Start configured run produced 9/1/0. A fractional draft disabled Start
without changing the active values. The daylight model showed growth/meal at
119, no production at 120, stored night food consumed, dawn rescue at 240, and
requested growth one/actual growth zero at a full patch. Boundary stepping
executed intervening ticks, and keyboard Reset restored the selected preset.

The five models now have **28 passing Node tests**, including a next-tick visual
projection check. The independent model audit found no rule defects and identified
one input-outline contrast issue; input/select borders now use the stronger accent
token. The daylight strip was inspected in light desktop and dark 375-pixel views.
Its phase-zero preview appeared after completing tick 239. A long preset select
initially extended beyond its card at phone width; the min-width/width fix was
visually rechecked, with its dropdown arrow inside the card and the full selected
starting state repeated below it. The viewport override was reset.

All seven chapters reported document width 375 at a 375-pixel viewport; all cited
images loaded. This establishes fit/load, not small-label readability in every
figure. The new Chapter 6 SVG was inspected at its native size without clipping.
The current browser reading theme was also checked in light and dark. Keyboard
Enter opened a complete reference; a search-highlight URL revealed all three
folded answers in Chapter 4 and highlighted its matches.

The export audit's three findings were repaired: real HTML start-tag attributes
are rewritten without touching example text/comments/scripts; renderable figures
are restricted to a passive SVG vocabulary; and HTML checking resolves base URLs
under the configured `/moss/` prefix. The 404 heading link also needed its explicit
page path because of the base element. **20 publication regressions passed** after
repair. The original failing report remains an accurate account of discovery.
The new loopback preview serves the production prefix. An unknown URL displayed
the styled custom error page with working local assets; the page was subsequently
given explicit home/first-edit links. It still needs the final public-host check.

An independent editor added local navigation, nearby run commands and ten folded
complete answers, preserving all seventeen marked Rust blocks. The coda now
recommends returning to the first movement edit. An external-reference review
retrieved **23 distinct primary-resource URLs**, finding their named subjects and
versions. This did not test every browser fragment scroll or future availability.

## Downloaded starter, independently exercised

The deterministic starter contains **52 files / 273,312 bytes**, SHA-256:

```text
6544fa8514b594bb3df8ad818edc0e17d3c114f5ee5a3ea3ec553aaa588146a3
```

Twelve starter-packaging tests passed. The archive was extracted into
`/private/tmp/moss-book-starter-12ctkwpb/moss-starter-2026-09-23`, restoring its
recorded executable script modes. These commands ran from that extracted root:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --offline --test bootstrap --test maintenance
scripts/with-toolchain.sh cargo test -p moss-sim --locked --offline --test movement movement_charges_only_an_affordable_actual_step -- --exact
scripts/with-toolchain.sh trunk build --locked --offline --release
python3 docs/tutorial/authoring/check_path_examples.py
python3 docs/tutorial/authoring/check_path_examples.py --movement
```

The baseline's **five tests passed**. The movement command compiled and ran
exactly one selected test, which failed at the intended `todo!()` (exit 101,
two tests filtered). The WASM release build passed in 58.77 seconds. All **17**
future reference tests passed from the extracted guide fences. The movement
runner passed its focused helper, all **five** activated movement/maintenance
tests, and focused native Clippy in a temporary copy. Both reference copies
were removed by the runners; the extracted and live exercises remain unchanged.

The extracted `dist/` was served separately on loopback port 8082. In in-app
Chromium it loaded the scene and inspector; three Steps left Fern at (10,10)/57
and Flint at (21,6)/57. Reset began run 2 and restored Fern to (10,10)/60, with
four initialization/placement journal entries and zero evictions. The tab's
warning/error log query returned no entries. This is a short maintenance smoke
check, not new coverage of all camera or hidden-tab paths. Its temporary tab was
closed after inspection.

Latest whole-book navigation check: **26 Markdown files, 203 source destinations,
17 exact reference copies, 51 HTML pages and 1,013 output destinations**. The
counts are an intermediate snapshot before editable-source integration.

## Editable-source rebuild and public release

The editable-source download contains **111 files / 957,162 bytes**, SHA-256:

```text
ae296f6d88a556b1daaaaf7aca69c33b6a5cc2ca4bbc9e1fbb35c9d2c060a362
```

Extracted it into
`/private/tmp/moss-edition-check-4c714p9k/moss-book-source-2026-09-23` and linked
the already installed mdBook 0.5.3 tool into its local tool location. From that
independent root, the build, link check, **42 Python tests** and **28 Node tests**
passed. Its regenerated **105 export files matched the original byte for byte**,
including both ZIP downloads and their checksums. This verifies rebuildability
with available tools; it does not claim a fresh mdBook installation on another OS.
The local whole-book check at release covered 27 Markdown files, 208 source
destinations, 17 exact reference copies, 51 HTML pages and 1,019 output destinations.
The extraction has fewer Markdown files because private editorial notes are excluded.

Published only the verified static export from a separate temporary Git checkout.
The normal push created `gh-pages` at
`14b93fab06513aa6440150816e94044ae7936dd9`. The authorized repository owner configured
Pages to serve that branch's root and set the repository homepage to the book URL.
The first Pages creation response ended unexpectedly; a fresh read confirmed the
operation had succeeded, so it was not repeated. The global CLI account stayed
unchanged. Remote `main` was read back as
`83a72a5abd46339e5448076fc183bfe33cd54029`.

[Pages deployment run 35841617425](https://github.com/NicholasHazen/moss/actions/runs/35841617425)
completed successfully for the publication commit. A read-only HTTP audit fetched
all **104 public files** (the export except `.nojekyll`): every response was 200
and every SHA-256 matched its local artifact. This includes every page, asset,
source snapshot and download. A missing path returned 404 with a body identical
to the checked custom error page. The ignored local
`book/artifacts/release-record.json` records the complete file manifest and results.

In Chromium at the actual public URL, the opening and first-edit link worked.
The movement lab accepted an exact-price step to `(3,2)` / 0. Searching for
`EnergySummary` reached Chapter 4, revealed its three folded references and showed
eight highlighted matches. A missing URL rendered the styled error page; its Home
link returned to the public opening. The repository link points to Moss, and the
repository's homepage was independently read back as the book URL.

This release does not add Safari/Firefox, screen-reader, native Windows or an
actual JavaScript-disabled session to the tested coverage. The plain Markdown and
static prose are present, but that is not a substitute for those execution checks.
Live biological rules, fixtures and tests remain unchanged. Only the local NOW,
documentation index and dated session log were updated afterward to link the
edition and preserve the one next learning action; these notes are outside both
public downloads and the published source snapshots.

## Second editorial pass — September 23, 2026

Nick requested independent reviews and editorial refinement. Two bounded helpers
reviewed the learning progression and later-chapter explanations using TutorBro,
then checked the integrated changes. Their local reports are
[the learning review](LEARNING_REVIEW_V2.md) and
[the editorial review](EDITORIAL_REVIEW_V2.md). These are source/editorial judgments,
not measurements of learning gains or new runtime evidence.

Corrected the claim that `AnimalEnergyCosts` was already in `energy.rs`; it remains
a future component. Clarified that the first helper command tests the reader's
edit while later reference commands run published answers. Added the mutable
slice loop's binding types, a conditional Idle-start trace, a shorter linked
route through costs, System/World glossary entries, a three-phase removal table,
and separate geometry/spread subsection destinations. The follow-up review
caught an imprecise table sentence: `map` yields pairs and `collect` gathers them.
That distinction is corrected in the final source.

The new trace is explicitly hypothetical: starting Idle at reserve 60, upkeep
one leaves reserve 45 after tick 15 without movement; tick 16's choice sees 44,
starts Seeking and pays two for a step, finishing at `(11,10)` / 42. Independent
arithmetic produced those values. This is not a new live fixture or integration test.

In Chromium at width 375, the 1000-pixel history diagram was displayed at about
307 pixels, making its fine labels hard to read. Added descriptive full-size
links to all ten figures. The local history link opened the SVG at its natural
1000-pixel width; at width 375 it retains that size for separate inspection.
Browser Back returned to the chapter. The new defaults shortcut and direct
comparison link reached their intended headings. The new Idle-start section
and table fit the 375-pixel reading page without page overflow. Temporary
viewport overrides were reset and the local review tab was closed.

The rebuilt edition passed **42 Python publishing/package tests**, **17 exact
canonical Rust-block comparisons**, 229 local destinations across 29 book
Markdown files, and 1,059 destinations across 51 output HTML pages. The project
documentation check passed 768 destinations across 77 Markdown files; whitespace
checks passed. Baseline hashes confirm no changes to `crates/` or `book/theme/`.
The starter ZIP remains byte-identical to the first release. The updated editable
source ZIP is 961,905 bytes, SHA-256:

```text
cf7cc873f59ff2122a1467cbf5db25a1ee620542aaf0d38244c264a6ed5aa5b3
```

Native Rust, WASM and JavaScript model execution were not rerun for these prose
and navigation edits. Their earlier evidence remains applicable to unchanged
code, without becoming a new execution claim. No new external-reference audit,
screen-reader audit or other-browser coverage was added.

The revised editable-source download was then extracted into a new scratch
workspace. Using the already installed pinned mdBook tool, its build and link
check passed and all **105 regenerated export files matched byte for byte**,
including both downloads. This rechecks the revised package's reproducibility;
it remains distinct from a clean-machine tool installation.

### Publication checkpoint for this refinement

The static export was committed and pushed normally on `gh-pages` as
`776dcec5fab903575ad73aacf424b417b24d9111`, retaining the first publication in
history. The generated search index changed its hashed name; only the obsolete
asset recorded in the previous release manifest was removed from the separate
publication checkout. That checkout matched all 105 new export files before push.

[Pages run 35879572240](https://github.com/NicholasHazen/moss/actions/runs/35879572240)
completed its build but failed during deployment with an OIDC identity-token
request timeout. A retry of failed jobs was accepted. Subsequent reads continued
to report queued, attempt 1, with no listed jobs; the Pages API reported the new
commit as building. After several minutes, cancellation of this retry was refused
as “completed,” while a full rerun was refused as “already running.” No cancellation
or second rerun succeeded. No permission, token or Pages configuration was changed.

The public Chapter 4 still differed from the reviewed export when checked with
system `curl`. Public publication of this refinement is therefore **pending**,
not successful. The local preview and committed export remain usable checkpoints.
The smallest remaining action is for GitHub to finish the queued retry, followed
by public file-hash and browser checks. The ignored
`book/artifacts/review-v2-release-record.json` identifies the exact expected files.
The original release's verification above must not be read as verification of
this new deployment.

## Follow-up Actions investigation — September 23, 15:45 UTC

At Nick's request, inspected fresh run, attempt, job and Pages state with a
bounded independent review of the official action source and recovery API.
Only the deployment job failed; the static build and upload succeeded. The
specific fault was an identity-token HTTP timeout. The permission footer is
emitted for any token-fetch error and does not establish missing permission.

The retry remained inconsistent after more than thirty minutes. One documented
force-cancel request returned HTTP 409, saying the rerun had not yet queued.
Stopped further run mutations after that conflict. The exact request ID, run
and job IDs, source links and minimal recovery path are recorded in the local
[publication incident](PUBLICATION_INCIDENT.md).

Fetched all 104 public files with system `curl`; every hash still matched the
first release. The refinement remains undeployed. No site configuration,
permissions, credentials, source or simulation tests changed. No support request
was sent. The working public release and verified local export remain available.

## Manual workflow recovery — September 23, 17:47 UTC

After Nick explicitly approved the prepared workaround, pushed workflow commit
`3d2b87673526064f3eb745e6c7cd99ce58442264`. Changed Pages from `legacy` to `workflow`
and added `main` alongside `gh-pages` in the existing deployment environment.
Settings were backed up before mutation. The manual workflow uses pinned official
actions, separate upload/deploy jobs, bounded timeouts and job-specific permissions.
Workflow lint and an independent read-only review passed. A fresh remote export
clone matched all 105 files in the reviewed `776dcec` manifest.

Dispatched the verified export `776dcec5fab903575ad73aacf424b417b24d9111` once.
[Run 35897888968](https://github.com/NicholasHazen/moss/actions/runs/35897888968)
completed successfully at 17:46:40 UTC. At 17:47:33 UTC, all 104 public files,
including both downloads, matched the refinement's hashes. A missing URL returned
HTTP 404 and the exact custom page. Pages reported `built`, build type `workflow`.
The old run still reported queued with its original timestamp, so this result
establishes recovered publication, not repair of the orphaned run.

No book-source content, teaching models or simulation rules changed. The verified
export was deployed unchanged; no native/WASM builds or browser interaction tests
were rerun. Maintainer documentation now describes manual publication; no Support
request was sent. The workflow and export have different commit IDs, recorded
separately in ignored `book/artifacts/manual-pages-recovery.json`.
