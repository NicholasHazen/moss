# What this edition establishes

This is the September 23, 2026 textbook edition of Moss. It begins with a
browser foundation and an implemented maintenance rule. Movement is the reader's
unfinished first edit. Chapters about meals, choice, individual costs, growth
and death contain complete worked proposals; they do not describe installed
behaviors in the starting application.

The [starter workspace](start-here.md) fixes that starting point independently
of later repository changes. Its manifest lists every included file and its
SHA-256 digest. Repository citations in the book open captured source text,
with the original path and digest, rather than silently following a moving
branch. Ordinary links to outside reference material remain external.

To edit or rebuild the book itself, use the
[editable source ZIP](../downloads/moss-book-source-2026-09-23.zip)
and its [checksum](../downloads/moss-book-source-2026-09-23.zip.sha256).
It includes Markdown, figures, teaching models, publishing scripts and the exact
code/reference inputs they need. Its README gives the build path; editorial and
collaboration records remain outside the package. The smaller
[starter download](start-here.md) is the recommended choice for the coding lesson.

## Different checks establish different facts

The native Rust checks use the pinned Rust 1.93.1 toolchain and Cargo lockfile,
including Bevy ECS 0.18.1. Browser packaging uses Trunk 0.21.14 and
wasm-bindgen 0.2.123. The book itself uses mdBook 0.5.3; it adds no dependency
to the Moss Cargo workspace.

| Check performed for this edition | What it established |
| --- | --- |
| Five baseline tests in a freshly extracted starter | The authored fixture, tick/reset behavior and maintenance work in the shipped source. |
| The starter's focused movement test | Exactly one selected test reaches the intended `todo!()` failure. The two activation tests remain ignored. |
| A fresh starter WebAssembly release build | The bundled browser source compiles and packages with the pinned tools. |
| Starter browser observation in in-app Chromium | The scene loads; three Steps leave Fern and Flint at 57 energy in their original cells; Reset starts run 2 and restores Fern to 60. No warning/error log entries were observed in this smoke check. |
| Seventeen named tests across sixteen isolated future references | The complete future answers execute their selected tests; they are not installed in the starter. |
| Isolated movement answer and schedule checks | The printed helper passes its focused test; five movement/maintenance checks and focused native Clippy pass with activation only in a temporary copy. |
| Exact-reference comparisons | The book's seventeen marked Rust blocks match their canonical guide answers byte for byte. |
| Twenty-eight Node tests of five teaching models | Selected JavaScript traces, input bounds, reset, ownership and daylight-marker contracts pass independently of Rust. |

Browser checks also exercised the teaching models' displayed state changes:
an exact-price step and rejected step; a four-plus-one shared meal; choice
before a meal; draft/default/owned-cost distinctions; and dusk, stored night
food, dawn and full-patch growth. The models are separate illustrations.
Passing their tests is not proof of equivalence for every possible Rust input.

The local book was checked at its `/moss/` URL prefix, including the custom
error page and its navigation. Chapters fit a 375-pixel viewport without page
overflow. Search can reveal folded answers, and the native answer disclosures
can be opened from the keyboard. Original diagrams and the main reading surface
were inspected during editing. The [reference shelf](reading-shelf.md) points
to primary material; its destinations were retrieved and checked for subject
and version during this edition's review.

## The limits matter when you use the result

The starter smoke check is a short observation, not a new exhaustive test of
every camera, input or hidden-tab interaction. The public book's future rules
have not been tested as one installed ecosystem. No animal in the starter
chooses food, eats, reproduces or dies. A worked predicate about starvation
does not change that fact.

The Rust and WebAssembly builds ran on macOS with tools and dependencies already
available. A clean installation on another machine, Linux, native Windows,
Safari and Firefox have not been verified for this edition. The book's static
prose, tables and native disclosures remain available without its JavaScript
models; a browser session with JavaScript disabled and a screen-reader audit
have not yet been performed. Full live-workspace tests intentionally remain red
at the learner-owned movement exercise.

These boundaries leave a useful next step: [make one affordable step pass its
named test](../chapters/01-one-affordable-step.md#your-first-edit), then review
and activate that one rule before adding another.
