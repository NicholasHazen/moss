#!/usr/bin/env python3
"""Check Fieldnotes' exact movement answer in a copy, never the live world.

Runs the helper and prepared movement/maintenance checks. Run from any directory. Uses only Python's standard
library and Moss's toolchain/lockfile. Dependencies must be cached (Cargo offline).
"""

from __future__ import annotations

import argparse
from html import unescape
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile


MOVEMENT_TESTS = (
    "movement_charges_only_an_affordable_actual_step",
    "movement_tick_resolves_target_and_reset",
    "movement_rejects_missing_empty_and_unaffordable_targets",
    "maintenance_spends_energy_and_stops_at_zero",
    "maintenance_uses_species_rates_and_stops_at_zero",
)


def movement_answer(file: Path) -> str:
    text = file.read_text()
    marker = 'data-movement-reference="move_one_cell"'
    pattern = r'<pre><code class="language-rust" data-movement-reference="move_one_cell">(.*?)</code></pre>'
    examples = re.findall(pattern, text, re.DOTALL)
    if text.count(marker) != 1 or len(examples) != 1:
        raise ValueError(f"{file.name} needs exactly one complete movement reference block")
    if "<" in examples[0] or ">" in examples[0]:
        raise ValueError("Movement code must contain escaped HTML text, not nested tags")
    return unescape(examples[0])


def replace_movement_helper(source: str, helper: str) -> str:
    # Deliberately limited to this file's top-level, rustfmt-shaped function.
    # Nested closing braces are indented. This is not a general Rust parser.
    pattern = r"^pub fn move_one_cell\(\n.*?^\}"
    matches = list(re.finditer(pattern, source, re.DOTALL | re.MULTILINE))
    answers = list(re.finditer(pattern, helper, re.DOTALL | re.MULTILINE))
    if (len(matches) != 1 or len(answers) != 1
            or answers[0].span() != (0, len(helper))):
        raise ValueError("Expected one complete top-level move_one_cell in source and answer")
    current = matches[0]
    if current[0].split("{", 1)[0].split() != helper.split("{", 1)[0].split():
        raise ValueError("move_one_cell signature changed; review the lesson before running it")
    return source[:current.start()] + helper + source[current.end():]


def activate_movement(source: str) -> str:
    old = "    schedule.add_systems((lessons::spend_energy, complete_tick).chain());"
    new = """    schedule.add_systems(
        (lessons::spend_energy, lessons::move_to_food, complete_tick).chain(),
    );"""
    if source.count(old) != 1:
        raise ValueError("Expected the maintenance-only schedule once; review the runner after schedule changes")
    return source.replace(old, new)


def run_checked(command: list[str], workspace: Path, env: dict[str, str],
                expected_tests: tuple[str, ...] = ()) -> int:
    print("$ " + " ".join(command), flush=True)
    result = subprocess.run(
        command, cwd=workspace, env=env, check=False, text=True,
        stdout=subprocess.PIPE if expected_tests else None,
    )
    if expected_tests:
        print(result.stdout, end="", flush=True)
    if result.returncode:
        return result.returncode
    if expected_tests:
        passed = set(re.findall(
            r"^test (\S+)(?: - should panic)? \.\.\. ok$", result.stdout, re.MULTILINE,
        ))
        missing = set(expected_tests) - passed
        if missing:
            raise ValueError("Expected tests did not report passing: " + ", ".join(sorted(missing)))
    return 0


def run_movement(workspace: Path, env: dict[str, str], example: str) -> int:
    subprocess.run(
        ["scripts/with-toolchain.sh", "rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=example, cwd=workspace, env=env, text=True, capture_output=True, check=True,
    )

    lessons = workspace / "crates/moss-sim/src/lessons.rs"
    simulation = workspace / "crates/moss-sim/src/simulation.rs"
    # Validate both replacements before running or writing the copied solution.
    updated_lessons = replace_movement_helper(lessons.read_text(), example)
    activated_schedule = activate_movement(simulation.read_text())
    lessons.write_text(updated_lessons)
    cargo = ["scripts/with-toolchain.sh", "cargo"]
    tests = [*cargo, "test", "-p", "moss-sim", "--locked", "--offline"]
    output = ["--format", "pretty", "--color", "never"]
    result = run_checked(
        [*tests, "--test", "movement", MOVEMENT_TESTS[0], "--", "--exact", *output],
        workspace, env, MOVEMENT_TESTS[:1],
    )
    if result:
        return result

    simulation.write_text(activated_schedule)
    targets = ["--test", "movement", "--test", "maintenance"]
    result = run_checked(
        [*tests, *targets, "--", "--include-ignored", *output],
        workspace, env, MOVEMENT_TESTS,
    )
    if result:
        return result
    return run_checked(
        [*cargo, "clippy", "-p", "moss-sim", "--locked", "--offline", *targets, "--", "-D", "warnings"],
        workspace, env,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keep-workspace", action="store_true")
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[2]
    try:
        example = movement_answer(root / "learning/content/14-movement.html")
    except (OSError, ValueError) as error:
        parser.error(str(error))

    workspace = Path(tempfile.mkdtemp(prefix="moss-movement-reference-"))
    try:
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
            shutil.copy2(root / name, workspace / name)
        for name in ("crates", "scripts"):
            shutil.copytree(root / name, workspace / name)
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(workspace / "target")
        env.pop("RUSTUP_TOOLCHAIN", None)  # Honor the copied rust-toolchain.toml.
        print("Worked references only: this does not activate or test future live systems.", flush=True)
        print(f"Isolated workspace: {workspace}", flush=True)
        return run_movement(workspace, env, example)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Example check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 1
    finally:
        if args.keep_workspace:
            print(f"Kept reference workspace: {workspace}", flush=True)
        else:
            shutil.rmtree(workspace)


if __name__ == "__main__":
    raise SystemExit(main())
