#!/usr/bin/env python3
"""Place a verified Fieldnotes export at the site root, retaining old links."""
import argparse
import hashlib
import html
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import tempfile

LEGACY = {
    "chapters/01-one-affordable-step.html": "14-movement.html",
    "chapters/02-a-meal-has-two-sides.html": "15-meals.html",
    "chapters/03-let-the-animal-choose.html": "16-targets.html",
    "chapters/04-two-hares-one-fair-comparison.html": "17-variation.html",
    "chapters/05-a-world-that-feeds.html": "18-growth.html",
    "chapters/06-a-disappearance-explained.html": "19-history.html",
    "chapters/07-another-kind-of-life.html": "20-predation.html",
    "reference/start-here.html": "setup.html",
    "reference/code-map.html": "introduction.html#what-bevy-ecs-does",
    "reference/concept-index.html": "roadmap.html",
    "reference/reading-shelf.html": "shelf.html",
    "reference/edition-notes.html": "about.html",
    "toc.html": "roadmap.html",
    "print.html": "introduction.html",
    'source/crates/moss-sim/src/components.rs.html': 'setup.html',
    'source/crates/moss-sim/src/config.rs.html': 'setup.html',
    'source/crates/moss-sim/src/energy.rs.html': 'setup.html',
    'source/crates/moss-sim/src/fixture.rs.html': 'setup.html',
    'source/crates/moss-sim/src/journal.rs.html': 'setup.html',
    'source/crates/moss-sim/src/lessons.rs.html': 'setup.html',
    'source/crates/moss-sim/src/lib.rs.html': 'setup.html',
    'source/crates/moss-sim/src/simulation.rs.html': 'setup.html',
    'source/crates/moss-web/src/browser.rs.html': 'setup.html',
    'source/crates/moss-web/src/browser/bridge.rs.html': 'setup.html',
    'source/crates/moss-web/src/browser/frame.rs.html': 'setup.html',
    'source/crates/moss-web/src/browser/scene.rs.html': 'setup.html',
    'source/crates/moss-web/src/browser/snapshot.rs.html': 'setup.html',
    'source/crates/moss-web/src/playback.rs.html': 'setup.html',
    'source/docs/design/ecology.md.html': 'introduction.html',
    'source/docs/design/observability.md.html': '13-evidence.html',
    'source/docs/research/attributes-and-energy.md.html': '04-units.html',
    'source/docs/research/property-bags-and-defaults.md.html': '09-traits.html',
    'source/docs/tutorial/03-food-choice.md.html': '16-targets.html',
    'source/docs/tutorial/05-eating.md.html': '15-meals.html',
    'source/docs/tutorial/context/a-tick-through-moss.md.html': '01-observe.html',
    'source/docs/tutorial/context/attributes-and-defaults.md.html': '17-variation.html',
    'source/docs/tutorial/path/01-bounded-meal.md.html': '15-meals.html',
    'source/docs/tutorial/path/02-a-shared-meal.md.html': '15-meals.html',
    'source/docs/tutorial/path/03-finding-food.md.html': '16-targets.html',
    'source/docs/tutorial/path/04-when-to-seek.md.html': '16-targets.html',
    'source/docs/tutorial/path/05-many-individuals.md.html': '17-variation.html',
    'source/docs/tutorial/path/06-owned-costs.md.html': '17-variation.html',
    'source/docs/tutorial/path/07-querying-costs.md.html': '17-variation.html',
    'source/docs/tutorial/path/08-a-fair-comparison.md.html': '17-variation.html',
    'source/docs/tutorial/path/09-growing-grass.md.html': '18-growth.html',
    'source/docs/tutorial/path/10-growth-meets-meals.md.html': '18-growth.html',
    'source/docs/tutorial/path/11-a-day-from-ticks.md.html': '01-observe.html',
    'source/docs/tutorial/path/12-light-controls-growth.md.html': '18-growth.html',
    'source/docs/tutorial/path/verification.md.html': 'about.html',
}


def redirect(target, preserve_fragment=True):
    suffix = " + location.search + location.hash" if preserve_fragment else ""
    return (f'<!doctype html><html lang="en"><head><meta charset="utf-8">'
            f'<meta name="viewport" content="width=device-width, initial-scale=1">'
            f'<meta http-equiv="refresh" content="0;url={html.escape(target, quote=True)}">'
            '<title>Moss Fieldnotes has moved</title></head><body>'
            f'<p>Continue to <a href="{html.escape(target, quote=True)}">Moss Fieldnotes</a>.</p>'
            f'<script>location.replace({json.dumps(target)}{suffix});</script></body></html>').encode()


def assemble(export, destination, base_path="/moss/"):
    if not re.fullmatch(r"/(?:[A-Za-z0-9_-]+/)*", base_path):
        raise ValueError("Base path must be a same-site absolute directory path")
    export, destination = export.resolve(), destination.resolve()
    if destination == export or destination.is_relative_to(export):
        raise ValueError("Destination must be outside the verified export")
    if destination.exists() and (not destination.is_dir() or any(destination.iterdir())):
        raise ValueError("Destination must be new or empty")
    manifest = json.loads((export / "manifest.json").read_text())
    if manifest.get("publicEdition") is not True:
        raise ValueError("Only reviewed public exports can be assembled")
    files = {}
    for name, evidence in manifest["files"].items():
        path = PurePosixPath(name)
        if path.is_absolute() or ".." in path.parts or str(path) != name or "\\" in name:
            raise ValueError(f"Unsafe export path: {name}")
        if path.parts[0] == "fieldnotes" or name in {"404.html", ".nojekyll", "site-manifest.json"}:
            raise ValueError(f"Reserved site output path: {name}")
        data = (export / name).read_bytes()
        if len(data) != evidence["bytes"] or hashlib.sha256(data).hexdigest() != evidence["sha256"]:
            raise ValueError(f"Changed export file: {name}")
        files[name] = data
    actual = {p.relative_to(export).as_posix() for p in export.rglob("*") if p.is_file()}
    if actual != set(files) | {"manifest.json"}:
        raise ValueError("Unreviewed files in export")
    files["manifest.json"] = (export / "manifest.json").read_bytes()
    if not {"index.html", "introduction.html", "setup.html"} <= files.keys():
        raise ValueError("Missing book entry pages")
    # HTML has one canonical home. Old asset/download URLs keep serving the same
    # reviewed bytes so existing bookmarks and cached source links still work.
    for name, data in list(files.items()):
        if name == "manifest.json":
            continue
        files["fieldnotes/" + name] = redirect(base_path + name) if name.endswith(".html") else data
    for old, new in LEGACY.items():
        if new.split("#")[0] not in files:
            raise ValueError(f"Missing redirect target: {new}")
        if old in files:
            raise ValueError(f"Legacy redirect would overwrite course content: {old}")
        files[old] = redirect(base_path + new, preserve_fragment=False)
    files[".nojekyll"] = b""
    files["404.html"] = (f'<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        '<title>Find your way back to Moss</title></head><body><main>'
        '<h1>Find your way back to Moss</h1><p>Fieldnotes is now the project’s book.</p>'
        f'<p><a href="{base_path}introduction.html">Meet Moss</a> or '
        f'<a href="{base_path}roadmap.html">find a chapter in the learning path</a>.</p>'
        '</main></body></html>').encode()
    site_manifest = {"edition": manifest["edition"], "basePath": base_path,
        "files": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                  for name, data in sorted(files.items())}}
    files["site-manifest.json"] = (json.dumps(site_manifest, indent=2) + "\n").encode()
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".moss-pages-", dir=destination.parent) as temporary:
        staging = Path(temporary) / "site"
        staging.mkdir()
        for name, data in files.items():
            target = staging / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        if destination.exists():
            destination.rmdir()
        shutil.move(staging, destination)
    return site_manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("export", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--base-path", default="/moss/")
    args = parser.parse_args()
    manifest = assemble(args.export, args.destination, args.base_path)
    print(f"Assembled {len(manifest['files'])} verified files plus site manifest. No upload performed.")


if __name__ == "__main__":
    main()
