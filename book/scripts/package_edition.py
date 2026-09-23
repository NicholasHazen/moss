#!/usr/bin/env python3
"""Package the editable web-book sources and their exact, bounded build inputs.

No build, network operation or Git command runs here. Direct Markdown citations
are collected once; the links inside copied project documents are not followed.
The public README is generated separately from local editorial instructions.
"""
from __future__ import annotations

import argparse
import io
import json
import os
from pathlib import Path
import stat
import zipfile
from urllib.parse import unquote, urlsplit

import finish_site
import package_starter as starter
from check_docs import _links, _visible_lines

ROOT = Path(__file__).resolve().parents[2]
NAME = "moss-book-source-" + starter.EDITION
SOURCE_AREAS = tuple(path.relative_to(finish_site.ROOT) for path in finish_site.ALLOWED)
BOOK_TREES = {
    "book/src": {".md", ".svg"},
    "book/theme": {".css", ".js", ".hbs"},
    "book/scripts": {".py", ".cjs", ".sh", ".js"},
}
REQUIRED = {
    "book/book.toml", "book/src/SUMMARY.md", "book/src/index.md",
    "book/scripts/book.sh", "book/scripts/finish_site.py",
    "book/scripts/check_book.py", "book/scripts/package_starter.py",
    "book/scripts/package_edition.py", "book/scripts/serve.py",
    "docs/tutorial/authoring/check_docs.py",
    "docs/tutorial/authoring/check_path_examples.py",
}
EXTRA_INPUTS = {"book/book.toml", "docs/tutorial/authoring/check_docs.py"}
EXECUTABLES = starter.EXECUTABLES | {"book/scripts/book.sh"}


def book_paths(root: Path) -> set[str]:
    result = set(EXTRA_INPUTS)
    for relative, suffixes in BOOK_TREES.items():
        directory = root / relative
        if directory.is_symlink() or not directory.is_dir():
            raise ValueError(f"Expected ordinary public book directory: {relative}")
        for path in sorted(directory.rglob("*")):
            name = path.relative_to(root)
            if path.is_symlink():
                raise ValueError(f"Symlink is not an edition source: {name}")
            if "__pycache__" in name.parts:
                continue
            if any(part.startswith(".") for part in name.parts):
                raise ValueError(f"Hidden file is not an edition source: {name}")
            if path.is_dir():
                continue
            if not path.is_file() or path.suffix not in suffixes:
                raise ValueError(f"Unexpected public book source type: {name}")
            result.add(name.as_posix())
    return result


def direct_citations(root: Path) -> set[str]:
    """Collect only direct repository citations, with the snapshot area's policy."""
    result: set[str] = set()
    errors: list[str] = []
    generated_downloads = {
        root / "book/src/downloads" / (base + ending)
        for base in (starter.NAME, NAME) for ending in (".zip", ".zip.sha256")
    }
    for page in sorted((root / "book/src").rglob("*.md")):
        starter.read_source(root, page.relative_to(root).as_posix())
        for _, destination in _links(_visible_lines(page, errors), page, errors):
            url = urlsplit(destination)
            if url.scheme or url.netloc or not url.path:
                continue
            # normpath removes .. without hiding an intermediate symlink, which
            # read_source rejects component-by-component before any read.
            target = Path(os.path.normpath(page.parent / unquote(url.path)))
            if target in generated_downloads:
                continue
            try:
                relative = target.relative_to(root)
            except ValueError as error:
                raise ValueError(f"Citation escapes the source repository: {destination}") from error
            if target.is_relative_to(root / "book/src"):
                continue  # Already included in the public book tree.
            if (not any(relative.is_relative_to(area) for area in SOURCE_AREAS)
                    or "authoring" in relative.parts
                    or any(part.startswith(".") for part in relative.parts)
                    or any(part in {"agents", "editorial", "history"} for part in relative.parts)
                    or target.suffix not in {".md", ".rs", ".toml", ".svg"}):
                raise ValueError(f"Disallowed direct citation: {relative}")
            data = starter.read_source(root, relative.as_posix())
            if target.suffix == ".svg":
                finish_site.validate_figure(data)
            result.add(relative.as_posix())
    if errors:
        raise ValueError("Invalid book Markdown: " + "; ".join(errors))
    return result


def public_readme() -> bytes:
    return f"""# Rebuild the Moss web book — {starter.EDITION}

This is the editable source edition of [Moss — A Small World, Understood]({starter.BOOK_URL}).
Open `book/src/index.md` to read, or edit the Markdown in `book/src/chapters`.
The lab code and styles live in `book/theme`; publishing helpers and their tests
live in `book/scripts`. No editorial or collaboration records are included.

Run the commands below from the extracted workspace root, the directory with
`Cargo.toml`. It contains the project's exact source and lockfile, plus only the
canonical references and directly cited documents needed to rebuild this book.
Those copied documents are build/reference inputs; this is not the complete Moss
documentation tree, and their own links are not recursively packaged.

## Install the publishing tool once

Use a Unix shell, Python **3.11 or later** for the publishing helpers, and Rust
through [rustup](https://rust-lang.github.io/rustup/installation/index.html).
Node is needed for the lab tests. From this extracted workspace:

```sh
rustup toolchain install 1.93.1 --component rustfmt --component clippy --target wasm32-unknown-unknown
scripts/with-toolchain.sh cargo install mdbook --version 0.5.3 --locked --root .tools/mdbook-0.5.3
```

The wrapper checks that local mdBook version. Its installation and the generated
site are deliberately absent from the archive. Tool installation needs network
access and the platform's native compiler/linker toolchain.

## Build, check and read

```sh
sh book/scripts/book.sh build
python3 -B book/scripts/check_book.py
python3 -B -m unittest discover -s book/scripts -p 'test_*.py'
node --test book/scripts/test_*lab*.cjs
sh book/scripts/book.sh serve
```

The fixed preview opens at http://127.0.0.1:8090/moss/ . Rebuild after edits,
then reload the browser. `book/_site` holds the static publication files;
`book/artifacts` holds the two reproducible download archives and checksums.
`package_starter.py` builds the reader's smaller coding workspace; the edition
packager builds this editable source download. Neither invokes a compiler or
changes the simulation. Copying a download into `_site` must follow mdBook's
build because mdBook clears the old output first.

If your ZIP extraction tool did not preserve executable modes, restore the
shell wrappers once with `chmod +x scripts/check.sh scripts/with-toolchain.sh`.
The archive manifest records normalized modes and exact per-file SHA-256 hashes.
The generator uses sorted entries and fixed ZIP metadata so the same source
bytes produce the same archive; no current date or machine path is embedded.

## The simulation checkpoint is still an exercise

The Rust source is a working-tree snapshot based on repository checkpoint
`{starter.REFERENCE_CHECKPOINT}`, including local modifications. It is not a
clean archive of that commit. `EDITION-MANIFEST.json` names every other included
file and its hash. The public repository is https://github.com/NicholasHazen/moss .

Movement remains unfinished and unscheduled. Read the book's setup page for the
application tool versions, green bootstrap/maintenance test command and the
deliberately red movement checkpoint. Full workspace tests stop at that exercise.
Worked-reference commands execute in temporary copies; they do not install later
biological rules in this source. Book build tests and simulation checks establish
different facts.

This package does not claim a fresh installation on every operating system.
The reading edition records what was actually verified and what remains untested.
""".encode("utf-8")


def build_archive(root: Path) -> tuple[bytes, dict]:
    root = root.resolve()
    paths = book_paths(root) | set(starter.source_paths(root))
    paths.update(path for path, _ in starter.reference_paths(root))
    citations = direct_citations(root)
    paths.update(citations)
    missing = REQUIRED - paths
    if missing:
        raise ValueError("Missing edition build inputs: " + ", ".join(sorted(missing)))
    files = {relative: starter.read_source(root, relative) for relative in sorted(paths)}
    starter.validate_exercise(files)
    for relative, data in files.items():
        if relative.endswith(".svg"):
            finish_site.validate_figure(data)
    # Do not copy the checkout's README, whose maintenance links are local.
    files["book/README.md"] = public_readme()
    files["README.md"] = public_readme()
    generated = {"README.md", "book/README.md"}
    manifest = {
        "format": 1, "edition": starter.EDITION, "archive_root": NAME,
        "reference_checkpoint": starter.REFERENCE_CHECKPOINT,
        "source_state": "working-tree snapshot including local modifications; not a clean commit archive",
        "repository": "https://github.com/NicholasHazen/moss",
        "build_tool": "mdbook 0.5.3",
        "direct_project_citations": sorted(citations),
        "files": [{
            "path": relative, "sha256": starter.digest(data), "bytes": len(data),
            "mode": "0755" if relative in EXECUTABLES else "0644",
            "kind": "generated" if relative in generated else "verbatim",
        } for relative, data in sorted(files.items())],
        "manifest_note": "Lists every other archive file; excludes this manifest to avoid a circular hash.",
    }
    files["EDITION-MANIFEST.json"] = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_STORED) as archive:
        for relative, data in sorted(files.items()):
            item = zipfile.ZipInfo(f"{NAME}/{relative}", date_time=starter.ZIP_TIMESTAMP)
            item.create_system = 3
            mode = 0o755 if relative in EXECUTABLES else 0o644
            item.external_attr = (stat.S_IFREG | mode) << 16
            archive.writestr(item, data)
    return buffer.getvalue(), manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--copy-to-site", action="store_true")
    args = parser.parse_args(argv)
    data, manifest = build_archive(ROOT)
    checksum = (starter.digest(data) + "  " + NAME + ".zip\n").encode()
    destinations = [ROOT / "book/artifacts"]
    if args.copy_to_site:
        destinations.append(ROOT / "book/_site/downloads")
    for directory in destinations:
        directory.mkdir(parents=True, exist_ok=True)
        (directory / (NAME + ".zip")).write_bytes(data)
        (directory / (NAME + ".zip.sha256")).write_bytes(checksum)
    print(f"Edition source: {len(manifest['files']) + 1} files, {len(data)} bytes; SHA-256 {starter.digest(data)}")
    print(f"Wrote book/artifacts/{NAME}.zip" + (" and the site download copy" if args.copy_to_site else ""))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
