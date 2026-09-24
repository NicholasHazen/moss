#!/usr/bin/env python3
"""Prepare a persistent course Cargo project and check its actual saved sources.

The practice project is separate from live Moss. Learner tests remain intact;
course acceptance tests run from a temporary copy. This executes trusted local
Rust with normal local permissions, not inside a security sandbox.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

from labs import ROOT, WORK, TOOLCHAIN, environment

REFERENCE = ROOT / "ecosystem"


def validate_manifest(files):
    """Keep the standalone course snapshot's compilation inputs explicit."""
    manifest = tomllib.loads(files["Cargo.toml"].decode())
    if manifest.get("workspace", {}).get("members") or manifest.get("workspace", {}).get("default-members"):
        raise ValueError("Course checks support one package; workspace members need their own runner")
    if manifest.get("package", {}).get("workspace"):
        raise ValueError("Course project must have its own workspace, not an external workspace")

    def dependencies(table):
        if isinstance(table, dict):
            for key, value in table.items():
                if key in {"dependencies", "dev-dependencies", "build-dependencies", "patch", "replace"}:
                    def local_path(value):
                        if isinstance(value, dict):
                            if "path" in value:
                                raise ValueError("Local path dependencies are outside the course snapshot; use the standalone course package")
                            for nested in value.values():
                                local_path(nested)
                    local_path(value)
                else:
                    dependencies(value)
    dependencies(manifest)
    if "build.rs" in files or manifest.get("package", {}).get("build") not in (None, False):
        raise ValueError("Custom build scripts are not supported by the course snapshot runner")
    targets = [manifest.get("lib", {})]
    for kind in ("bin", "test", "example", "bench"):
        targets.extend(manifest.get(kind, []))
    for target in targets:
        if str(target.get("name", "")).startswith("__fieldnotes_"):
            raise ValueError("Cargo target names beginning __fieldnotes_ are reserved for course checks")
        path = target.get("path")
        if path is not None and (Path(path).is_absolute() or ".." in Path(path).parts or path not in files):
            raise ValueError(f"Cargo target path is outside the saved course inputs: {path}")
    if manifest.get("lib", {}).get("path", "src/lib.rs") != "src/lib.rs":
        raise ValueError("Course library must remain at src/lib.rs for the browser adapter")
    return manifest


def source_files(project):
    """Snapshot the declared Cargo inputs, including test/example support files."""
    project = Path(project).resolve()
    paths = [project / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "build.rs")]
    for name in ("src", "tests", "examples", "benches"):
        directory = project / name
        if directory.is_symlink():
            raise ValueError(f"Course project source directories must not be symlinks: {directory}")
        if directory.exists():
            paths.extend(sorted(directory.rglob("*")))
    result = {}
    for path in paths:
        if path.is_symlink():
            raise ValueError(f"Course project inputs must be ordinary files, not symlinks: {path}")
        if path.is_file():
            result[path.relative_to(project).as_posix()] = path.read_bytes()
    for required in ("Cargo.toml", "Cargo.lock", "src/lib.rs"):
        if required not in result:
            raise ValueError(f"Course project is missing {required}: {project}")
    validate_manifest(result)
    return result


def fingerprint(files):
    digest = hashlib.sha256()
    for name, data in sorted(files.items()):
        digest.update(name.encode() + b"\0" + str(len(data)).encode() + b"\0" + data)
    return digest.hexdigest()


def prepare(destination, source=None):
    # copytree refuses an existing directory. It never refreshes a learner copy.
    shutil.copytree(source or REFERENCE, destination, ignore=shutil.ignore_patterns("target", ".git"))
    print(f"Created persistent course project: {destination.resolve()}")
    print("Open its Cargo.toml in your editor. Edit source and add tests in this same directory.")
    print("Preparing another checkpoint never updates or overwrites this copy.")


def run(command, package):
    print("+ " + " ".join(command), flush=True)
    env = environment()
    env["CARGO_TARGET_DIR"] = str(WORK / "fieldwork-target")
    result = subprocess.run(command, cwd=package, env=env, text=True,
                            capture_output=True, timeout=600)
    print(result.stdout, end="")
    if result.returncode:
        print(result.stderr, file=sys.stderr, end="")
        raise subprocess.CalledProcessError(result.returncode, command)
    return result.stdout


def acceptance_suites(stage="shared-meadow"):
    if stage not in {"shared-meadow", "population", "mobile", "resting", "hunting", "refuge"}:
        raise ValueError(f"Unknown fieldwork stage: {stage}")
    suites = {"acceptance": (REFERENCE / "tests/acceptance.rs").read_bytes()}
    if stage in {"population", "mobile", "resting", "hunting", "refuge"}:
        suites["population"] = (REFERENCE / "tests/population.rs").read_bytes()
    if stage in {"mobile", "resting", "hunting", "refuge"}:
        suites["mobile"] = (REFERENCE / "tests/mobile.rs").read_bytes()
    if stage in {"resting", "hunting", "refuge"}:
        suites["rest"] = (REFERENCE / "tests/rest.rs").read_bytes()
    if stage in {"hunting", "refuge"}:
        suites["hunting"] = (REFERENCE / "tests/hunting.rs").read_bytes()
    if stage == "refuge":
        suites["refuge"] = (REFERENCE / "tests/refuge.rs").read_bytes()
    return suites


def check(project, label, stage="shared-meadow"):
    selected = source_files(project)
    selected_hash = fingerprint(selected)
    canonical = acceptance_suites(stage)
    required_by_suite = {}
    WORK.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="fieldwork-", dir=WORK) as directory:
        package = Path(directory)
        for name, data in selected.items():
            path = package / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        # A concurrently checked project may share this crate's name/version but
        # contain different source. Every acceptance invocation owns its target.
        command = ["cargo", f"+{TOOLCHAIN}", "test", "--offline", "--locked", "--color", "never",
                   "--target-dir", str(package / "target")]
        for name, source in canonical.items():
            target = package / f"tests/__fieldnotes_{name}.rs"
            if target.exists():
                raise ValueError(f"{target.name} is reserved for the temporary course checks")
            target.parent.mkdir(exist_ok=True)
            target.write_bytes(source)
            required = set(re.findall(r"#\[test\]\s*(?:#\[[^\n]*\]\s*)*fn\s+(\w+)\s*\(", source.decode()))
            if not required:
                raise ValueError(f"Course {name} suite has no named tests")
            suite = command + ["--test", f"__fieldnotes_{name}"]
            listed = run(suite + ["--", "--list"], package)
            found = {line.removesuffix(": test") for line in listed.splitlines() if line.endswith(": test")}
            if not required <= found:
                raise ValueError(f"Missing course checks in {name}: {sorted(required - found)}")
            required = found
            output = run(suite + ["--", "--test-threads=1", "--color", "never"], package)
            passed = set(re.findall(r"^test (\S+)(?: - should panic)? \.\.\. ok$", output, re.MULTILINE))
            if not required <= passed:
                raise ValueError(f"Course checks did not execute successfully in {name}: {sorted(required - passed)}")
            required_by_suite[name] = sorted(required)
        # Keep every learner test. Cargo's output retains pass/fail/ignored counts.
        run(command + ["--all-targets", "--", "--test-threads=1", "--color", "never"], package)
    if fingerprint(source_files(project)) != selected_hash:
        raise ValueError("The selected source changed during this check; rerun before using its result")
    if acceptance_suites(stage) != canonical:
        raise ValueError("The course acceptance suite changed during this check; rerun it")
    names = [test if stage == "shared-meadow" else f"{suite}::{test}"
             for suite, tests in required_by_suite.items() for test in tests]
    acceptance_hash = (hashlib.sha256(canonical["acceptance"]).hexdigest()
                       if stage == "shared-meadow" else fingerprint(canonical))
    report = {"format": 1, "exercise": stage, "selection": label,
              "sourceHash": selected_hash, "acceptanceHash": acceptance_hash,
              "toolchain": TOOLCHAIN, "requiredTests": sorted(names),
              "requiredBySuite": required_by_suite, "passed": True}
    print(f"Checked {label} ({stage}): {len(names)} required acceptance tests passed.")
    print("The project's other test outcomes are printed above; ignored tests are not verified.")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    new = commands.add_parser("prepare")
    new.add_argument("--dest", type=Path, required=True)
    verify = commands.add_parser("check")
    selection = verify.add_mutually_exclusive_group(required=True)
    selection.add_argument("--project", type=Path)
    selection.add_argument("--reference", action="store_true")
    verify.add_argument("--report", type=Path)
    verify.add_argument("--stage", choices=["shared-meadow", "population", "mobile", "resting", "hunting", "refuge"], default="shared-meadow")
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.dest, ROOT / "checkpoints/shared-meadow")
        return
    project = args.project.resolve() if args.project else REFERENCE
    label = f"LEARNER PROJECT {project}" if args.project else "COURSE REFERENCE"
    print(label, flush=True)
    report = check(project, label, args.stage)
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"Fieldwork check stopped: {error}", file=sys.stderr)
        sys.exit(1)
