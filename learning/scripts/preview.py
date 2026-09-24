#!/usr/bin/env python3
"""Check the evidence lab, then compile that exact program for its browser panel.

This is a local Rust build, with normal local process permissions. The optional
--file selects the learner's own code; it is not sent to a compiler service.
"""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from labs import ROOT, WORK, TOOLCHAIN, examples, checked_source, check, run


def inputs(learner_path=None):
    reference = examples()[("data-runnable", "evidence")]
    learner = Path(learner_path).read_text() if learner_path else None
    source = checked_source(reference, learner)
    host = (ROOT / "preview/evidence-host.rs").read_text()
    manifest = (ROOT / "scripts/lab.Cargo.toml").read_text() + '\n[lib]\ncrate-type = ["cdylib"]\n'
    lock = (ROOT / "scripts/lab.Cargo.lock").read_bytes()
    digest = hashlib.sha256()
    for value in (TOOLCHAIN.encode(), source.encode(), host.encode(), manifest.encode(), lock):
        digest.update(len(value).to_bytes(8, "big") + value)
    return source, reference, host, manifest, lock, digest.hexdigest()


def current(metadata):
    """A learner preview is current only while the checked source is unchanged."""
    try:
        if not isinstance(metadata, dict) or metadata.get("api") != 1:
            return False
        kind = metadata.get("sourceKind")
        if kind not in {"reference", "learner"}:
            return False
        learner = metadata.get("sourcePath") if kind == "learner" else None
        if kind == "learner" and (not isinstance(learner, str) or not learner):
            return False
        return metadata.get("sourceHash") == inputs(learner)[-1]
    except (OSError, ValueError, KeyError):
        return False


def copy_current(output):
    target = output / "previews/evidence"
    if target.exists():
        shutil.rmtree(target)
    directory = WORK / "previews/evidence"
    try:
        metadata = json.loads((directory / "build.json").read_text())
        if not current(metadata):
            return
        # Never turn an arbitrary metadata filename into a filesystem path.
        filename = f"evidence-{metadata['sourceHash']}.wasm"
        if metadata.get("wasm") != filename or not (directory / filename).is_file():
            return
        target.mkdir(parents=True)
        shutil.copyfile(directory / filename, target / filename)
        (target / "build.json").write_text(json.dumps(metadata) + "\n")
    except (OSError, ValueError, KeyError):
        return


def build(learner_path=None):
    if learner_path:
        learner_path = learner_path.resolve()
    source, reference, host, manifest, lock, source_hash = inputs(learner_path)
    print(f"Checking {'LEARNER FILE ' + str(learner_path) if learner_path else 'PRINTED REFERENCE'}: evidence", flush=True)
    check("evidence", source, reference)
    WORK.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="evidence-preview-", dir=WORK) as directory:
        package = Path(directory)
        (package / "src").mkdir()
        (package / "src/lib.rs").write_text('#![allow(dead_code)]\n' + source + "\n" + host)
        (package / "Cargo.toml").write_text(manifest)
        (package / "Cargo.lock").write_bytes(lock)
        # This crate name is shared with other labs. Keep the final artifact in
        # this invocation's own target directory, so another build cannot swap
        # in a different learner implementation between compilation and copy.
        target = package / "target"
        run(["cargo", f"+{TOOLCHAIN}", "build", "--offline", "--locked", "--release", "--target", "wasm32-unknown-unknown", "--target-dir", str(target)], package)
        if inputs(learner_path)[-1] != source_hash:
            raise ValueError("Preview inputs changed during checking or compilation; rebuild before publishing.")
        output = WORK / "previews/evidence"
        output.mkdir(parents=True, exist_ok=True)
        filename = f"evidence-{source_hash}.wasm"
        with tempfile.NamedTemporaryFile(dir=output, prefix="binary-", delete=False) as pending_binary:
            binary_path = Path(pending_binary.name)
        try:
            shutil.copyfile(target / "wasm32-unknown-unknown/release/moss_fieldnote_lab.wasm", binary_path)
            os.replace(binary_path, output / filename)
        finally:
            binary_path.unlink(missing_ok=True)
        metadata = {"api": 1, "sourceKind": "learner" if learner_path else "reference",
                    "sourcePath": str(learner_path) if learner_path else None,
                    "sourceHash": source_hash, "wasm": filename,
                    "rust": TOOLCHAIN, "bevyEcs": "0.18.1"}
        with tempfile.NamedTemporaryFile(mode="w", dir=output, prefix="metadata-", delete=False) as pending:
            pending.write(json.dumps(metadata) + "\n")
            pending_path = Path(pending.name)
        try:
            os.replace(pending_path, output / "build.json")
        finally:
            pending_path.unlink(missing_ok=True)
    print("Preview compiled from the checked source. Run build.py, then reload lesson 13.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("name", choices=["evidence"])
    parser.add_argument("--file", type=Path)
    args = parser.parse_args()
    build(args.file)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Preview stopped: {error}", file=sys.stderr)
        sys.exit(1)
