# Textbook edition — work card

**Published and verified:** September 23, 2026, at
<https://nicholashazen.github.io/moss/>. Work began at 07:56 UTC within the requested
twelve-hour allowance. The edition and public release are complete; the next
learner action remains the movement edit in `NOW.md`.

**Latest refinement:** export `776dcec` is **published and verified** as of
17:47 UTC. The new manual [publication run](https://github.com/NicholasHazen/moss/actions/runs/35897888968)
on workflow commit `3d2b876` succeeded; all 104 public files match its manifest,
and the custom 404 was verified. The old managed rerun still reports queued,
but it did not block this publication. See the [recovery record](WORKFLOW_RECOVERY.md)
for the authorized configuration change and future release instructions.

## Outcome

Produce a separate book with a coherent reading arc, original explanatory prose,
accurate Rust/ECS examples, purposeful diagrams and interactive labs, curated
primary references and a searchable web edition.
Preserve the existing guides and live learner-owned exercise. The checked package
was published only after Nick selected a public site linked to the repository.

## Decisions

- Source Markdown lives in `book/src/`; private editorial notes stay in
  `book/editorial/`. Only the source tree enters the web edition.
- mdBook **0.5.3** is pinned in project-local ignored tooling. It provides search,
  navigation and print output without adding a dependency to Moss.
- All interactive labs are explicitly teaching models. A browser model does not
  claim to be the live Rust simulation or evidence that its future rules ran.
- Ordinary Markdown contains the complete argument. Interactive views enhance
  it; the print edition and RustRover reader must remain understandable.
- No remote fonts, analytics or third-party interactive embeds are required.
- Nick selected **web book only** and **a public site linked to the Moss
  repository**. GitHub Pages serves a verified static export stored on `gh-pages`,
  through the manual publication workflow on `main`;
  the repository homepage links back to the book. No PDF edition is
  required; the ordinary browser print view remains a convenience.

## Completed edition

The opening, six main chapters, horizon coda, setup and reference pages build
locally. Five original teaching models have passed 28 Node tests. All five have
been exercised in Chromium. A new daylight strip previews the next executing
phase; Chapter 6 now has an original history/accounting diagram. Long answers
use native disclosures, with central functions still visible and exact commands
beside each reference. Search and keyboard disclosure checks passed.

Independent technical, editorial, teaching-model and publication reviews are
recorded alongside this file. Publication findings were repaired and 42 Python
tests pass. All 23 audited external references retrieved their intended material.
The downloadable starter has passed fresh native and WASM builds, the deliberate
red movement check, extracted reference checks and a short browser smoke test.

The first editable-source download rebuilt all 105 export files byte for byte,
including both ZIP packages. Initial publication commit `14b93fab06513aa6440150816e94044ae7936dd9`
deployed successfully on `gh-pages`. All 104 public files returned HTTP 200 with
matching hashes; a missing URL returned the exact custom 404. Public Chromium
checks covered navigation, movement, search/disclosures and error-page recovery.
The static branch preserves the existing working checkout and unrelated edits;
remote `main` remained `83a72a5abd46339e5448076fc183bfe33cd54029` at that first release.

## Second editorial pass

Two new independent reviews examined the learning route and later explanations,
then checked their integrated refinements:
[learning review](LEARNING_REVIEW_V2.md) and
[editorial review](EDITORIAL_REVIEW_V2.md). The edition now distinguishes a test
of the learner's edit from a complete-reference command at its opening, explains
the meal loop's mutable bindings, traces an illustrative Idle start, and offers
a shorter route through individual costs. Corrected the future status of
`AnimalEnergyCosts`; added System/World index entries, a removal-phase table,
geometry/spread headings and ten full-size figure links.

All seventeen canonical Rust blocks remain identical. The rebuilt site and
42 publishing checks passed; targeted Chromium checks covered new navigation,
the expanded figure and narrow reading layout. The updated source download
independently rebuilt all 105 files byte for byte. No runtime or teaching-model
code changed. Latest publication observations belong in [the evidence record](EVIDENCE.md).
The refinement's public hashes now match `book/artifacts/review-v2-release-record.json`.
Nick authorized switching to a manual workflow and allowing `main` alongside
`gh-pages` in the deployment environment. This restored publication without
Support or deleting the site. The old run remains a separate unresolved record;
its state should be checked before releasing newer content.

## Evidence

2026-09-23: inspected current source, tutorial contracts and the existing
worktree. Recorded SHA-256 hashes of 135 pre-existing tracked/untracked files in
`/private/tmp/moss-textbook-baseline.json` for comparison. Installed mdBook 0.5.3
with `cargo install --version 0.5.3 --locked` into the ignored local tool path;
the release build and installation completed successfully. The first source
and generated-site link checks passed, seventeen copied reference blocks match
their originals, all seventeen canonical future tests passed in isolation, and
the isolated movement helper/schedule checks and focused Clippy passed.

[The evidence record](EVIDENCE.md) preserves the initial observations separately
from the later full-book, extracted-package and public-site checks.

## Limits and handoff

Browser checks used in-app Chromium. Safari, Firefox, native Windows, screen-reader
behavior and an actual JavaScript-disabled session were not tested. Phone-width
checks establish fit and loading, not the readability of every figure label.
The extracted book reused the installed pinned mdBook binary; the starter reused
cached Rust dependencies. These are not clean-machine installation tests.

The book teaches later rules through isolated Rust examples and separate JavaScript
models. Those checks do not prove full future integration, model equivalence for
all inputs, ecological balance or enjoyment. The public edition notes state these
boundaries. Future revisions should respond to use of the book and changes to its
canonical examples, rather than introduce another active learner task.

The installed simulation remains maintenance-only. `move_one_cell` is still
unfinished and unscheduled; later biological behaviors remain future work.
