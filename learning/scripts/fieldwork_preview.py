#!/usr/bin/env python3
"""Check a persistent course project, then compile those exact sources for the browser."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

import fieldwork
from labs import ROOT, WORK, TOOLCHAIN

STAGES = {"ecosystem": "shared-meadow", "population": "population", "mobile": "mobile", "resting": "resting", "hunting": "hunting"}


def inputs(project, mode="ecosystem"):
    if mode not in STAGES:
        raise ValueError(f"Unknown preview mode: {mode}")
    files = fieldwork.source_files(project)
    manifest = fieldwork.validate_manifest(files)
    name = manifest["package"]["name"]
    if not re.fullmatch(r"[a-zA-Z0-9_-]+", name):
        raise ValueError("Course package name must be a Cargo identifier")
    host = (ROOT / f"preview/{mode}-host.rs").read_bytes()
    host_manifest = ('[package]\nname = "fieldnotes_ecosystem_host"\nversion = "0.1.0"\n'
                     'edition = "2024"\npublish = false\n[workspace]\nexclude = ["course"]\n'
                     '[lib]\ncrate-type = ["cdylib", "rlib"]\n'
                     f'[dependencies]\ncourse = {{ package = "{name}", path = "course" }}\n')
    canonical = fieldwork.acceptance_suites(STAGES[mode])
    source_hash = fieldwork.fingerprint(files)
    bundle = {"course/" + path: data for path, data in files.items()}
    bundle.update({"host.rs": host, "host.toml": host_manifest.encode(), "toolchain": TOOLCHAIN.encode()})
    bundle.update({f"{suite}.rs": source for suite, source in canonical.items()})
    return files, host, host_manifest, name, source_hash, fieldwork.fingerprint(bundle)


def current(metadata, mode="ecosystem"):
    try:
        if not isinstance(metadata, dict) or metadata.get("api") != 1:
            return False
        if metadata.get("mode", "ecosystem") != mode:
            return False
        kind = metadata.get("sourceKind")
        if kind not in {"reference", "learner"}:
            return False
        project = metadata.get("sourcePath") if kind == "learner" else fieldwork.REFERENCE
        if not project:
            return False
        values = inputs(project, mode) if mode != "ecosystem" else inputs(project)
        return metadata.get("sourceHash") == values[-2] and metadata.get("buildHash") == values[-1]
    except (OSError, ValueError, KeyError, TypeError):
        return False


def copy_current(output, mode="ecosystem"):
    if mode not in STAGES:
        raise ValueError(f"Unknown preview mode: {mode}")
    destination = output / "previews" / mode
    if destination.exists():
        shutil.rmtree(destination)
    source = WORK / "previews" / mode
    try:
        metadata = json.loads((source / "build.json").read_text())
        if not (current(metadata, mode) if mode != "ecosystem" else current(metadata)):
            return
        filename = f"{mode}-{metadata['buildHash']}.wasm"
        if metadata.get("wasm") != filename:
            return
        binary = (source / filename).read_bytes()
        if hashlib.sha256(binary).hexdigest() != metadata.get("wasmHash"):
            return
        destination.mkdir(parents=True)
        (destination / filename).write_bytes(binary)
        (destination / "build.json").write_text(json.dumps(metadata) + "\n")
    except (OSError, ValueError, KeyError, TypeError):
        return


def write_atomic(path, data):
    with tempfile.NamedTemporaryFile(dir=path.parent, prefix="pending-", delete=False) as pending:
        pending.write(data)
        temporary = Path(pending.name)
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def build(project=None, mode="ecosystem"):
    learner = project is not None
    project = Path(project).resolve() if learner else fieldwork.REFERENCE
    read_inputs = lambda: inputs(project, mode) if mode != "ecosystem" else inputs(project)
    files, host, manifest, name, source_hash, build_hash = read_inputs()
    label = f"LEARNER PROJECT {project}" if learner else "COURSE REFERENCE"
    report = (fieldwork.check(project, label, STAGES[mode]) if mode != "ecosystem"
              else fieldwork.check(project, label))
    if report["sourceHash"] != source_hash or read_inputs()[-1] != build_hash:
        raise ValueError("Preview inputs changed during checking; rerun before building")
    with tempfile.TemporaryDirectory(prefix="ecosystem-preview-", dir=WORK) as directory:
        package = Path(directory)
        for relative, data in files.items():
            path = package / "course" / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        (package / "src").mkdir()
        (package / "src/lib.rs").write_bytes(host)
        (package / "Cargo.toml").write_text(manifest)
        # Reuse the selected dependency lock exactly, adding only this local
        # adapter package. --locked rejects any dependency re-resolution.
        lock = files["Cargo.lock"].decode()
        lock += ('\n[[package]]\nname = "fieldnotes_ecosystem_host"\nversion = "0.1.0"\n'
                 f'dependencies = ["{name}"]\n')
        (package / "Cargo.lock").write_text(lock)
        command = ["cargo", f"+{TOOLCHAIN}"]
        # Native adapter artifacts need the same isolation as the final WASM:
        # concurrent modes share the adapter crate name but not its program.
        target = package / "target"
        host_tests = command + ["test", "--offline", "--locked", "--lib", "--color", "never",
                                "--target-dir", str(target)]
        listed = fieldwork.run(host_tests + ["--", "--list"], package)
        required = {line.removesuffix(": test") for line in listed.splitlines() if line.endswith(": test")}
        expected = {"tests::" + name for name in re.findall(r"#\[test\]\s*fn\s+(\w+)\s*\(", host.decode())}
        if not expected or not expected <= required:
            raise ValueError("Native browser adapter checks are missing")
        output = fieldwork.run(host_tests + ["--", "--color", "never"], package)
        passed = set(re.findall(r"^test (\S+)(?: - should panic)? \.\.\. ok$", output, re.MULTILINE))
        if not required <= passed:
            raise ValueError("Native browser adapter checks did not execute successfully")
        # A private final target prevents concurrent builds from substituting a
        # different program's binary between compilation and publication.
        fieldwork.run(command + ["build", "--offline", "--locked", "--release", "--target",
                                 "wasm32-unknown-unknown", "--target-dir", str(target)], package)
        binary = (target / "wasm32-unknown-unknown/release/fieldnotes_ecosystem_host.wasm").read_bytes()
        if read_inputs()[-1] != build_hash:
            raise ValueError("Preview inputs changed during compilation; rerun before publishing")
        destination = WORK / "previews" / mode
        destination.mkdir(parents=True, exist_ok=True)
        filename = f"{mode}-{build_hash}.wasm"
        metadata = {"api": 1, "mode": mode, "sourceKind": "learner" if learner else "reference",
                    "sourcePath": str(project) if learner else None, "sourceHash": source_hash,
                    "buildHash": build_hash, "wasm": filename,
                    "wasmHash": hashlib.sha256(binary).hexdigest(), "rust": TOOLCHAIN,
                    "acceptanceHash": report["acceptanceHash"],
                    "requiredTests": report["requiredTests"]}
        write_atomic(destination / filename, binary)
        write_atomic(destination / "build.json", (json.dumps(metadata, indent=2) + "\n").encode())
    page = "fieldwork.html" if mode == "ecosystem" else f"{mode}.html"
    print(f"Compiled the checked course project. Run build.py, then open {page}.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument("--project", type=Path)
    selection.add_argument("--reference", action="store_true")
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--population", action="store_true", help="require population-stage checks and build its adapter")
    modes.add_argument("--mobile", action="store_true", help="require mobile-stage checks and build its adapter")
    modes.add_argument("--resting", action="store_true", help="require fatigue/rest stage checks and build its adapter")
    modes.add_argument("--hunting", action="store_true", help="require the cumulative hunting contract and build its adapter")
    args = parser.parse_args()
    try:
        build(args.project, "hunting" if args.hunting else "resting" if args.resting else "mobile" if args.mobile else "population" if args.population else "ecosystem")
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"Fieldwork preview stopped: {error}", file=sys.stderr)
        sys.exit(1)
