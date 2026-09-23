"""Focused safety and evidence checks; no Rust commands run in these fixtures."""

from contextlib import redirect_stderr, redirect_stdout
import io
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_path_examples as runner


STUB = """pub fn move_one_cell(
    position: &mut Position,
) -> u32 {
    todo!()
}"""
ANSWER = """pub fn move_one_cell(
    position: &mut Position,
) -> u32 {
    if position.x < 2 {
        position.x += 1;
        return 1;
    }
    0
}"""
SCHEDULE = "    schedule.add_systems((lessons::spend_energy, complete_tick).chain());"


class ExampleRunnerChecks(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="moss-runner-check-")
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.enterContext(redirect_stdout(io.StringIO()))
        self.enterContext(redirect_stderr(io.StringIO()))

    def write(self, root, name, contents):
        destination = root / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(contents, encoding="utf-8")
        return destination

    def simulation_files(self, root):
        lessons = self.write(root, "crates/moss-sim/src/lessons.rs", STUB)
        simulation = self.write(root, "crates/moss-sim/src/simulation.rs", SCHEDULE)
        return lessons, simulation

    def formatter_result(self):
        return subprocess.CompletedProcess([], 0, stdout=ANSWER, stderr="")

    def project_fixture(self):
        root = self.directory / "live"
        self.simulation_files(root)
        (root / "crates/moss-sim/tests").mkdir()
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
            self.write(root, name, f"fixture for {name}\n")
        self.write(root, "scripts/with-toolchain.sh", "fixture wrapper\n")
        for name, marker in (
            ("today-v2.md", "movement-v2-helper"),
            ("04-movement.md", "movement-helper"),
        ):
            self.write(root, f"docs/tutorial/{name}",
                       f"<!-- example: {marker} -->\n```rust\n{ANSWER}\n```\n")
        return root

    def test_tagged_example_requires_one_complete_fence(self):
        fence = f"<!-- example: movement -->\n```rust\n{ANSWER}\n```\n"
        file = self.write(self.directory, "lesson.md", fence)
        self.assertEqual(runner.tagged_example(file, "example: movement"), ANSWER)
        for contents in (
            "# No example", fence + fence, fence.removesuffix("```\n"),
            fence + "<!-- example: movement -->\nmissing fence\n",
        ):
            with self.subTest(contents=contents):
                file.write_text(contents)
                with self.assertRaises(ValueError):
                    runner.tagged_example(file, "example: movement")

    def test_helper_replacement_preserves_neighbors_and_nested_blocks(self):
        before = "fn previous() {}\n\n"
        after = "\n\n/// Prepared adapter.\npub fn move_to_food() {}\n"
        self.assertEqual(
            runner.replace_movement_helper(before + STUB + after, ANSWER),
            before + ANSWER + after,
        )

    def test_helper_replacement_rejects_absent_or_duplicate_functions(self):
        for source, answer in (
            ("fn other() {}", ANSWER),
            (STUB + "\n" + STUB, ANSWER),
            (STUB, "fn other() {}"),
            (STUB, ANSWER + "\n" + ANSWER),
            (STUB, ANSWER + "\nfn unrelated() {}"),
        ):
            with self.subTest(source=source, answer=answer):
                with self.assertRaises(ValueError):
                    runner.replace_movement_helper(source, answer)

    def test_signature_and_schedule_drift_require_review(self):
        with self.assertRaisesRegex(ValueError, "signature changed"):
            runner.replace_movement_helper(STUB, ANSWER.replace("-> u32", "-> u64"))
        activated = runner.activate_movement(SCHEDULE)
        self.assertIn("lessons::move_to_food", activated)
        for source in ("", SCHEDULE + "\n" + SCHEDULE, activated):
            with self.subTest(source=source):
                with self.assertRaises(ValueError):
                    runner.activate_movement(source)

    def test_zero_ignored_or_missing_expected_tests_cannot_pass(self):
        for output in (
            "running 0 tests\ntest result: ok. 0 passed; 0 failed\n",
            "test wanted ... ignored\n",
            "test other ... ok\n",
            "test wanted ... ok\n",
        ):
            with self.subTest(output=output):
                result = subprocess.CompletedProcess([], 0, stdout=output)
                with patch.object(runner.subprocess, "run", return_value=result):
                    with self.assertRaisesRegex(ValueError, "did not report passing"):
                        runner.run_checked(["cargo"], self.directory, {}, ("wanted", "second"))

    def test_named_passes_still_require_successful_process_exit(self):
        output = "test wanted ... ok\ntest second ... ok\n"
        for returncode in (0, 101):
            with self.subTest(returncode=returncode):
                result = subprocess.CompletedProcess([], returncode, stdout=output)
                with patch.object(runner.subprocess, "run", return_value=result):
                    self.assertEqual(
                        runner.run_checked(["cargo"], self.directory, {}, ("wanted", "second")),
                        returncode,
                    )

    def test_path_inventory_accepts_multiple_and_nested_test_names(self):
        names = ("session_01::meal", "session_01::nested::empty", "session_02::sharing")
        output = "\n".join(f"{name}: test" for name in names) + "\n\n3 tests, 0 benchmarks\n"
        self.assertEqual(runner.path_test_names(output, [1, 2]), names)

    def test_path_inventory_requires_a_test_in_every_selected_module(self):
        for output in (
            "0 tests, 0 benchmarks\n",
            "session_01::meal: benchmark\n",
            "session_01::meal test\n",
            "session_01::meal: test\n",
            "session_03::meal: test\n",
        ):
            with self.subTest(output=output):
                with self.assertRaisesRegex(ValueError, "No compiled tests found"):
                    runner.path_test_names(output, [1, 2])
        for extra in ("unwrapped", "session_02::unexpected"):
            with self.subTest(extra=extra):
                with self.assertRaisesRegex(ValueError, "outside selected reference modules"):
                    runner.path_test_names(f"session_01::meal: test\n{extra}: test\n", [1])

    def test_failed_path_discovery_does_not_execute_tests(self):
        result = subprocess.CompletedProcess([], 101, stdout="")
        with patch.object(runner.subprocess, "run", return_value=result) as process:
            with patch.object(runner, "run_checked") as execute:
                self.assertEqual(runner.run_path(self.directory, {}, [1]), 101)
        self.assertIn("--list", process.call_args.args[0])
        execute.assert_not_called()

    def test_path_run_requires_every_discovered_test_to_pass(self):
        listing = "session_01::meal: test\nsession_01::empty: test\n"
        for second in ("test session_01::empty ... ignored\n", "", "test session_01::empty ... ok\n"):
            with self.subTest(second=second):
                results = [
                    subprocess.CompletedProcess([], 0, stdout=listing),
                    subprocess.CompletedProcess([], 0, stdout="test session_01::meal ... ok\n" + second),
                ]
                with patch.object(runner.subprocess, "run", side_effect=results) as process:
                    if second.endswith("ok\n"):
                        self.assertEqual(runner.run_path(self.directory, {}, [1]), 0)
                    else:
                        with self.assertRaisesRegex(ValueError, "did not report passing.*session_01::empty"):
                            runner.run_path(self.directory, {}, [1])
                command = process.call_args.args[0]
                self.assertNotIn("--list", command)
                self.assertNotIn("--include-ignored", command)
                self.assertEqual(command[-4:], ["--format", "pretty", "--color", "never"])

    def test_expected_panic_is_a_pass_but_not_an_arbitrary_suffix(self):
        for suffix in (" - should panic", " - ignored"):
            with self.subTest(suffix=suffix):
                result = subprocess.CompletedProcess([], 0, stdout=f"test wanted{suffix} ... ok\n")
                with patch.object(runner.subprocess, "run", return_value=result):
                    if suffix == " - should panic":
                        self.assertEqual(runner.run_checked(["cargo"], self.directory, {}, ("wanted",)), 0)
                    else:
                        with self.assertRaises(ValueError):
                            runner.run_checked(["cargo"], self.directory, {}, ("wanted",))

    def test_failed_focus_does_not_activate_schedule(self):
        lessons, simulation = self.simulation_files(self.directory)
        with patch.object(runner.subprocess, "run", return_value=self.formatter_result()):
            with patch.object(runner, "run_checked", return_value=101) as check:
                self.assertEqual(runner.run_movement(self.directory, {}, (ANSWER, ANSWER)), 101)
        self.assertEqual(lessons.read_text(), ANSWER)
        self.assertEqual(simulation.read_text(), SCHEDULE)
        self.assertEqual(check.call_count, 1)

    def test_answer_disagreement_or_schedule_drift_writes_no_solution(self):
        lessons, simulation = self.simulation_files(self.directory)
        for disagreement in (True, False):
            with self.subTest(disagreement=disagreement):
                simulation.write_text(SCHEDULE if disagreement else "changed schedule")
                second = self.formatter_result()
                if disagreement:
                    second.stdout = ANSWER.replace("return 1", "return 2")
                with patch.object(runner.subprocess, "run", side_effect=[self.formatter_result(), second]):
                    with patch.object(runner, "run_checked") as check:
                        with self.assertRaises(ValueError):
                            runner.run_movement(self.directory, {}, (ANSWER, ANSWER))
                self.assertEqual(lessons.read_text(), STUB)
                check.assert_not_called()

    def test_integrated_failure_stops_before_clippy_and_includes_ignored_tests(self):
        self.simulation_files(self.directory)
        with patch.object(runner.subprocess, "run", return_value=self.formatter_result()):
            with patch.object(runner, "run_checked", side_effect=[0, 101]) as check:
                self.assertEqual(runner.run_movement(self.directory, {}, (ANSWER, ANSWER)), 101)
        self.assertEqual(check.call_count, 2)
        command, _, _, expected = check.call_args.args
        self.assertIn("--include-ignored", command)
        self.assertIn("movement", command)
        self.assertIn("maintenance", command)
        self.assertEqual(set(expected), set(runner.MOVEMENT_TESTS))

    def test_main_isolates_writes_and_cleans_or_retains_failed_workspace(self):
        root = self.project_fixture()
        before = {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}
        for keep in (False, True):
            with self.subTest(keep=keep):
                scratch = self.directory / f"scratch-{keep}"
                scratch.mkdir()

                def fail_in_copy(workspace, env, examples):
                    self.assertEqual(workspace, scratch)
                    self.assertEqual(env["CARGO_TARGET_DIR"], str(scratch / "target"))
                    self.assertNotIn("RUSTUP_TOOLCHAIN", env)
                    self.assertEqual(examples, (ANSWER, ANSWER))
                    (workspace / "crates/moss-sim/src/lessons.rs").write_text("copied edit")
                    raise ValueError("fixture failure after editing the copy")

                with patch.object(runner, "__file__", str(root / "docs/tutorial/authoring/check_path_examples.py")):
                    with patch.object(runner.tempfile, "mkdtemp", return_value=str(scratch)):
                        with patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "unexpected-toolchain"}):
                            with patch.object(runner, "run_movement", side_effect=fail_in_copy):
                                args = ["--movement"] + (["--keep-workspace"] if keep else [])
                                self.assertEqual(runner.main(args), 1)
                self.assertEqual(scratch.exists(), keep)
                after = {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}
                self.assertEqual(after, before)

    def test_movement_and_session_modes_are_mutually_exclusive(self):
        with patch.object(runner.tempfile, "mkdtemp") as create:
            with self.assertRaises(SystemExit) as error:
                runner.main(["--movement", "--session", "4"])
        self.assertEqual(error.exception.code, 2)
        create.assert_not_called()

    def test_path_main_isolates_selected_examples_and_cleans_or_retains_copy(self):
        root = self.project_fixture()
        # A macro can generate tests; source text is not the compiled inventory.
        for number in (1, 2):
            self.write(root, f"docs/tutorial/path/{number:02d}-fixture.md",
                       f"<!-- runnable: session-{number:02d} -->\n```rust\nmake_test!();\n```\n")
        before = {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}
        for outcome, keep in ((0, False), (101, False), (ValueError("no compiled tests"), False), (101, True)):
            with self.subTest(outcome=outcome, keep=keep):
                scratch = Path(tempfile.mkdtemp(dir=self.directory))

                def run_copy(workspace, env, sessions):
                    self.assertEqual(workspace, scratch)
                    self.assertEqual(sessions, [2])
                    self.assertEqual(env["CARGO_TARGET_DIR"], str(scratch / "target"))
                    self.assertNotIn("RUSTUP_TOOLCHAIN", env)
                    self.assertEqual((scratch / "crates/moss-sim/tests/month_guide.rs").read_text(),
                                     "mod session_02 {\nmake_test!();\n}\n")
                    if isinstance(outcome, Exception):
                        raise outcome
                    return outcome

                with patch.object(runner, "__file__", str(root / "docs/tutorial/authoring/check_path_examples.py")):
                    with patch.object(runner.tempfile, "mkdtemp", return_value=str(scratch)):
                        with patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "unexpected-toolchain"}):
                            with patch.object(runner, "run_path", side_effect=run_copy):
                                args = ["--session", "2"] + (["--keep-workspace"] if keep else [])
                                self.assertEqual(runner.main(args), 1 if isinstance(outcome, Exception) else outcome)
                self.assertEqual(scratch.exists(), keep)
                after = {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}
                self.assertEqual(after, before)

if __name__ == "__main__":
    unittest.main()
