#!/usr/bin/env python3
"""Extract exact course examples; check references or an explicitly selected learner file.

This runs local Rust with normal local process permissions; it is not a sandbox.
Only original course examples and files explicitly passed by the learner are run.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / "work"
TOOLCHAIN = "1.93.1"
TEST_MARKER = "#[cfg(test)]"


class Examples(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.blocks = {}
        self.current = None

    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if tag != "code":
            return
        for attribute in ("data-runnable", "data-starter", "data-practice-starter", "data-practice-answer"):
            if attribute in values:
                self.current = (attribute, values[attribute])
                if self.current in self.blocks:
                    raise ValueError(f"Duplicate example: {self.current}")
                self.blocks[self.current] = ""

    def handle_endtag(self, tag):
        if tag == "code":
            self.current = None

    def handle_data(self, data):
        if self.current:
            self.blocks[self.current] += data


def examples():
    result = {}
    course = json.loads((ROOT / "course.json").read_text())
    modules = course["modules"] + [{"id": name} for name in course.get("guides", {})]
    for module in modules:
        path = ROOT / "content" / f"{module['id']}.html"
        parser = Examples()
        parser.feed(path.read_text())
        for key, text in parser.blocks.items():
            if key in result:
                raise ValueError(f"Duplicate example id: {key}")
            result[key] = text.strip() + "\n"
    return result


def environment():
    env = os.environ.copy()
    if sys.platform == "darwin" and "DEVELOPER_DIR" not in env and Path("/Library/Developer/CommandLineTools").exists():
        env["DEVELOPER_DIR"] = "/Library/Developer/CommandLineTools"
    env["CARGO_TARGET_DIR"] = str(WORK / "target")
    return env


def run(command, cwd, capture=False):
    print("+ " + " ".join(str(v) for v in command), flush=True)
    result = subprocess.run(command, cwd=cwd, env=environment(), text=True, capture_output=capture, timeout=600)
    if result.returncode:
        if capture:
            print(result.stdout)
            print(result.stderr, file=sys.stderr)
        raise subprocess.CalledProcessError(result.returncode, command)
    return result.stdout if capture else ""


def test_names(source):
    return {f"tests::{name}" for name in re.findall(r"#\[test\]\s*fn\s+([a-zA-Z0-9_]+)\s*\(", source)}


def checked_source(reference, learner):
    if learner is None:
        return reference
    # The starter has exactly one trailing test module. Keep the course's tests
    # authoritative even if a learner accidentally edits or removes their copy.
    if reference.count(TEST_MARKER) != 1:
        raise ValueError("Reference needs exactly one trailing test module")
    prefix = learner.split(TEST_MARKER, 1)[0]
    return prefix.rstrip() + "\n\n" + TEST_MARKER + reference.split(TEST_MARKER, 1)[1]


def check(name, source, reference):
    required = test_names(reference)
    if not required:
        raise ValueError("Reference has no required tests; refusing a zero-test pass")
    WORK.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=f"{name}-", dir=WORK) as directory:
        dest = Path(directory)
        is_ecs = "bevy_ecs::" in reference
        if is_ecs:
            (dest / "src").mkdir()
            (dest / "src/main.rs").write_text(source)
            shutil.copyfile(ROOT / "scripts/lab.Cargo.toml", dest / "Cargo.toml")
            shutil.copyfile(ROOT / "scripts/lab.Cargo.lock", dest / "Cargo.lock")
            command = ["cargo", f"+{TOOLCHAIN}", "test", "--offline", "--locked"]
            listed = run(command + ["--", "--list"], dest, True)
            found = {line.removesuffix(": test") for line in listed.splitlines() if line.endswith(": test")}
            if not required <= found:
                raise ValueError(f"Missing required tests: {sorted(required - found)}")
            output = run(command + ["--", "--nocapture"], dest, True)
            print(output)
            run(["cargo", f"+{TOOLCHAIN}", "run", "--offline", "--locked", "--quiet"], dest)
        else:
            path = dest / f"{name}.rs"
            path.write_text(source)
            binary = dest / "tests"
            run(["rustc", f"+{TOOLCHAIN}", "--edition=2024", "--test", str(path), "-o", str(binary)], dest)
            listed = run([str(binary), "--list"], dest, True)
            found = {line.removesuffix(": test") for line in listed.splitlines() if line.endswith(": test")}
            if not required <= found:
                raise ValueError(f"Missing required tests: {sorted(required - found)}")
            output = run([str(binary), "--nocapture"], dest, True)
            print(output)
            run(["rustc", f"+{TOOLCHAIN}", "--edition=2024", str(path), "-o", str(dest / "example")], dest)
            run([str(dest / "example")], dest)
        passed = set(re.findall(r"^test (\S+) \.\.\. ok$", output, re.MULTILINE))
        if not required <= passed:
            raise ValueError(f"Required tests did not pass: {sorted(required - passed)}")
        print(f"Checked {name}: {len(required)} required tests passed; program ran.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    prepare = commands.add_parser("prepare", help="create a separate exercise copy; never overwrite")
    prepare.add_argument("name")
    prepare.add_argument("--dest", type=Path, required=True)
    checks = commands.add_parser("check", help="check the printed reference or the selected learner file")
    checks.add_argument("name", help="example id or all")
    checks.add_argument("--file", type=Path)
    args = parser.parse_args()
    blocks = examples()
    if args.command == "prepare":
        code = blocks.get(("data-starter", args.name))
        if code is None:
            raise ValueError(f"No editable starter named {args.name}; this lab may be a reading experiment.")
        args.dest.mkdir(parents=True, exist_ok=False)
        path = args.dest / f"{args.name}.rs"
        path.write_text(code)
        print(f"Created learner copy: {path.resolve()}\nEdit only the named exercise function, then check with --file.")
        return
    if args.name == "all" and args.file:
        raise ValueError("--file requires one specific lab")
    names = sorted(name for kind, name in blocks if kind == "data-runnable") if args.name == "all" else [args.name]
    if not names:
        raise ValueError("No runnable labs found")
    for name in names:
        reference = blocks.get(("data-runnable", name))
        if reference is None:
            raise ValueError(f"No runnable reference named {name}")
        learner = args.file.read_text() if args.file else None
        print(f"\nChecking {'LEARNER FILE ' + str(args.file.resolve()) if args.file else 'PRINTED REFERENCE'}: {name}")
        check(name, checked_source(reference, learner), reference)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"Lab check stopped: {error}", file=sys.stderr)
        sys.exit(1)
