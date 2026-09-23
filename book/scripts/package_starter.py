#!/usr/bin/env python3
"""Build this edition's small starter ZIP without changing the live exercise.

Only explicit files and the two source crates are read. Publication integration
can pass --copy-to-site after building the book. No compiler, installer, Git
mutation or network request is run. Python's standard library is sufficient.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import stat
import zipfile

ROOT = Path(__file__).resolve().parents[2]
EDITION = "2026-09-23"
NAME = "moss-starter-" + EDITION
REFERENCE_CHECKPOINT = "83a72a5abd46339e5448076fc183bfe33cd54029"
BOOK_URL = "https://nicholashazen.github.io/moss/"
FIXED_FILES = (
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "Trunk.toml",
    "scripts/check.sh", "scripts/with-toolchain.sh",
    "docs/tutorial/authoring/check_path_examples.py",
)
CRATES = ("moss-sim", "moss-web")
SOURCE_SUFFIXES = {".rs", ".toml", ".js", ".css", ".html"}
EXECUTABLES = {"scripts/check.sh", "scripts/with-toolchain.sh",
               "docs/tutorial/authoring/check_path_examples.py"}
ZIP_TIMESTAMP = (2026, 9, 23, 0, 0, 0)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_source(root: Path, relative: str) -> bytes:
    """Reject symlinks in every selected path component, including directories."""
    path = root
    parts = Path(relative).parts
    if not parts or Path(relative).is_absolute() or ".." in parts:
        raise ValueError(f"Unsafe source path: {relative}")
    for part in parts:
        path /= part
        if path.is_symlink():
            raise ValueError(f"Symlink is not a starter source: {relative}")
    if not path.is_file():
        raise ValueError(f"Missing ordinary starter source: {relative}")
    return path.read_bytes()


def source_paths(root: Path) -> list[str]:
    paths = list(FIXED_FILES)
    crates = root / "crates"
    if crates.is_symlink() or not crates.is_dir():
        raise ValueError("Expected an ordinary crates directory")
    if {path.name for path in crates.iterdir()} != set(CRATES):
        raise ValueError("Review the starter allowlist when the crate inventory changes")
    for name in CRATES:
        directory = crates / name
        if directory.is_symlink() or not directory.is_dir():
            raise ValueError(f"Expected an ordinary source crate: {name}")
        for path in sorted(directory.rglob("*")):
            relative = path.relative_to(root)
            if path.is_symlink():
                raise ValueError(f"Symlink is not a starter source: {relative}")
            if any(part.startswith(".") or part in {
                "target", "dist", "__pycache__", "agents", "editorial", "history",
            } for part in relative.parts):
                raise ValueError(f"Private or generated crate path: {relative}")
            if path.is_dir():
                continue
            if not path.is_file() or path.suffix not in SOURCE_SUFFIXES:
                raise ValueError(f"Unexpected crate source type: {relative}")
            paths.append(relative.as_posix())
    return sorted(paths)


def extract_reference(data: bytes, marker: str, source: str) -> bytes:
    text = data.decode("utf-8")
    pattern = rf"^<!-- {re.escape(marker)} -->\s*```rust\n(.*?)\n```"
    blocks = re.findall(pattern, text, re.MULTILINE | re.DOTALL)
    markers = re.findall(rf"^<!-- {re.escape(marker)} -->$", text, re.MULTILINE)
    if len(blocks) != 1 or len(markers) != 1:
        raise ValueError(f"Expected one complete {marker} reference in {source}")
    return (
        f"# Extracted worked reference: {marker}\n\n"
        "This generated file contains only the canonical Rust example required\n"
        "by the isolated reference runner. It is not the full tutorial page.\n"
        f"Read the lesson at {BOOK_URL}.\n\n"
        f"Original source: `{source}`.\n"
        f"Original source SHA-256: `{digest(data)}`.\n\n"
        f"<!-- {marker} -->\n```rust\n{blocks[0]}\n```\n"
    ).encode("utf-8")


def reference_paths(root: Path) -> list[tuple[str, str]]:
    result = [
        ("docs/tutorial/today-v2.md", "example: movement-v2-helper"),
        ("docs/tutorial/04-movement.md", "example: movement-helper"),
    ]
    for number in range(1, 17):
        matches = sorted((root / "docs/tutorial/path").glob(f"{number:02d}-*.md"))
        if len(matches) != 1:
            raise ValueError(f"Expected one canonical source for session {number:02d}")
        result.append((matches[0].relative_to(root).as_posix(), f"runnable: session-{number:02d}"))
    return result


def validate_exercise(files: dict[str, bytes]) -> None:
    lessons = files["crates/moss-sim/src/lessons.rs"].decode("utf-8")
    helpers = re.findall(r"^pub fn move_one_cell\(\n.*?^\}", lessons,
                         re.MULTILINE | re.DOTALL)
    if len(helpers) != 1 or 'todo!("Paired movement exercise:' not in helpers[0]:
        raise ValueError("This edition requires the unfinished move_one_cell exercise")
    schedule = files["crates/moss-sim/src/simulation.rs"].decode("utf-8")
    expected = "schedule.add_systems((lessons::spend_energy, complete_tick).chain());"
    # This is a checkpoint-shape guard, not a general Rust parser. Ignore the
    # existing line comments describing the future activation step.
    schedule_code = "\n".join(line.split("//", 1)[0] for line in schedule.splitlines())
    if (schedule_code.count(expected) != 1
            or schedule_code.count("schedule.add_systems") != 1
            or "lessons::move_to_food" in schedule_code):
        raise ValueError("This edition requires the maintenance-only schedule")


def starter_readme() -> bytes:
    return f"""# Moss starter — {EDITION}

Start with [the web book]({BOOK_URL}reference/start-here.html), then open this
folder's Cargo.toml in RustRover or your editor. This is a working-tree snapshot
based on repository checkpoint `{REFERENCE_CHECKPOINT}`, with local source
changes included. It is **not** a clean archive of that Git commit. The exact
contents and per-file hashes are recorded in STARTER-MANIFEST.json.

Maintenance runs. Movement is deliberately unfinished and unscheduled. Animals
remain alive at zero energy. Your first edit is the body of `move_one_cell` in
`crates/moss-sim/src/lessons.rs`. No later biological rules have been installed.

## Prepare the tools

These commands use a Unix shell on macOS or Linux. Install Rust through the
[official rustup instructions](https://rust-lang.github.io/rustup/installation/index.html)
if `rustup` is not available. macOS also needs a working Apple compiler/SDK;
Linux needs its normal native compiler/linker toolchain. Native Windows setup
has not been verified; the supplied `.sh` wrappers require a Unix-like shell.

From this extracted folder:

```sh
rustup toolchain install 1.93.1 --component rustfmt --component clippy --target wasm32-unknown-unknown
cargo +1.93.1 install trunk --version 0.21.14 --locked
cargo +1.93.1 install wasm-bindgen-cli --version 0.2.123 --locked
scripts/with-toolchain.sh cargo fetch --locked
```

The first install/build may take a while. Python 3 is needed only for the
isolated worked-reference runner. Node is needed only for the JavaScript syntax
check in `scripts/check.sh`; it is not part of the application runtime.

## Establish the green baseline

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test bootstrap --test maintenance
scripts/with-toolchain.sh trunk serve --locked --release
```

The focused baseline contains five tests. When Trunk reports readiness, open
http://127.0.0.1:8080/ . Select Fern, press Step three times and inspect reserve:
the authored baseline goes from 60 to 57 while position stays unchanged. Reset
returns the starting state. These are observation instructions, not evidence
that a browser was tested on your machine. Stop the server with Ctrl-C.

## Make one affordable step

Before editing, this exact test intentionally fails at `todo!()`:

```sh
scripts/with-toolchain.sh cargo test -p moss-sim --locked --test movement movement_charges_only_an_affordable_actual_step -- --exact
```

Use Chapter 1 to replace the helper's body. A passing focused test is the first
review checkpoint. The prepared ECS movement adapter remains unscheduled;
activation and the two ignored integration tests are the next reviewed change.
Running `scripts/check.sh` or all workspace tests also reaches this intentional
failure before the exercise is complete. Do not remove its assertions.

## Run a complete reference separately

The web book's reference commands work here after `cargo fetch --locked`:

```sh
python3 docs/tutorial/authoring/check_path_examples.py --session 01
python3 docs/tutorial/authoring/check_path_examples.py
python3 docs/tutorial/authoring/check_path_examples.py --movement
```

The runner uses a separate temporary workspace and target directory, with
Cargo offline. It never replaces this starter's helper or live schedule.
Reference Markdown under `docs/tutorial/` is **extraction-only**, containing the
exact Rust fences the runner needs; it is not the full guide. Read the prose in
the web book. Some original source comments mention repository documents not
included in this deliberately small bundle.

The bundle excludes Git history, collaboration notes, editorial records, local
IDE settings and credentials. No dependency cache or compiled output is bundled.
Project development continues at https://github.com/NicholasHazen/moss .
""".encode("utf-8")


def build_archive(root: Path) -> tuple[bytes, dict]:
    root = root.resolve()
    files: dict[str, bytes] = {}
    origins: dict[str, dict] = {}
    for relative in source_paths(root):
        data = read_source(root, relative)
        files[relative] = data
        origins[relative] = {"kind": "verbatim", "source": relative, "source_sha256": digest(data)}
    validate_exercise(files)
    for relative, marker in reference_paths(root):
        data = read_source(root, relative)
        files[relative] = extract_reference(data, marker, relative)
        origins[relative] = {"kind": "extracted-reference", "source": relative, "source_sha256": digest(data)}
    files["README.md"] = starter_readme()
    origins["README.md"] = {"kind": "generated"}
    inventory = [{
        "path": relative, "sha256": digest(data), "bytes": len(data),
        "mode": "0755" if relative in EXECUTABLES else "0644", **origins[relative],
    } for relative, data in sorted(files.items())]
    manifest = {
        "format": 1, "edition": EDITION, "archive_root": NAME,
        "repository": "https://github.com/NicholasHazen/moss",
        "reference_checkpoint": REFERENCE_CHECKPOINT,
        "source_state": "working-tree snapshot including local modifications; not a clean commit archive",
        "live_behavior": "maintenance only; move_one_cell unfinished and movement unscheduled",
        "tools": {"rust": "1.93.1", "bevy_ecs": "0.18.1", "trunk": "0.21.14", "wasm_bindgen": "0.2.123"},
        "files": inventory,
        "manifest_note": "The manifest lists every other archive file; its own hash would be circular.",
    }
    files["STARTER-MANIFEST.json"] = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_STORED) as archive:
        for relative, data in sorted(files.items()):
            item = zipfile.ZipInfo(f"{NAME}/{relative}", date_time=ZIP_TIMESTAMP)
            item.create_system = 3
            mode = 0o755 if relative in EXECUTABLES else 0o644
            item.external_attr = (stat.S_IFREG | mode) << 16
            archive.writestr(item, data)
    return buffer.getvalue(), manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--copy-to-site", action="store_true",
                        help="also write this validated archive under book/_site/downloads")
    args = parser.parse_args(argv)
    data, manifest = build_archive(ROOT)
    checksum = (digest(data) + "  " + NAME + ".zip\n").encode()
    destinations = [ROOT / "book/artifacts"]
    if args.copy_to_site:
        destinations.append(ROOT / "book/_site/downloads")
    for directory in destinations:
        directory.mkdir(parents=True, exist_ok=True)
        (directory / (NAME + ".zip")).write_bytes(data)
        (directory / (NAME + ".zip.sha256")).write_bytes(checksum)
    print(f"Starter: {len(manifest['files']) + 1} files, {len(data)} bytes; SHA-256 {digest(data)}")
    print(f"Wrote book/artifacts/{NAME}.zip" + (" and the site download copy" if args.copy_to_site else ""))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
