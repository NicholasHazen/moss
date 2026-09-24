"""Focused safety and evidence checks; no Rust commands run in these fixtures."""

from contextlib import redirect_stderr, redirect_stdout
import io
from html import escape
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_movement_reference as runner


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
        self.write(root, "learning/content/14-movement.html", self.reference())
        return root

    def reference(self):
        return ('<pre><code class="language-rust" data-movement-reference="move_one_cell">'
                + escape(ANSWER, quote=False) + '</code></pre>')

    def test_reference_requires_one_complete_escaped_html_block(self):
        block = self.reference()
        file = self.write(self.directory, "lesson.html", block)
        self.assertEqual(runner.movement_answer(file), ANSWER)
        for contents in (
            "No example", block + block, block.removesuffix("</code></pre>"),
            block + '<p data-movement-reference="move_one_cell">wrong</p>',
            block.replace("&lt;", "<"),
        ):
            with self.subTest(contents=contents):
                file.write_text(contents)
                with self.assertRaises(ValueError):
                    runner.movement_answer(file)

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
                self.assertEqual(runner.run_movement(self.directory, {}, ANSWER), 101)
        self.assertEqual(lessons.read_text(), ANSWER)
        self.assertEqual(simulation.read_text(), SCHEDULE)
        self.assertEqual(check.call_count, 1)

    def test_schedule_drift_writes_no_solution(self):
        lessons, simulation = self.simulation_files(self.directory)
        simulation.write_text("changed schedule")
        with patch.object(runner.subprocess, "run", return_value=self.formatter_result()):
            with patch.object(runner, "run_checked") as check:
                with self.assertRaises(ValueError):
                    runner.run_movement(self.directory, {}, ANSWER)
        self.assertEqual(lessons.read_text(), STUB)
        check.assert_not_called()

    def test_integrated_failure_stops_before_clippy_and_includes_ignored_tests(self):
        self.simulation_files(self.directory)
        with patch.object(runner.subprocess, "run", return_value=self.formatter_result()):
            with patch.object(runner, "run_checked", side_effect=[0, 101]) as check:
                self.assertEqual(runner.run_movement(self.directory, {}, ANSWER), 101)
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
                    self.assertEqual(examples, ANSWER)
                    (workspace / "crates/moss-sim/src/lessons.rs").write_text("copied edit")
                    raise ValueError("fixture failure after editing the copy")

                with patch.object(runner, "__file__", str(root / "learning/scripts/check_movement_reference.py")):
                    with patch.object(runner.tempfile, "mkdtemp", return_value=str(scratch)):
                        with patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "unexpected-toolchain"}):
                            with patch.object(runner, "run_movement", side_effect=fail_in_copy):
                                args = ["--keep-workspace"] if keep else []
                                self.assertEqual(runner.main(args), 1)
                self.assertEqual(scratch.exists(), keep)
                after = {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}
                self.assertEqual(after, before)

if __name__ == "__main__":
    unittest.main()
