# Reading the guide in RustRover

[Guide home](README.md) · [Start Chapter 1A](01-species-energy.md#checkpoint-a--describe-the-settings)

The useful arrangement is the lesson beside the file you are editing. You
should be able to follow a link, run one test, and return to the same checkpoint
without searching through the rest of the project.

## On this page

- [Choose a reading view](#choose-a-reading-view)
- [Move between the lesson and code](#move-between-the-lesson-and-code)
- [Run a checkpoint](#run-a-checkpoint)

## Choose a reading view

Open `docs/tutorial/README.md` from the Project tree. Markdown's **Preview** mode
gives the prose the available width; **Editor and Preview** shows its source and
rendering together. Use the controls at the top right of the Markdown editor.
These modes and relative file/heading links are supported by
[RustRover's Markdown editor](https://www.jetbrains.com/help/rust/markdown.html).

For coding, keep a chapter open and open the linked Rust file beside it. The
chapter's contents list jumps to a checkpoint, while **Guide home**, **Previous**
and **Next** links connect the pages. The filenames stay numbered so the Project
tree gives the same order as the guide.

If the text feels small, adjust **Preview font size** under
**Settings → Languages & Frameworks → Markdown**. The guide uses ordinary
Markdown and your IDE's existing colors. There is no custom CSS to maintain.
Tables stay narrow, and code fences name their language for syntax highlighting.

## Move between the lesson and code

Links use paths relative to their Markdown file, so the guide travels with the
repository. In the rendered preview, click a link to another page or source file.
When a checkpoint says “find `pub struct Energy`,” that symbol is the landmark;
line numbers may move as the code grows.

A displayed code block can be a replacement function, a new declaration, an
excerpt or an illustrative trace. Read the sentence immediately above it before
pasting. RustRover may analyze a Rust fence without the surrounding file's imports;
the named Cargo test is the check that compiles the code in its real context.
Future chapter APIs are explicitly marked as preparation, so a missing function
there is not a problem in your current project.

Optional context links are invitations to investigate, not prerequisites. Each
context page links back to the chapter where its idea first matters.

## Run a checkpoint

Use **Moss - Simulation tests** for Chapter 1A. For a single later regression,
use the Run gutter beside its `#[test]` in the Rust file, or the exact terminal
command in the lesson. Choose normal **Run** while the native debugger stall
remains unresolved. Build and preview setup lives in [the IDE guide](../RUSTROVER.md).

Check that the named test actually ran. If you pasted a command into a terminal,
its working directory should be `/Users/nick/Code/moss`. A “could not find
Cargo.toml” message means the terminal is elsewhere; it does not diagnose your
Rust edit. A test failure prints the mismatched values so we can inspect what
the rule did.

When the checkpoint is green, the next action is review. Keep the lesson open at
its stopping point and send the short request printed there; the next visit
starts from the updated [NOW.md](../../NOW.md).
