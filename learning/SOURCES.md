# Source registry

Retrieved or directly inspected on **September 23, 2026** unless stated
otherwise. The first-edition explanations, code probes and diagrams are original.
These sources supply facts, API verification, pedagogical inspiration and
optional further reading. A repository license is not a blanket license for
third-party assets, embedded videos or linked publications.

## Curriculum and API foundations

| ID | Primary source and checked version | Role in Moss | License/reuse status |
| --- | --- | --- | --- |
| R01 | [The Rust Programming Language](https://doc.rust-lang.org/book/), live edition-2024 text; introduction says Rust 1.90.0 or later. [Source repository](https://github.com/rust-lang/book). Moss remains pinned to 1.93.1. | Conceptual reference and optional remedial reading; local examples must compile on Moss's pinned version | Repository offers MIT or Apache-2.0; link and original explanation in this edition, no chapter reproduction |
| R02 | [Rustlings](https://rustlings.rust-lang.org/) and [Usage](https://rustlings.rust-lang.org/usage/), live documentation | Small repair exercises, compiler feedback, hints and a revisitable exercise list inform the interaction model; no mandatory linear gate | Site links MIT license; no upstream exercise suite vendored |
| R03 | [Bevy ECS 0.18.1 API](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/) | Exact-version reference for components, worlds, resources, typed queries and scheduling; standalone ECS supports renderer-free labs | Bevy code MIT or Apache-2.0 except separately noted material; link and original code |
| R04 | [Bevy 0.18.1 API](https://docs.rs/bevy/0.18.1/bevy/) and [v0.18.1 repository](https://github.com/bevyengine/bevy/tree/v0.18.1) | Presentation/reference integration; use versioned APIs instead of assuming current examples match | MIT or Apache-2.0 code; repository assets can have distinct licenses; no external assets copied |
| R05 | [Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html), live Cargo Book; actual runner version 1.93.1 | Focused check commands and distinction between build, test and filtered-out tests | Official reference linked; no document reproduction |
| R06 | [Cargo external tools / JSON messages](https://doc.rust-lang.org/cargo/reference/external-tools.html), live Cargo Book | Future runner diagnostics seam; do not mistake compiler JSON for proof that named runtime tests passed | Official reference linked; no document reproduction |
| R07 | [Rust standard-library documentation](https://doc.rust-lang.org/std/), live docs showing 1.98.1 at the final September 23 inspection | Linked per API throughout the course: options, results, collections, traits, pointers, threads, channels and futures. These links follow current documentation; actual course examples are tested on pinned Rust 1.93.1 | Official references linked and explained in original prose; no documentation or upstream examples reproduced |
| R08 | [Rust Reference](https://doc.rust-lang.org/reference/), live reference inspected September 23 | Precise language rules for visibility, lifetimes, trait objects, macros, hygiene and procedural derives; individual lesson links identify the relevant section. The runner establishes the pinned-version evidence | Link and original explanation only; no chapters reproduced |
| R09 | [Bevy error B0001](https://bevy.org/learn/errors/b0001/), official unversioned error guide inspected September 23 | Optional explanation for conflicting query access in lesson 10. The guide is not a version pin; a local Bevy 0.18.1 negative probe independently produced the stated initialization error | Link and original exercise only; no guide text or example copied |

Rustlings explicitly combines compile-error and test-based exercises and offers
watch-mode hints plus an exercise list. Moss borrows those interaction ideas while
writing ecosystem-specific tasks and explanations. The Book remains a reference,
not a second prerequisite course. [Rustlings usage](https://rustlings.rust-lang.org/usage/).

## Execution platform evidence

| ID | Primary source and checked version | Role / limitation | License/reuse status |
| --- | --- | --- | --- |
| P01 | [Rust Playground repository](https://github.com/rust-lang/rust-playground), `main` inspected on retrieval date | Confirms server/container compilation and resource boundaries; no fixed public-service SLA inferred | MIT or Apache-2.0; architecture summarized, code not vendored |
| P02 | [Playground crate policy](https://github.com/rust-lang/rust-playground/blob/main/CRATE_POLICY.md), `main` | Explains curated dependencies and why Playground availability is not a version pin | Same repository licenses; link and short factual summary |
| P03 | [Rust wasm32-unknown-unknown target](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html), current rustc book | Distinguishes a compilation target from a supported compiler host; std facilities have target-specific limits | Official reference linked; no document reproduction |
| P04 | [Weblings](https://github.com/AngelOnFira/weblings), `main`; [artifact manifest](https://github.com/AngelOnFira/weblings/blob/main/artifacts.lock) pins `artifacts-test-7` | Real browser compiler candidate; synthetic std success, E0382 and unresolved Bevy import observed in its [demo](https://weblings.forest-anderson.ca/). Compiler semver and full dependency support not verified | MIT repository; research use only, no compiler archive redistributed |
| P05 | [BrowserPod Rust announcement](https://labs.leaningtech.com/blog/browserpod-rust), 3.0, published August 13 and amended August 17, 2026 | Vendor explicitly separates running offline-compiled Rust from future browser-hosted rustc | Page says all rights reserved; link and original summary only; product terms not evaluated |
| P06 | [Runno repository](https://github.com/taybenlor/runno), `main` | WASI runtime candidate; documented source runtimes do not list Rust. Runtime alone does not supply a Rust compiler | No code incorporated; exact package/license audit deferred until adoption |
| P07 | [WebContainers](https://webcontainers.io/) and [browser support](https://webcontainers.io/guides/browser-support), live pages | Node-oriented browser tooling; isolation requirements affect deployment/embeds. Browser-support page labels its update February 2023, so it is not a current browser compatibility matrix | Vendor documentation linked; product redistribution/commercial terms not evaluated |

The tested execution route uses local Cargo for edits and precompiled WASM
for browser observations. A passed precompiled WASM probe does not certify any
browser-hosted compiler. Weblings' successful
synthetic programs do not certify Bevy 0.18.1, Cargo workspaces or offline reuse.

## Existing Moss material

| ID | Source | Intended use |
| --- | --- | --- |
| M01 | [Project brief](../PROJECT_BRIEF.md), [current assignment](../NOW.md), [architecture](../docs/design/architecture.md) | Learner ownership, one live assignment, one authoritative ECS world and simulation/presentation boundary |
| M02 | [Existing book](../book/README.md), [tutorial path](../docs/tutorial/path/README.md), [authoring contract](../docs/tutorial/authoring/README.md) | Consequence-led organization, explicit worked answers, helper/integration/browser evidence distinction |
| M03 | [Development commands](../docs/development/README.md), [tested stack](../docs/development/verification.md) | Existing exact-version run path and clearly bounded prior verification |

These are user-supplied project sources. Preserve repository notices for adapted
material. New lessons must label hypothetical future biology and keep the current
learner edit intact.

## Adding media or adapted material

### Initial viewing and reading shelf

| ID | Creator, work, date and canonical source | Placement and purpose | Reuse and verification |
| --- | --- | --- | --- |
| V01 | Jon Gjengset, [Crust of Rust: Lifetime Annotations](https://www.youtube.com/watch?v=rAl-9HwD858), April 22, 2020 | Shelf; ownership follow-up and module 25. Ask which owner constrains a returned reference. Publisher chapter markers: 17:10 missing lifetime, 57:49 multiple lifetimes, 1:03:19 str/String | Link and optional publisher-hosted YouTube player; no video or transcript redistributed. Title/date/markers verified from creator description on September 23, 2026; not viewed minute by minute. Original textual alternative supplied. Embedding behavior separately checked in browser evidence. |
| V02 | Jon Gjengset, [Crust of Rust: Iterators](https://www.youtube.com/watch?v=yozQ9C69pNs), May 27, 2020 | Shelf; modules 08–09. Follow the item type through a report pipeline. Publisher markers: 1:45 Iterator, 4:25 IntoIterator, 6:24 associated types | Same link/player-only policy; no third-party code copied. Markers and description verified September 23, 2026. Original explanation is sufficient without playback. |
| V03 | Glenn Fiedler, [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/), June 10, 2004 | Shelf and module 01/11 companion; distinguish pacing requests from fixed simulation steps and bounded catch-up | Copyright retained by author. Link and original short summary only. Primary page inspected September 23, 2026. Its physics examples do not certify Moss determinism. |

The shelf's plain links and text alternatives remain useful when a publisher
blocks embedding or the reader is offline. Loading a player is an explicit
reader action; page load makes no video-host request. Recorded videos are
conceptual supplements, not evidence for current dependency compatibility.

Add the exact creator, title, canonical URL, publication/version, retrieval date,
license or permission, lesson placement and learning purpose before inclusion.
For video, record whether publisher embedding is available and give a textual
alternative in our own words; do not redistribute an unlicensed transcript.
Show what to watch for and verified timestamps when useful. External media is
supplemental, never the only place a required concept is taught. No video or
third-party image was downloaded or redistributed by this platform spike.

For code adaptation, record the upstream file/revision and required notices. For
our own examples informed by API documentation, state that they are original and
record the compiler/test evidence. Do not present a source's current release as
tested with Moss merely because its landing page was reachable.

## Runtime host references

- [rustc target support: wasm32-unknown-unknown](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html), inspected September 23, 2026. Supports the target/platform distinction; actual compatibility comes from the recorded Rust 1.93.1 build and executions. Original host code, no documentation text copied.
- [Rust 2024 unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html), inspected September 23, 2026. Informs unique exported-symbol safety comments. No third-party code copied.

## Ecology context and advanced reference — September 23, 2026

- Mary Ann Clark, Matthew Douglas, Jung Choi; OpenStax, *Biology 2e*, section
  [46.2 Energy Flow through Ecosystems](https://openstax.org/books/biology-2e/pages/46-2-energy-flow-through-ecosystems).
  Publisher page inspected 2026-09-23. Purpose: ask which terms in the toy ledger
  are external input, transfer, and expenditure; distinguish authored units from
  empirical calibration. Link only, with original Moss questions; no figures,
  passages, transcripts, or exercises reproduced. Current publisher footer lists
  CC BY-NC-SA and additional AI-use conditions; no blanket reuse claim is made.
- Same authors/publisher, [19.1 Population Evolution](https://openstax.org/books/biology-2e/pages/19-1-population-evolution).
  Inspected 2026-09-23. Purpose: contextualize the difference between inherited
  variation and evidence of population change. Same link-only treatment. The
  text route concerns the course's own toy model rather than adapting the chapter.
- Rust project, [Rustonomicon introduction](https://doc.rust-lang.org/nomicon/).
  Inspected 2026-09-23. Optional advanced reference for unsafe boundaries and FFI;
  no new unsafe requirement for ordinary Moss systems. Link and original guidance
  only. Actual browser host compiled with Rust 1.93.1.


## Public narration boundary

The hosted edition omits the locally generated macOS voice recordings. The
installed macOS software licence, section 2F, restricts System Voices to the
stated personal uses and excludes public redistribution. Browser Listen uses
the reader's own speech service. The optional ElevenLabs generator is included
as source, but this release contains no ElevenLabs-generated recording.
