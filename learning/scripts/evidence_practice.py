#!/usr/bin/env python3
"""Check learner-authored tests against correct and deliberately wrong projections.

This is trusted local Rust execution, using the pinned course toolchain. Unlike
the implementation lab, this task preserves and runs the learner's own tests.
A mutant must compile and fail a named test; compiler errors do not count.
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

from labs import ROOT, WORK, TOOLCHAIN, TEST_MARKER, environment, examples


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError("The evidence lesson changed; review this counterexample before running it")
    return source.replace(before, after, 1)


def implementations():
    reference = examples()[("data-runnable", "evidence")].split(TEST_MARKER, 1)[0]
    arithmetic = "Some(((u64::from(used.min(capacity)) * 100) / u64::from(capacity)) as u32)"
    schedule = "schedule.add_systems((project_selected, observe_display).chain());"
    changes = [
        ("rounding", "Fractional percentages round upward", arithmetic,
         "Some(((u64::from(used.min(capacity)) * 100 + u64::from(capacity) - 1) / u64::from(capacity)) as u32)",
         "Try a fractional result whose floor differs from its ceiling."),
        ("overflow", "Intermediate multiplication wraps in u32", arithmetic,
         "Some(used.min(capacity).wrapping_mul(100) / capacity)",
         "Try large valid inputs; a full sample should still report 100."),
        ("selection", "Projection reads the unselected row", "Query<&Sample, With<Selected>>",
         "Query<&Sample, Without<Selected>>",
         "Run the real system with selected and unselected rows that yield different values."),
        ("omitted", "Installed schedule omits projection", schedule,
         "schedule.add_systems(observe_display);",
         "A helper call cannot show that the installed schedule used the helper."),
        ("order", "Observer runs before projection", schedule,
         "schedule.add_systems((observe_display, project_selected).chain());",
         "Assert what the observer recorded, not only the final display."),
        ("stale", "Missing or ambiguous selection retains old data", "Err(_) => None,",
         "Err(_) => display.0,",
         "Remove the selection or make it ambiguous; inspect both the current value and its trace."),
    ]
    cases = [{"id": "reference", "title": "Correct projection", "source": reference}]
    for name, title, before, after, hint in changes:
        cases.append({"id": name, "title": title, "hint": hint,
                      "source": replace_once(reference, before, after)})
    return cases


def compose(implementation, tests):
    return implementation + "\n#[cfg(test)]\nmod learner_tests {\nuse super::*;\n" + tests + "\n}\n"


def outcomes(output):
    return {
        state: set(re.findall(r"^test (learner_tests::\S+)(?: - should panic)? \.\.\. "
                             + state + (r"(?:, .*)?" if state == "ignored" else "")
                             + r"$", output, re.MULTILINE))
        for state in ("ok", "FAILED", "ignored")
    }


def classify(returncode, output, discovered):
    results = outcomes(output)
    if not discovered:
        return "no-tests"
    if results["ignored"] or (results["ok"] | results["FAILED"]) != discovered:
        return "incomplete"
    if returncode == 0 and results["ok"] == discovered:
        return "passed"
    if returncode != 0 and results["FAILED"]:
        return "failed"
    return "execution-error"


def execute(package, source):
    (package / "src/main.rs").write_text(source)
    command = ["cargo", f"+{TOOLCHAIN}", "test", "--offline", "--locked", "--color", "never"]
    listing = subprocess.run(command + ["--", "--list"], cwd=package,
                             env=environment(), text=True, capture_output=True, timeout=600)
    if listing.returncode:
        return {"state": "build-error", "tests": [], "failed": [],
                "output": listing.stdout + listing.stderr}
    found = {line.removesuffix(": test") for line in listing.stdout.splitlines()
             if line.startswith("learner_tests::") and line.endswith(": test")}
    if not found:
        return {"state": "no-tests", "tests": [], "failed": [], "output": listing.stdout}
    result = subprocess.run(command + ["--", "learner_tests::", "--test-threads=1", "--color", "never"],
                            cwd=package, env=environment(), text=True,
                            capture_output=True, timeout=600)
    return {"state": classify(result.returncode, result.stdout, found),
            "tests": sorted(found), "failed": sorted(outcomes(result.stdout)["FAILED"]),
            "output": result.stdout + result.stderr}


def evaluate(tests, label, executor=execute):
    cases = implementations()
    digest = hashlib.sha256(compose(cases[0]["source"], tests).encode()).hexdigest()
    report = {"format": 1, "exercise": "evidence-test-design", "selection": label,
              "sourceHash": digest, "toolchain": TOOLCHAIN, "cases": [], "complete": False}
    WORK.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="evidence-tests-", dir=WORK) as directory:
        package = Path(directory)
        (package / "src").mkdir()
        shutil.copyfile(ROOT / "scripts/lab.Cargo.toml", package / "Cargo.toml")
        shutil.copyfile(ROOT / "scripts/lab.Cargo.lock", package / "Cargo.lock")
        expected_tests = None
        for case in cases:
            result = executor(package, compose(case["source"], tests))
            state = result["state"]
            if case["id"] == "reference":
                expected_tests = result["tests"]
            elif state not in {"passed", "failed"}:
                pass
            elif result["tests"] != expected_tests:
                state = "test-set-changed"
            elif state == "passed":
                state = "survived"
            elif state == "failed":
                state = "caught"
            entry = {"id": case["id"], "title": case["title"], "state": state,
                     "sourceHash": hashlib.sha256(compose(case["source"], tests).encode()).hexdigest(),
                     "tests": result["tests"], "failed": result["failed"]}
            report["cases"].append(entry)
            print(f"{case['id']}: {state} — {case['title']}", flush=True)
            if state == "caught":
                print("  Failed: " + ", ".join(result["failed"]))
            elif state == "survived":
                print("  " + case["hint"])
            elif state != "passed":
                print(result["output"])
            if case["id"] == "reference" and state != "passed":
                print("Repair the test against the correct program first. Counterexamples were not run.")
                return report
    report["complete"] = all(c["state"] == ("passed" if c["id"] == "reference" else "caught")
                             for c in report["cases"])
    if report["complete"]:
        print("Your tests accept the correct program and reject all six stated defects.")
        print("This establishes these counterexamples, not universal correctness or live Moss behavior.")
    elif any(c["state"] not in {"passed", "caught", "survived"} for c in report["cases"]):
        print("Practice remains incomplete. Resolve check errors before interpreting the missing evidence.")
        print("Build errors, missing or ignored tests, and interrupted checks do not count as detected defects.")
    else:
        print("Practice remains incomplete. Strengthen the cases that survived; keep correct behavior passing.")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    prepare = commands.add_parser("prepare")
    prepare.add_argument("--dest", type=Path, required=True)
    check = commands.add_parser("check")
    selection = check.add_mutually_exclusive_group(required=True)
    selection.add_argument("--file", type=Path)
    selection.add_argument("--answer", action="store_true")
    check.add_argument("--report", type=Path)
    args = parser.parse_args()
    blocks = examples()
    if args.command == "prepare":
        args.dest.mkdir(parents=True, exist_ok=False)
        target = args.dest / "evidence_tests.rs"
        target.write_text(blocks[("data-practice-starter", "evidence-tests-starter")])
        print(f"Created learner tests: {target.resolve()}")
        print("Edit and add #[test] functions. These tests will be preserved, not replaced.")
        return 0
    tests = args.file.read_text() if args.file else blocks[("data-practice-answer", "evidence-tests-answer")]
    label = f"LEARNER TESTS {args.file.resolve()}" if args.file else "WORKED ANSWER"
    print(label, flush=True)
    report = evaluate(tests, label)
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["complete"] else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"Test-design check stopped: {error}", file=sys.stderr)
        sys.exit(2)
