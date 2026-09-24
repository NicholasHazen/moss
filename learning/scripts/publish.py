#!/usr/bin/env python3
"""Prepare a reviewed static public export; never build, upload, or change local media."""
from __future__ import annotations

import argparse
import hashlib
import html
import io
import json
import os
import posixpath
import re
import tempfile
import zipfile
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath
from urllib.parse import quote, unquote, urlsplit, urlunsplit

import fieldwork_preview
import preview
from narration import read_current
from runtime import runtime_fingerprint

ROOT = Path(__file__).resolve().parents[1]
MODES = ("evidence", "ecosystem", "population", "mobile", "resting", "hunting")
CHECKPOINTS = ("shared-meadow", "population", "mobile", "resting", "hunting", "refuge")
VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}
PRIVATE_PARTS = {".git", ".env", ".codex", ".agents", "work", "_site", "__pycache__"}
SOURCE_DOWNLOAD = "downloads/moss-fieldnotes-source.zip"
ABOUT_TEXT = ('<p class="scope-note">This public edition keeps the browser <strong>Listen</strong> control. '
              'The local edition’s recordings made with macOS system voices are excluded from this public export '
              'because Apple’s voice terms restrict public redistribution. Reading, highlighting, and the compiled '
              'Rust previews remain available. Browser voices depend on your device and may use a network service.</p>')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def safe_name(name):
    path = PurePosixPath(name)
    require(name and not name.startswith("/") and "\\" not in name
            and ".." not in path.parts and ":" not in path.parts[0], f"Unsafe archive path: {name}")
    require(path.as_posix() == name.rstrip("/"), f"Noncanonical archive path: {name}")
    require(not PRIVATE_PARTS.intersection(path.parts), f"Private/work files are not publication inputs: {name}")
    require(not any(path.parts[i:i+2] in {("learning", "design"), ("learning", "STATUS.md")}
                    for i in range(len(path.parts) - 1)), f"Private course notes are not publication inputs: {name}")
    return path


def check_zip(data, label):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        names = archive.namelist()
        require(names and len(names) == len(set(names)), f"Empty/duplicate ZIP entries: {label}")
        require(archive.testzip() is None, f"Corrupt ZIP: {label}")
        for entry in archive.infolist():
            safe_name(entry.filename)
            require((entry.external_attr >> 16) & 0o170000 != 0o120000, f"Symlink in ZIP: {label}")
        return set(names)


class Page(HTMLParser):
    """Record exact source ranges; preserve every untouched byte of the HTML."""
    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.source = source
        self.offsets = [0]
        for line in source.splitlines(keepends=True):
            self.offsets.append(self.offsets[-1] + len(line))
        self.stack, self.nodes, self.refs, self.ids, self.tags = [], [], [], set(), []
        self.players = 0

    def position(self):
        line, column = self.getpos()
        return self.offsets[line - 1] + column

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        self.tags.append({"start": self.position(), "raw": self.get_starttag_text(), "attrs": attrs})
        if "id" in attrs:
            require(attrs["id"] not in self.ids, f"Duplicate HTML id: {attrs['id']}")
            self.ids.add(attrs["id"])
        if tag == "base":
            raise ValueError("Public course pages must use relative links, not a base element")
        self.refs.extend(v for k, v in attributes if k in {"href", "src", "data-narration-source", "data-source-file"} and v)
        if "data-narration-source" in attrs:
            self.players += 1
        if tag not in VOID:
            self.stack.append({"tag": tag, "attrs": attrs, "start": self.position(), "text": []})

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in VOID:
            self.stack.pop()

    def handle_data(self, text):
        for node in self.stack:
            node["text"].append(text)

    def handle_endtag(self, tag):
        if tag in VOID:
            return
        require(self.stack and self.stack[-1]["tag"] == tag, f"Unbalanced HTML closing tag: {tag}")
        node = self.stack.pop()
        node["end"] = self.source.index(">", self.position()) + 1
        node["text"] = "".join(node["text"])
        self.nodes.append(node)


def parsed(source):
    page = Page(source)
    page.feed(source)
    require(not page.stack, "Unclosed HTML elements")
    return page


def public_html(name, data, source_download):
    source = data.decode("utf-8")
    page = parsed(source)
    edits = []
    for node in page.nodes:
        tag, attrs = node["tag"], node["attrs"]
        if (tag == "section" and "narration-player" in attrs.get("class", "").split()
                or tag == "script" and urlsplit(attrs.get("src", "")).path == "assets/narration.js"):
            edits.append((node["start"], node["end"], ""))
    if name == "about.html":
        paragraphs = [n for n in page.nodes if n["tag"] == "p" and
                      n["text"].startswith("Where generated narration is available,")]
        require(len(paragraphs) == 1, "About's generated-recording explanation changed; review public wording")
        node = paragraphs[0]
        edits.append((node["start"], node["end"], ABOUT_TEXT))
    if name == "setup.html":
        headings = [n for n in page.nodes if n["tag"] == "h1"]
        require(len(headings) == 1, "Setup must have one title")
        if source_download:
            notice = ('<p class="scope-note"><strong>Public edition.</strong> Read and try the previews here. '
                      '<a href="downloads/moss-fieldnotes-source.zip" download>Download the complete course source</a> '
                      'to edit Rust and run the checks below. Extract it, follow its README, and keep one saved learner project. '
                      'Its <code>learning/published-preview</code> directory contains the worked source used for these public previews; '
                      'compile your own changes locally. '
                      'The pinned Rust toolchain and Cargo dependencies are installed separately.</p>')
        else:
            notice = ('<p class="scope-note"><strong>Public edition.</strong> Read and try the previews here. '
                      'The local commands below require the Moss source checkout containing this course; '
                      'a complete source download is not included in this export.</p>')
        edits.append((headings[0]["end"], headings[0]["end"], notice))
    previous = len(source) + 1
    for start, end, replacement in sorted(edits, reverse=True):
        require(end <= previous, "Overlapping publication HTML edits")
        source = source[:start] + replacement + source[end:]
        previous = start
    return source.encode("utf-8")


def rename_public_gitignore(files):
    """Pages' upload excludes dotfiles; only exported names and URL attributes change."""
    names = {".gitignore": "gitignore.txt", ".gitignore.html": "gitignore.txt.html"}
    renamed = {name: str(PurePosixPath(name).with_name(names[PurePosixPath(name).name]))
               for name in files if PurePosixPath(name).name in names}
    require(not set(renamed.values()).intersection(files), "Public gitignore export name collision")
    attributes = re.compile(r'''(?P<name>[^\s"'<>/=]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+))?''')
    result = {}
    for name, data in files.items():
        output_name = renamed.get(name, name)
        if name.endswith(".html"):
            source = data.decode("utf-8")
            edits = []
            for tag in parsed(source).tags:
                for key in ("href", "src", "data-source-file"):
                    value = tag["attrs"].get(key)
                    if not value:
                        continue
                    url = urlsplit(value)
                    if url.scheme or url.netloc or not url.path:
                        continue
                    target = posixpath.normpath(posixpath.join(posixpath.dirname(name), unquote(url.path)))
                    if target not in renamed:
                        continue
                    relative = posixpath.relpath(renamed[target], posixpath.dirname(output_name) or ".")
                    replacement = urlunsplit(("", "", quote(relative, safe="/"), url.query, url.fragment))
                    token = next(match for match in attributes.finditer(tag["raw"])
                                 if match.group("name").lower() == key)
                    edits.append((tag["start"] + token.start(), tag["start"] + token.end(),
                                  f'{key}="{html.escape(replacement, quote=True)}"'))
            for start, end, replacement in sorted(edits, reverse=True):
                source = source[:start] + replacement + source[end:]
            data = source.encode("utf-8")
        result[output_name] = data
    return result


def validate_export(files, course, public=False):
    expected_pages = {m["id"] + ".html" for m in course["modules"]} | {n + ".html" for n in course["guides"]}
    require(len(expected_pages) == len(course["modules"]) + len(course["guides"]), "Duplicate course page IDs")
    require({n for n in files if "/" not in n and n.endswith(".html")} == expected_pages, "Missing/extra course pages")
    allowed = {"assets", "examples", "reference", "downloads", "runtime", "previews"} | ({"narration"} if not public else set())
    for name in files:
        path = safe_name(name)
        if public:
            require(not any(part.startswith(".") for part in path.parts), f"Pages upload would exclude dotfile: {name}")
        require(name in expected_pages or len(path.parts) > 1 and path.parts[0] in allowed, f"Unknown/private publication file: {name}")
    wasm = {n for n in files if n.endswith(".wasm")}
    require(len(wasm) == 7 and "runtime/moss.wasm" in wasm, "Expected all seven compiled WASM assets")
    for name in wasm:
        require(files[name].startswith(b"\0asm\x01\0\0\0"), f"Invalid WASM header: {name}")
    for mode in MODES:
        metadata = json.loads(files[f"previews/{mode}/build.json"])
        require(isinstance(metadata, dict) and metadata.get("api") == 1, f"Invalid preview metadata: {mode}")
        filename = metadata.get("wasm", "")
        require(filename and PurePosixPath(filename).name == filename and "\\" not in filename, f"Unsafe WASM name: {mode}")
        require(f"previews/{mode}/{filename}" in wasm, f"Missing preview binary: {mode}")
        if mode != "evidence":
            require(digest(files[f"previews/{mode}/{filename}"]) == metadata.get("wasmHash"), f"WASM hash mismatch: {mode}")
        if public:
            require(metadata.get("sourcePath") in {None, "verified-course-project"}, f"Private source path remains: {mode}")
    expected_downloads = {f"downloads/{n}-reference.zip" for n in CHECKPOINTS}
    actual = {n for n in files if n.startswith("downloads/") and n.endswith(".zip")}
    require(actual - {SOURCE_DOWNLOAD} == expected_downloads, "Expected six complete checkpoint downloads")
    for checkpoint in CHECKPOINTS:
        names = check_zip(files[f"downloads/{checkpoint}-reference.zip"], checkpoint)
        for required in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/lib.rs", "tests/acceptance.rs", "README.md"):
            require(f"{checkpoint}/{required}" in names, f"Incomplete checkpoint: {checkpoint}/{required}")
    pages = {name: parsed(data.decode("utf-8")) for name, data in files.items() if name.endswith(".html")}
    for name, page in pages.items():
        if public:
            require(page.players == 0, f"Narration player remains: {name}")
        for ref in page.refs:
            url = urlsplit(ref)
            if url.scheme or url.netloc:
                continue
            path = unquote(url.path)
            require(not path.startswith("/") and "\\" not in path, f"Nonportable URL in {name}: {ref}")
            target = posixpath.normpath(posixpath.join(posixpath.dirname(name), path)) if path else name
            require(not target.startswith("../") and target in files, f"Missing/escaping local URL in {name}: {ref}")
            if public:
                require(not target.startswith("narration/") and target != "assets/narration.js", f"Removed narration reference in {name}")
            if url.fragment and target in pages:
                require(unquote(url.fragment) in pages[target].ids, f"Missing anchor in {name}: {ref}")


def read_site(site):
    require(site.is_dir() and not site.is_symlink(), "Build the complete local site before exporting")
    files = {}
    for path in sorted(site.rglob("*")):
        require(not path.is_symlink(), f"Site symlinks are not publication inputs: {path.name}")
        if path.is_file():
            files[path.relative_to(site).as_posix()] = path.read_bytes()
    return files


def validate_built_bytes(files):
    # These two older hosts lack a separate wasmHash field. Bind their export
    # to the exact local build pair instead of accepting any WASM header.
    for directory in ("runtime", "previews/evidence"):
        metadata_name = directory + "/build.json"
        metadata = json.loads(files[metadata_name])
        built_metadata = json.loads((ROOT / "work" / metadata_name).read_bytes())
        require(metadata == built_metadata, f"Export differs from local build metadata: {directory}")
        binary_name = directory + "/" + ("moss.wasm" if directory == "runtime" else metadata["wasm"])
        require(files[binary_name] == (ROOT / "work" / binary_name).read_bytes(),
                f"Export differs from local built WASM: {directory}")


def validate_current(files, course):
    expected_assets = {"assets/" + p.relative_to(ROOT / "assets").as_posix(): p.read_bytes()
                       for p in (ROOT / "assets").rglob("*") if p.is_file()}
    require({n: data for n, data in files.items() if n.startswith("assets/") and n != "assets/search.json"} == expected_assets,
            "Generated assets differ from the current authored assets; rebuild and review")
    for mode in MODES:
        metadata = json.loads(files[f"previews/{mode}/build.json"])
        current = preview.current(metadata) if mode == "evidence" else fieldwork_preview.current(metadata, mode)
        require(current, f"Preview is not source-current: {mode}")
    runtime = json.loads(files["runtime/build.json"])
    require(runtime.get("api") == 1 and runtime.get("sourceHash") == runtime_fingerprint(), "Maintenance runtime is not source-current")
    validate_built_bytes(files)
    for checkpoint in CHECKPOINTS:
        root = ROOT / "checkpoints" / checkpoint
        expected = {checkpoint + "/" + path.relative_to(root).as_posix(): path.read_bytes()
                    for path in root.rglob("*") if path.is_file()
                    and not {"target", ".git", ".DS_Store"}.intersection(path.relative_to(root).parts)}
        with zipfile.ZipFile(io.BytesIO(files[f"downloads/{checkpoint}-reference.zip"])) as archive:
            require({n: archive.read(n) for n in archive.namelist()} == expected,
                    f"Checkpoint download differs from frozen source: {checkpoint}")
    titles = {m["id"]: m["title"] for m in course["modules"]} | course["guides"]
    for name in files:
        if name.startswith("narration/") and name.endswith("/cues.json"):
            page = name.split("/")[1]
            require(page in titles, f"Unknown narration page: {page}")
            snapshot = read_current(ROOT / "_site/narration" / page, titles[page], (ROOT / "content" / (page + ".html")).read_text())
            require(snapshot is not None, f"Narration is not source-current: {page}")
            require(snapshot[0].get("provider", "macos") == "macos", "Review publication rights before exporting another narration provider")


def publish(destination, source_archive=None):
    destination = Path(destination)
    require(not destination.is_symlink(), "Destination must not be a symlink")
    destination = destination.resolve()
    site = (ROOT / "_site").resolve()
    require(not destination.is_relative_to(site), "Destination must be outside the local site")
    require(not destination.exists() or destination.is_dir() and not any(destination.iterdir()), "Destination must be new or empty")
    course_bytes = (ROOT / "course.json").read_bytes()
    course = json.loads(course_bytes)
    original = read_site(site)
    validate_export(original, course)
    validate_current(original, course)
    source_bytes = Path(source_archive).read_bytes() if source_archive else None
    if source_bytes is not None:
        check_zip(source_bytes, "course source")
    files = {name: data for name, data in original.items() if not name.startswith("narration/") and name != "assets/narration.js"}
    for name, data in list(files.items()):
        if name.endswith(".html"):
            files[name] = public_html(name, data, source_bytes is not None)
        elif name.startswith("previews/") and name.endswith("/build.json"):
            metadata = json.loads(data)
            if metadata.get("sourcePath") is not None:
                metadata["sourcePath"] = "verified-course-project"
            files[name] = (json.dumps(metadata, indent=2) + "\n").encode()
    if source_bytes is not None:
        files[SOURCE_DOWNLOAD] = source_bytes
    files = rename_public_gitignore(files)
    validate_export(files, course, public=True)
    manifest = {"edition": course["edition"], "publicEdition": True,
                "recordings": "Excluded macOS system-voice recordings; browser Listen remains available",
                "pageCount": len(course["modules"]) + len(course["guides"]), "wasmCount": 7, "checkpointCount": 6,
                "files": {n: {"bytes": len(data), "sha256": digest(data)} for n, data in sorted(files.items())}}
    files["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".fieldnotes-public-", dir=destination.parent) as temporary:
        staging = Path(temporary) / "site"
        staging.mkdir()
        for name, data in files.items():
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        require(read_site(site) == original and (ROOT / "course.json").read_bytes() == course_bytes,
                "Local site changed during export; review it and retry")
        # All validation precedes replacement; a failed check leaves the requested destination untouched.
        if destination.exists():
            require(not any(destination.iterdir()), "Destination changed during export")
            destination.rmdir()
        os.replace(staging, destination)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", required=True, type=Path, help="new or empty public export directory")
    parser.add_argument("--source-archive", type=Path, help="reviewed source ZIP to include unchanged")
    args = parser.parse_args()
    try:
        manifest = publish(args.destination, args.source_archive)
    except (OSError, ValueError, KeyError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Public export refused: {error}\n")
    print(f"Prepared {len(manifest['files'])} manifested files in {args.destination}; {manifest['pageCount']} pages, 7 WASM, 6 checkpoints. No upload performed.")


if __name__ == "__main__":
    main()
