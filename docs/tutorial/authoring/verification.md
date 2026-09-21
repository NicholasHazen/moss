# Guide verification record

[Guide home](../README.md) · [Maintaining the guide](README.md)

The guide targets this repository's pinned Rust 1.93.1 and Bevy ECS 0.18.1.
Worked code is checked in an isolated copy; it is not installed into the live
simulation by writing this guide. The validation record below distinguishes
tested examples from the future integration/browser acceptance checks.

<!-- tutorial-validation -->
## Worked examples, September 21, 2026

The original tested guide was saved in `ee514e9`; live implementation was the
foundation and maintenance rule, with IDE setup from `7c8a22a`. Source anchors
are [lib.rs](../../../crates/moss-sim/src/lib.rs),
[lessons.rs](../../../crates/moss-sim/src/lessons.rs), and
[bootstrap.rs](../../../crates/moss-sim/tests/bootstrap.rs).
This is a historical record of that validation, not an assertion that future
edits have been retested automatically.

Observed:

- Chapter 1A's resource/default example compiles with the existing six simulation tests passing.
- Chapter 1B's new regression fails for the intended reason: Fox actual 57 versus expected 54. Chapter 1C's worked system makes it pass; all seven simulation tests then pass.
- Each helper test in Chapters 3–5 fails against its unfinished stub and passes against the exact worked function. The food-choice test also checks that a closer patch beats a lower-ID, farther patch.
- A separate temporary regression confirms reset preserves custom species rates. With all examples and that check installed in the temporary copy, 11 native simulation tests pass. Formatting and simulation Clippy with warnings denied also pass.
- The live repository's six simulation tests still pass. No live Rust source, dependency or biological behavior was changed for this guide.

Commands used in the temporary copy: the focused `cargo test` commands printed
in the chapters, `cargo test -p moss-sim --locked`, `cargo fmt --all`, and
`cargo clippy -p moss-sim --all-targets --locked -- -D warnings`, through the
repository's toolchain wrapper. Two read-only subagent reviews checked API
accuracy, test coverage and the size of the learning steps.

**Not yet implemented or verified:** population scaffolding, the installed
choice/movement/eating systems, their future integration tests, and the browser
acceptance checks described in these chapters. The helper examples do not claim
those later behaviors work. No new WASM/browser smoke test was run for this
documentation-only change; the current foundation's evidence remains in
[build notes](../../BUILD_NOTES.md).

## TutorBro revision, September 21, 2026

**Reviewed base:** `ee514e9`, plus this documentation revision. No source or
dependency changes were present. Read the actual TutorBro `SKILL.md`, prose
guide and exemplar reference through Nick's shared skill page in the signed-in
browser. The authoring contract records the source and selected techniques.

Preserved all existing Rust and shell fences in the five chapters byte-for-byte
against `ee514e9`. Their earlier executable validation therefore remains the
relevant record; no future behavior was added to the live simulation. The existing
focused maintenance command was run again while preparing the context pages:
one test passed.

Checked all guide and entry-point local file links and heading anchors, fenced
block closure, and patch whitespace. Two bounded helpers prepared authoring and
context pages; a subsequent understanding/prose review corrected the distinction
between simulation reset, browser camera reset, and future cross-run history.

**RustRover observation:** opened guide home and Chapter 1 directly in the
installed EAP. Both render in Editor and Preview mode with headings, linked
contents, paragraphs and highlighted Rust code. The preview's automatic click
attempt did not navigate, so end-to-end clicking of every link is not claimed.
The link destinations and anchors were checked statically; direct file opening
worked. No extra Markdown plugin or global appearance setting was installed.

**Unchanged limits:** population/foraging integration and future browser behavior
remain acceptance criteria. No simulation WASM/browser smoke test was repeated
for this prose and navigation revision.
