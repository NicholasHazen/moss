#!/usr/bin/env python3
"""Check and package a portable reading edition, including current optional media."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def portable_guide(course):
    return f"""# {course['edition']}

This portable reading edition contains {len(course['modules'])} lessons and
{len(course['guides'])} guide pages, exact Rust references, browser teaching
models, and any current narration and compiled Rust previews included by its
builder. Reading needs no account or package installation.

Extract this ZIP, open a terminal in the extracted directory, and run:

    python3 serve.py

Open http://127.0.0.1:8091 in your browser. Python 3.11+ is sufficient. Keep the
terminal running; Control-C stops the server. If that port is occupied, use
`python3 serve.py --port 8092` and open the printed address. Do not open index.html
directly: browser fetches for search, narration cues, and WASM need the local server.

The course is readable offline. Optional publisher videos and external reference
links need a connection. Narration is generated speech, not a human performance.

The course supports five cumulative Rust browser modes: shared meadow,
population, mobile foraging, committed rest, and hunting. Included previews play
compiled Rust artifacts; browser models and preview controls do not compile your
edits. The refuge capstone is a native Cargo investigation, with no sixth browser
mode. Static explanations and worked answers remain available without a preview.

The downloadable Cargo checkpoint ZIPs contain complete standalone source
projects, manifests, lockfiles, tests and examples. You can inspect them or build
them with the pinned Rust toolchain and Cargo dependencies; the toolchain and
third-party dependencies are not bundled in this reading ZIP. Building offline
requires those dependencies to have been fetched already.

For the continuing learner project, course acceptance checks and rebuilding a
preview from your own edits, use the Moss source checkout and its pinned course
runners. Follow Setup there, then keep editing the same saved project through the
cumulative guides. This portable reading ZIP does not include those runner
scripts. A complete reference checkpoint is an answer to inspect beside your
work, not an instruction to replace your project or discard your investigations.

Notes, highlights, and progress stay in this browser's local origin. Keep the same
address and port to retain them. Copy/export your reading record before changing
browser, address, or port; paste/import it at the new location. Progress is self
reported and does not certify that tests passed.

`manifest.json` lists the edition, course page counts, and SHA-256 and size of each
other packaged file. No public deployment or change to the learner-owned live
Moss assignment is included.
"""


def package_folder(edition):
    return re.sub(r"[^a-z0-9.]+", "-", edition.lower()).strip("-")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    subprocess.run([sys.executable, str(ROOT / "scripts/check.py")], check=True)
    course = json.loads((ROOT / "course.json").read_text())
    folder = package_folder(course["edition"])
    files = {"README.md": portable_guide(course).encode()}
    server = (ROOT / "scripts/serve.py").read_text()
    server = server.replace('Path(__file__).resolve().parents[1] / "_site"',
                            'Path(__file__).resolve().parent / "site"')
    files["serve.py"] = server.encode()
    for path in sorted((ROOT / "_site").rglob("*")):
        if path.is_file():
            files["site/" + path.relative_to(ROOT / "_site").as_posix()] = path.read_bytes()
    files["manifest.json"] = (json.dumps({
        "edition": course["edition"],
        "lessonCount": len(course["modules"]),
        "guideCount": len(course["guides"]),
        "pageCount": len(course["modules"]) + len(course["guides"]),
        "files": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                  for name, data in files.items()},
    }, indent=2) + "\n").encode()
    destination = args.destination.resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=destination.parent, suffix=".zip", delete=False) as handle:
            temporary = Path(handle.name)
        with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for name, data in files.items():
                archive.writestr(folder + "/" + name, data)
        with zipfile.ZipFile(temporary) as archive:
            if archive.testzip() is not None:
                raise ValueError("Archive integrity check failed")
        os.replace(temporary, destination)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    print(f"Packaged {len(files)} files: {destination} ({destination.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
