"""Runner boundary regressions; these tests do not invoke Cargo.

The production extractor, composer, classifier, evaluator and CLI are exercised
at their respective boundaries. Rust execution is simulated, and temporary files
are isolated. Real Rust candidate checks are separate evidence, not implied here.
"""
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from build import Fragment
from labs import Examples
import evidence_practice as practice


NAMES = ["learner_tests::current_value", "learner_tests::observed_value"]


def result(state, names=None, failed=None, output="diagnostic detail"):
    return {"state": state, "tests": NAMES.copy() if names is None else names,
            "failed": [] if failed is None else failed, "output": output}


class HarnessClassification(unittest.TestCase):
    def test_success_requires_every_discovered_test_to_finish(self):
        complete = "\n".join(f"test {name} ... ok" for name in NAMES)
        self.assertEqual(practice.classify(0, complete, set(NAMES)), "passed")
        self.assertEqual(practice.classify(0, complete.splitlines()[0], set(NAMES)), "incomplete")
        self.assertEqual(practice.classify(0, "", set()), "no-tests")

    def test_ignored_named_tests_cannot_complete_the_exercise(self):
        for suffix in ("ignored", "ignored, optional"):
            with self.subTest(suffix=suffix):
                output = (f"test {NAMES[0]} ... ok\n"
                          f"test {NAMES[1]} ... {suffix}\n")
                self.assertEqual(practice.classify(0, output, set(NAMES)), "incomplete")

    def test_standard_should_panic_decoration_retains_the_actual_test_name(self):
        # Observed with the pinned Rust 1.93.1 native test harness.
        name = "learner_tests::expected_panic"
        output = f"test {name} - should panic ... ok\n"
        self.assertEqual(practice.classify(0, output, {name}), "passed")
        failed = f"test {name} - should panic ... FAILED\n"
        self.assertEqual(practice.classify(101, failed, {name}), "failed")
        self.assertEqual(practice.outcomes(failed)["FAILED"], {name})

    def test_process_failure_without_a_named_failed_test_is_not_a_caught_defect(self):
        output = "\n".join(f"test {name} ... ok" for name in NAMES)
        self.assertEqual(practice.classify(101, output, set(NAMES)), "execution-error")
        self.assertEqual(practice.classify(101, "error: could not execute test", set(NAMES)), "incomplete")
        failed = f"test {NAMES[0]} ... FAILED\ntest {NAMES[1]} ... ok\n"
        self.assertEqual(practice.classify(101, failed, set(NAMES)), "failed")


class SubprocessBoundary(unittest.TestCase):
    def package(self, directory):
        path = Path(directory)
        (path / "src").mkdir()
        return path

    def test_build_failure_stops_before_execution_and_preserves_the_diagnostic(self):
        listing = subprocess.CompletedProcess([], 101, "", "error: unavailable cached crate")
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(practice.subprocess, "run", return_value=listing) as run:
            checked = practice.execute(self.package(directory), "exact learner source")
        self.assertEqual(checked["state"], "build-error")
        self.assertEqual(checked["tests"], [])
        self.assertIn("unavailable cached crate", checked["output"])
        self.assertEqual(run.call_count, 1)

    def test_zero_learner_tests_does_not_execute_unrelated_tests(self):
        listing = subprocess.CompletedProcess([], 0, "tests::canonical_only: test\n", "")
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(practice.subprocess, "run", return_value=listing) as run:
            checked = practice.execute(self.package(directory), "unrelated test source")
        self.assertEqual(checked["state"], "no-tests")
        self.assertEqual(run.call_count, 1)

    def test_execution_uses_exact_source_and_reports_the_named_failure(self):
        source = "// distinctive learner assertions, not the canonical test suffix\n"
        listing = subprocess.CompletedProcess([], 0, "\n".join(f"{n}: test" for n in NAMES), "")
        execution = subprocess.CompletedProcess(
            [], 101, f"test {NAMES[0]} ... ok\ntest {NAMES[1]} ... FAILED\n", "assertion context")
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(practice.subprocess, "run", side_effect=[listing, execution]) as run:
            package = self.package(directory)
            checked = practice.execute(package, source)
            self.assertEqual((package / "src/main.rs").read_text(), source)
        self.assertEqual(checked["state"], "failed")
        self.assertEqual(checked["tests"], NAMES)
        self.assertEqual(checked["failed"], [NAMES[1]])
        self.assertIn("assertion context", checked["output"])
        self.assertIn("learner_tests::", run.call_args_list[1].args[0])


class PracticeEvaluation(unittest.TestCase):
    def evaluate_with(self, results):
        """Use the real six mutations, isolated files, and a controlled executor."""
        outputs = iter(results)
        sources = []
        def executor(package, source):
            self.assertTrue((package / "Cargo.lock").is_file())
            sources.append(source)
            return next(outputs)
        console = io.StringIO()
        learner = "#[test]\nfn my_own_test() { assert_eq!(percentage(3, 8), Some(37)); }"
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(practice, "WORK", Path(directory)), redirect_stdout(console):
            report = practice.evaluate(learner, "LEARNER TESTS fixture.rs", executor=executor)
        self.assertTrue(all(learner in source for source in sources))
        return report, console.getvalue(), sources

    def test_rejecting_correct_program_stops_before_any_mutant_is_counted(self):
        for state in ("failed", "build-error", "no-tests", "incomplete", "execution-error"):
            with self.subTest(state=state):
                report, console, sources = self.evaluate_with([result(state, failed=[NAMES[0]])])
                self.assertFalse(report["complete"])
                self.assertEqual(len(sources), 1)
                self.assertEqual(report["cases"][0]["state"], state)
                self.assertIn("Counterexamples were not run", console)

    def test_all_green_but_weak_tests_are_incomplete_and_receive_survivor_hints(self):
        report, console, sources = self.evaluate_with([result("passed") for _ in range(7)])
        self.assertFalse(report["complete"])
        self.assertEqual([c["id"] for c in report["cases"]],
                         ["reference", "rounding", "overflow", "selection", "omitted", "order", "stale"])
        self.assertTrue(all(c["state"] == "survived" for c in report["cases"][1:]))
        self.assertEqual(len(set(sources)), 7)
        self.assertIn("fractional", console)
        self.assertIn("Strengthen the cases that survived", console)

    def test_complete_requires_the_correct_reference_and_six_named_rejections(self):
        outcomes = [result("passed")] + [result("failed", failed=[NAMES[1]]) for _ in range(6)]
        report, console, _ = self.evaluate_with(outcomes)
        self.assertTrue(report["complete"])
        self.assertEqual(report["selection"], "LEARNER TESTS fixture.rs")
        self.assertTrue(all(c["state"] == "caught" and c["failed"] == [NAMES[1]]
                            for c in report["cases"][1:]))
        self.assertIn("not universal correctness or live Moss behavior", console)

    def test_mutant_check_errors_are_preserved_and_never_credited_as_caught(self):
        for state in ("build-error", "no-tests", "incomplete", "execution-error"):
            with self.subTest(state=state):
                outcomes = [result("passed"), result(state, names=[])]
                outcomes += [result("failed", failed=[NAMES[0]]) for _ in range(5)]
                report, console, _ = self.evaluate_with(outcomes)
                self.assertFalse(report["complete"])
                self.assertEqual(report["cases"][1]["state"], state)
                self.assertIn("Resolve check errors", console)
                self.assertNotIn("Strengthen the cases that survived", console)

    def test_changed_discovered_tests_cannot_make_a_mutant_look_caught(self):
        outcomes = [result("passed"), result("failed", names=[NAMES[0]], failed=[NAMES[0]])]
        outcomes += [result("failed", failed=[NAMES[0]]) for _ in range(5)]
        report, console, _ = self.evaluate_with(outcomes)
        self.assertFalse(report["complete"])
        self.assertEqual(report["cases"][1]["state"], "test-set-changed")
        self.assertIn("Resolve check errors", console)


class PracticeSourceSelection(unittest.TestCase):
    def test_html_practice_blocks_decode_identically_for_runner_and_downloads(self):
        markup = ('<pre><code data-practice-starter="example-start">'
                  '#[test]\nfn own() { assert_eq!(&amp;3, &amp;3); }'
                  '</code></pre><pre><code data-practice-answer="example-answer">'
                  '#[test]\nfn own_answer() { assert!(3 &lt; 8); }</code></pre>')
        runner, builder = Examples(), Fragment()
        runner.feed(markup)
        builder.feed(markup)
        self.assertEqual(set(runner.blocks), {("data-practice-starter", "example-start"),
                                             ("data-practice-answer", "example-answer")})
        for (_, name), source in runner.blocks.items():
            self.assertEqual(builder.examples[name], source)
        self.assertIn("assert_eq!(&3, &3)", builder.examples["example-start"])
        self.assertIn("assert!(3 < 8)", builder.examples["example-answer"])

    def test_cli_checks_selected_learner_assertions_and_keeps_report_incomplete(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            tests = path / "mine.rs"
            tests.write_text("#[test]\nfn learner_named_test() { assert!(true); }\n")
            report_path = path / "outcome.json"
            report = {"complete": False, "cases": [{"id": "order", "state": "survived"}]}
            with patch.object(practice, "examples", return_value={}), \
                    patch.object(practice, "evaluate", return_value=report) as evaluate, \
                    patch.object(practice.sys, "argv", ["evidence_practice.py", "check", "--file", str(tests),
                                                       "--report", str(report_path)]), redirect_stdout(io.StringIO()):
                exit_code = practice.main()
            evaluate.assert_called_once_with(tests.read_text(), f"LEARNER TESTS {tests.resolve()}")
            self.assertEqual(exit_code, 1)
            self.assertEqual(json.loads(report_path.read_text()), report)

    def test_prepare_preserves_existing_learner_work(self):
        with tempfile.TemporaryDirectory() as directory:
            dest = Path(directory) / "practice"
            source = "#[test]\nfn starter() {}\n"
            blocks = {("data-practice-starter", "evidence-tests-starter"): source}
            with patch.object(practice, "examples", return_value=blocks), \
                    patch.object(practice.sys, "argv", ["evidence_practice.py", "prepare", "--dest", str(dest)]), \
                    redirect_stdout(io.StringIO()):
                self.assertEqual(practice.main(), 0)
                learner_file = dest / "evidence_tests.rs"
                self.assertEqual(learner_file.read_text(), source)
                learner_file.write_text("my later work")
                with self.assertRaises(FileExistsError):
                    practice.main()
            self.assertEqual(learner_file.read_text(), "my later work")


if __name__ == "__main__":
    unittest.main()
