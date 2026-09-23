# Editable edition package

September 23, 2026. Added
[`package_edition.py`](../scripts/package_edition.py) and
[`test_edition.py`](../scripts/test_edition.py). The package contains the editable
book and the bounded source inputs needed by its build, checks and both download
generators. It does not depend on the local editorial directory.

## Inclusion rules

The generator assembles three explicit sets:

- `book/src`, `book/theme` and `book/scripts`, with narrow extension checks,
  plus `book/book.toml`. This includes the edition's own generator and tests.
- The starter generator's exact Cargo/toolchain/Trunk files, two source crates,
  public shell scripts and canonical eighteen reference pages. The canonical
  pages are retained byte for byte here so this package can reproduce the
  smaller starter's extracted references and source hashes exactly.
- Files directly cited by book Markdown, using the existing Markdown helper
  and the snapshot packager's allowed source areas. Only those direct files
  are copied; their own Markdown links are not followed. `check_docs.py` and
  `check_path_examples.py` are explicit build dependencies, not permission to
  copy the tutorial authoring directory generally.

The public README is generated at the archive root and at `book/README.md`.
The local maintenance README is not copied. The generated text explains Python
3.11+, the pinned local mdBook installation, build/test/serve commands, exact
working-tree provenance, the incomplete movement exercise and the limited
documentation tree. The source crate comments and directly cited guides remain
unchanged; their references to unbundled project documents are not expanded into
additional exports.

No `AGENTS.md`, `NOW.md`, Git history, personal IDE configuration, `.run` files,
collaboration material, `book/editorial`, caches, compiled outputs or existing
download ZIPs are included. Symlinks and unexpected public source types fail
closed. Known Python bytecode directories are skipped. Passive SVG validation
uses the same function as site packaging. No licensing declaration was added.

## Artifact and determinism

The default command writes only into `book/artifacts`:

```sh
python3 -B book/scripts/package_edition.py
```

It produces `moss-book-source-2026-09-23.zip` and a `.zip.sha256` sidecar.
`--copy-to-site` additionally writes those files beneath `_site/downloads`, for
use after mdBook has cleared and rebuilt the site. Root is integrating that
option into the normal wrapper after starter packaging.

The archive has one named top-level directory. `EDITION-MANIFEST.json` records
every other file, its byte length, SHA-256 digest, normalized mode and whether
it was copied or generated. Entries are sorted, timestamps fixed, permissions
explicit, and ZIP contents stored without a compressor. Identical input bytes
produce identical archive bytes. Neither an absolute workstation path nor the
current clock is introduced into the generated metadata.

## Checks actually run

```sh
python3 -B -m unittest discover -s book/scripts -p test_edition.py -v
python3 -B book/scripts/package_edition.py
```

All **10 fixture tests passed**. They check required build inputs, exact source
bytes, exclusions, direct-only citation discovery, generated-download handling,
forbidden citations, symlinks, unexpected file types, manifest completeness and
byte-identical rebuilds from extracted fixture sources.

The first actual package held **111 files / 956,620 bytes**, including **43 direct
project citations**. Its SHA-256 was:

```text
a87841f1dec978f460ee8e79fdad6226cd35e3e0c3c45f9a930f9c4c8472d3de
```

This is an **intermediate artifact identity**, not the final publication hash:
root subsequently integrated the wrapper/download checks, and the generated
README was updated to remove its now-redundant separate packaging command.
The final root build must regenerate the source archive after those edits.

For that initial actual archive, an isolated temporary extraction check verified
all manifest hashes and every verbatim file against its working-tree bytes.
Executing the **included** edition generator from the extracted workspace
reproduced the source ZIP byte for byte. Executing its included starter generator
also reproduced the existing starter ZIP byte for byte. The extraction check
restored the ZIP's Unix modes explicitly. It did not invoke a compiler, network,
Git, browser or shared output build.

## Handoff and limits

The parent owns the final integrated archive, a complete mdBook build from a new
extraction, source/HTML checks on that build, public download links and deployment.
The generator tests establish packaging properties; they do not substitute for
the extracted book build. The source package intentionally cannot satisfy a
full-project documentation-link scan, because the copied project guides may
refer to excluded history or collaboration material. The supported navigation
check is `book/scripts/check_book.py`, for the book and its direct citations.

No live source, existing guide, shared build script, book configuration, chapter
or Git state was edited by this helper. The two generators preserve the
unfinished learner-owned rule. The editable archive adds no private publication
of the local editorial records and makes no claim of fresh-machine installation
or cross-platform build verification.
