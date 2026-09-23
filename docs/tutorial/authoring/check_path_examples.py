#!/usr/bin/env python3
"""Check worked references in an isolated copy, never the live world.

Default: the sixteen path examples. --movement: today's v2 answer and prepared
movement/maintenance checks. Run from any directory. Uses only Python's standard
library and Moss's toolchain/lockfile. Dependencies must be cached (Cargo offline).
"""

from __future__ import annotations

import argparse
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


def tagged_example(file: Path, marker: str) -> str:
    pattern = rf"^<!-- {re.escape(marker)} -->\s*```rust\n(.*?)\n```"
    text = file.read_text()
    examples = re.findall(pattern, text, re.DOTALL | re.MULTILINE)
    markers = re.findall(rf"^<!-- {re.escape(marker)} -->$", text, re.MULTILINE)
    if len(markers) != 1 or len(examples) != 1:
        raise ValueError(f"{file.name} needs exactly one complete {marker!r} Rust fence")
    return examples[0]


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


def path_test_names(output: str, sessions: list[int]) -> tuple[str, ...]:
    # Read libtest's compiled inventory, not Rust source or its attributes.
    names = tuple(re.findall(r"^(\S+): test$", output, re.MULTILINE))
    expected_modules = {f"session_{number:02d}" for number in sessions}
    found_modules = {name.split("::", 1)[0] for name in names if "::" in name}
    missing = expected_modules - found_modules
    if missing:
        raise ValueError("No compiled tests found for: " + ", ".join(sorted(missing)))
    unexpected = [name for name in names if name.split("::", 1)[0] not in expected_modules
                  or "::" not in name]
    if unexpected:
        raise ValueError("Tests outside selected reference modules: " + ", ".join(unexpected))
    return names


def run_path(workspace: Path, env: dict[str, str], sessions: list[int]) -> int:
    command = [
        "scripts/with-toolchain.sh", "cargo", "test", "-p", "moss-sim",
        "--locked", "--offline", "--test", "month_guide",
    ]
    output_options = ["--format", "pretty", "--color", "never"]
    discovery = [*command, "--", "--list", *output_options]
    print("$ " + " ".join(discovery), flush=True)
    result = subprocess.run(
        discovery, cwd=workspace, env=env, check=False, text=True, stdout=subprocess.PIPE,
    )
    print(result.stdout, end="", flush=True)
    if result.returncode:
        return result.returncode
    names = path_test_names(result.stdout, sessions)
    # An ignored reference must fail this check, not be silently activated.
    return run_checked([*command, "--", *output_options], workspace, env, names)


def run_movement(workspace: Path, env: dict[str, str], examples: tuple[str, str]) -> int:
    normalized = []
    for example in examples:
        formatted = subprocess.run(
            ["scripts/with-toolchain.sh", "rustfmt", "--edition", "2024", "--emit", "stdout"],
            input=example, cwd=workspace, env=env, text=True, capture_output=True, check=True,
        )
        normalized.append(formatted.stdout)
    if normalized[0] != normalized[1]:
        raise ValueError("V2 and Chapter 4 movement answers differ after pinned rustfmt")

    lessons = workspace / "crates/moss-sim/src/lessons.rs"
    simulation = workspace / "crates/moss-sim/src/simulation.rs"
    # Validate both replacements before running or writing the copied solution.
    updated_lessons = replace_movement_helper(lessons.read_text(), examples[0])
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
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--session", type=int, choices=range(1, 17))
    mode.add_argument("--movement", action="store_true", help="check today's v2 answer and isolated activation")
    parser.add_argument("--keep-workspace", action="store_true")
    args = parser.parse_args(argv)

    root = Path(__file__).resolve().parents[3]
    path = root / "docs/tutorial/path"
    modules = []
    try:
        if args.movement:
            examples = (
                tagged_example(path.parent / "today-v2.md", "example: movement-v2-helper"),
                tagged_example(path.parent / "04-movement.md", "example: movement-helper"),
            )
        else:
            sessions = [args.session] if args.session else list(range(1, 17))
            for number in sessions:
                files = sorted(path.glob(f"{number:02d}-*.md"))
                if len(files) != 1:
                    parser.error(f"Expected one guide for session {number:02d}, found {len(files)}")
                example = tagged_example(files[0], f"runnable: session-{number:02d}")
                modules.append(f"mod session_{number:02d} {{\n{example}\n}}\n")
    except (OSError, ValueError) as error:
        parser.error(str(error))

    workspace = Path(tempfile.mkdtemp(prefix="moss-path-examples-"))
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
        if args.movement:
            return run_movement(workspace, env, examples)
        (workspace / "crates/moss-sim/tests/month_guide.rs").write_text("\n".join(modules))
        return run_path(workspace, env, sessions)
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
