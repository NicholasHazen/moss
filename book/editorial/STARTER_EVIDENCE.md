# Edition starter workspace evidence

September 23, 2026. This record describes packaging checks, not fresh native,
WASM or browser acceptance of the extracted workspace.

## Contents and provenance

Added [`package_starter.py`](../scripts/package_starter.py),
[`test_starter.py`](../scripts/test_starter.py) and the public
[setup page](../src/reference/start-here.md). The generator produced
`book/artifacts/moss-starter-2026-09-23.zip` plus a checksum sidecar. No shared
build configuration, runtime source, chapter, navigation index or Git state was
changed by this helper.

The first generated archive contains **52 files / 273,312 bytes**. Its SHA-256 is:

```text
6544fa8514b594bb3df8ad818edc0e17d3c114f5ee5a3ea3ec553aaa588146a3
```

There are 31 verbatim files: the workspace/toolchain/Trunk files, both source
crates, the two public shell scripts and the isolated reference runner. Eighteen
generated Markdown files preserve exactly the canonical Rust fences for the
sixteen future sessions and both movement answers. A generated README and
`STARTER-MANIFEST.json` complete the archive.

Extraction-only references were chosen deliberately: all published commands
remain executable without copying unrelated documentation or giving readers
broken navigation through a partial guide tree. Every extracted file names its
canonical source and source hash, labels itself as an extraction, and points to
the web book for prose. The manifest distinguishes verbatim, extracted and
generated files, with per-file hashes and normalized modes.

The provenance names reference checkpoint
`83a72a5abd46339e5448076fc183bfe33cd54029` and explicitly says this is a snapshot
with local modifications, not a clean archive of that commit. It contains no
Git history, `NOW.md`, agent/collaboration material, editorial notes, caches,
compiled outputs or private IDE state. Optional shared `.run` configurations
were omitted because some currently select a remote build target; terminal
commands are the portable entry path for this bundle. No license declaration
was added.

## Checks actually performed

Ran:

```sh
python3 -B -m unittest discover -s book/scripts -p test_starter.py -v
python3 -B book/scripts/package_starter.py
```

All **12 unit tests passed** using isolated temporary fixtures. They cover the
allowlist, private/hidden file exclusion, symlink and unexpected-extension
rejection, unchanged Rust/lockfile bytes, executable shell modes, complete hash
inventory, deterministic ZIP metadata, exact reference extraction, and rejection
of a solved helper or activated schedule. Adding a crate or duplicating a
canonical reference requires explicit review instead of silent inclusion.

A separate read-only comparison of the actual archive confirmed all 31 verbatim
files equal their working-tree bytes. All 18 extracted Rust fences equal the
canonical runner's `tagged_example` output. Building the archive again in memory
produced identical bytes. The ZIP uses sorted entries, fixed timestamps and
stored bytes rather than compression, avoiding compressor-version variability.

The live helper remains unfinished and the maintenance-only schedule remains
unchanged. The generator validates this checkpoint shape before export; it is
a deliberately narrow guard, not a general Rust parser or proof about arbitrary
future source. The generator executes no compiler, test runner, installer,
network request or Git mutation.

## Setup references checked

The public setup page uses the
[official rustup installation guide](https://rust-lang.github.io/rustup/installation/index.html),
[Trunk's current official setup guide](https://trunk-rs.github.io/trunk/guide/getting-started/index.html),
[wasm-bindgen CLI guide](https://rustwasm.github.io/docs/wasm-bindgen/reference/cli.html),
and [Cargo fetch reference](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html).
Cargo's [install reference](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
also confirmed exact `--version` selection and `--locked` behavior. Pinned
versions were taken from the existing project configuration and development
record, not silently upgraded.

The old `trunkrs.dev` guide URL redirected to unrelated casino content during
this check. The parent confirmed the replacement `trunk-rs.github.io/trunk`
site through the Trunk repository's About link and successfully fetched the new
guide. The obsolete hostname is not linked by the setup page.

## Integration and remaining checks

The default generator writes only into `book/artifacts`. After a normal book
build, root can run:

```sh
python3 -B book/scripts/package_starter.py --copy-to-site
```

That writes the same ZIP and checksum under `book/_site/downloads`. Root still
owns adding the setup page to navigation, handling generated download links in
the source checker, and including this step in the build wrapper. At this handoff
the helper has not copied into the shared `_site` output.

The parent must run the green baseline, deliberate red movement checkpoint,
isolated references and WASM build from an extracted copy before claiming those
commands passed for the downloadable workspace. Check extraction preserves the
executable script modes on the chosen platform. Python's `ZipFile.extractall`
does not promise to restore Unix executable permissions, so use a mode-preserving
archive tool or apply the manifest's modes explicitly in an automated check.
Fresh-machine installation, native Windows and every ZIP application's extraction
behavior remain unverified. Public download routing and its checksum also need
checking after deployment.
