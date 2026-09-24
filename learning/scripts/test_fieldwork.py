"""Persistent-project runner regressions without invoking Cargo.

Fixtures are independent of the in-progress ecosystem reference. These checks
cover selection, copied inputs, acceptance execution, freshness and failure
reporting; they do not establish that a Rust ecosystem implementation is correct.
"""
from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import fieldwork


MANIFEST = '[package]\nname = "fieldwork_fixture"\nversion = "0.1.0"\nedition = "2024"\n\n[workspace]\n'
CANONICAL = b'#[test]\nfn course_contract() { assert_eq!(2 + 2, 4); }\n'


class Fixture(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="moss-fieldwork-tests-")
        self.addCleanup(self.directory.cleanup)
        self.base = Path(self.directory.name)
        self.reference = self.make_project("reference")
        (self.reference / "tests/acceptance.rs").write_bytes(CANONICAL)
        self.project = self.make_project("learner")
        (self.project / "tests/acceptance.rs").write_text("// learner-authored acceptance remains intact\n")
        self.work = self.base / "work"
        for attribute, value in (("REFERENCE", self.reference), ("WORK", self.work)):
            context = patch.object(fieldwork, attribute, value)
            context.start()
            self.addCleanup(context.stop)

    def make_project(self, name):
        project = self.base / name
        (project / "src").mkdir(parents=True)
        (project / "tests").mkdir()
        (project / "Cargo.toml").write_text(MANIFEST)
        (project / "Cargo.lock").write_text("version = 4\n")
        (project / "src/lib.rs").write_text("pub fn reading() -> u32 { 7 }\n")
        return project

    def check_with(self, probe=None, listed="course_contract: test\n",
                   executed="test course_contract ... ok\n"):
        calls = []
        def run(command, package):
            calls.append((command, package))
            self.assertEqual(Path(command[command.index("--target-dir") + 1]), package / "target")
            if probe:
                probe(command, package)
            if "--list" in command:
                return listed
            if "--all-targets" in command:
                return "project test output\n"
            return executed
        output = io.StringIO()
        with patch.object(fieldwork, "run", side_effect=run), redirect_stdout(output):
            report = fieldwork.check(self.project, "LEARNER PROJECT fixture")
        return report, calls, output.getvalue()


class SnapshotInputs(Fixture):
    def test_cli_prepares_the_stable_start_not_the_growing_reference(self):
        root = self.base / "course"
        checkpoint = root / "checkpoints/shared-meadow"
        (checkpoint / "src").mkdir(parents=True)
        (checkpoint / "src/lib.rs").write_text("// stable first arc\n")
        destination = self.base / "new-attempt"
        with patch.object(fieldwork, "ROOT", root), \
                patch("sys.argv", ["fieldwork.py", "prepare", "--dest", str(destination)]), \
                redirect_stdout(io.StringIO()):
            fieldwork.main()
            self.assertEqual((destination / "src/lib.rs").read_text(), "// stable first arc\n")
            (destination / "src/lib.rs").write_text("// my continuing work\n")
            with self.assertRaises(FileExistsError):
                fieldwork.main()
        self.assertEqual((destination / "src/lib.rs").read_text(), "// my continuing work\n")

    def test_snapshot_keeps_nested_test_and_example_support_bytes(self):
        fixtures = {
            "tests/support/world.rs": b"pub const COUNT: u32 = 3;\n",
            "tests/fixtures/input.bin": b"\0\xff\x01",
            "examples/common/report.rs": b"// shared example support\n",
            "rust-toolchain.toml": b'[toolchain]\nchannel = "1.93.1"\n',
        }
        for name, data in fixtures.items():
            target = self.project / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        snapshot = fieldwork.source_files(self.project)
        for name, data in fixtures.items():
            self.assertEqual(snapshot[name], data)

    def test_missing_required_inputs_stop_before_any_cargo_call(self):
        (self.project / "Cargo.lock").unlink()
        with patch.object(fieldwork, "run") as run, self.assertRaisesRegex(ValueError, "Cargo.lock"):
            fieldwork.check(self.project, "LEARNER PROJECT")
        run.assert_not_called()

    def test_symlinked_source_directory_roots_are_rejected(self):
        for name in ("src", "tests", "examples", "benches"):
            with self.subTest(root=name):
                project = self.make_project(f"symlink-{name}")
                source = project / name
                external = self.base / f"outside-{name}"
                if source.exists():
                    source.rename(external)
                else:
                    external.mkdir()
                    (external / "example.rs").write_text("fn main() {}")
                source.symlink_to(external, target_is_directory=True)
                with self.assertRaisesRegex(ValueError, "symlink"):
                    fieldwork.source_files(project)

    def test_external_manifest_targets_and_local_dependencies_are_refused(self):
        outside = self.base / "outside.rs"
        outside.write_text("pub fn outside() {}\n")
        variants = {
            "absolute-lib": f'\n[lib]\npath = {json.dumps(str(outside))}\n',
            "escaping-test": '\n[[test]]\nname = "outside"\npath = "../outside.rs"\n',
            "local-dependency": '\n[dependencies]\nother = { path = "../other" }\n',
            "target-dependency": '\n[target.\'cfg(unix)\'.dependencies]\nother = { path = "../other" }\n',
            "patched-dependency": '\n[patch.crates-io]\nother = { path = "../other" }\n',
        }
        for name, extension in variants.items():
            with self.subTest(shape=name):
                project = self.make_project(name)
                (project / "Cargo.toml").write_text(MANIFEST + extension)
                with self.assertRaises(ValueError):
                    fieldwork.source_files(project)

    def test_workspace_members_and_uncopied_build_paths_are_refused(self):
        variants = {
            "workspace-members": MANIFEST.replace("[workspace]", '[workspace]\nmembers = ["child"]'),
            "custom-build": MANIFEST.replace('edition = "2024"', 'edition = "2024"\nbuild = "tools/generate.rs"'),
            "default-build": MANIFEST,
        }
        for name, manifest in variants.items():
            with self.subTest(shape=name):
                project = self.make_project(name)
                (project / "Cargo.toml").write_text(manifest)
                if name == "default-build":
                    (project / "build.rs").write_text("fn main() {}\n")
                with self.assertRaises(ValueError):
                    fieldwork.source_files(project)

    def test_saved_bench_targets_cannot_silently_disappear_from_snapshot(self):
        target = self.project / "benches/reading.rs"
        target.parent.mkdir()
        target.write_bytes(b"// a saved Cargo-discoverable target\n")
        try:
            snapshot = fieldwork.source_files(self.project)
        except ValueError as error:
            # Explicitly refusing an unsupported target shape is honest too.
            self.assertIn("bench", str(error).lower())
        else:
            self.assertEqual(snapshot.get("benches/reading.rs"), target.read_bytes())


class PersistentPreparation(Fixture):
    def test_prepare_copies_once_and_never_overwrites_an_existing_attempt(self):
        (self.reference / "target").mkdir()
        (self.reference / "target/stale-artifact").write_text("not source")
        (self.reference / ".git").mkdir()
        (self.reference / ".git/config").write_text("not source")
        destination = self.base / "practice"
        with redirect_stdout(io.StringIO()):
            fieldwork.prepare(destination)
            edited = destination / "src/lib.rs"
            self.assertEqual(edited.read_bytes(), (self.reference / "src/lib.rs").read_bytes())
            edited.write_text("// learner's later edit\n")
            with self.assertRaises(FileExistsError):
                fieldwork.prepare(destination)
        self.assertEqual(edited.read_text(), "// learner's later edit\n")
        self.assertFalse((destination / "target").exists())
        self.assertFalse((destination / ".git").exists())


class CheckedSnapshot(Fixture):
    def test_checks_exact_learner_copy_with_separate_injected_acceptance(self):
        (self.project / "src/lib.rs").write_text("pub fn reading() -> u32 { 42 }\n")
        (self.project / "tests/my_observation.rs").write_text("// learner's additional tests\n")
        selected = fieldwork.source_files(self.project)
        packages = []
        def inspect(command, package):
            packages.append(package)
            self.assertEqual((package / "src/lib.rs").read_bytes(), selected["src/lib.rs"])
            self.assertEqual((package / "tests/acceptance.rs").read_bytes(), selected["tests/acceptance.rs"])
            self.assertEqual((package / "tests/my_observation.rs").read_bytes(), selected["tests/my_observation.rs"])
            self.assertEqual((package / "tests/__fieldnotes_acceptance.rs").read_bytes(), CANONICAL)
        report, calls, output = self.check_with(inspect)
        self.assertTrue(report["passed"])
        self.assertEqual(report["sourceHash"], fieldwork.fingerprint(selected))
        self.assertEqual(report["acceptanceHash"], hashlib.sha256(CANONICAL).hexdigest())
        self.assertEqual(report["selection"], "LEARNER PROJECT fixture")
        self.assertEqual(report["requiredTests"], ["course_contract"])
        self.assertIn("--all-targets", calls[-1][0])
        self.assertIn("ignored tests are not verified", output)
        self.assertEqual(fieldwork.source_files(self.project), selected)
        self.assertFalse((self.project / "tests/__fieldnotes_acceptance.rs").exists())
        self.assertTrue(all(not path.exists() for path in packages))

    def test_reserved_acceptance_filename_never_overwrites_learner_data(self):
        reserved = self.project / "tests/__fieldnotes_acceptance.rs"
        reserved.write_text("my work in an accidentally reserved name")
        with patch.object(fieldwork, "run") as run, self.assertRaisesRegex(ValueError, "reserved"):
            fieldwork.check(self.project, "LEARNER PROJECT")
        run.assert_not_called()
        self.assertEqual(reserved.read_text(), "my work in an accidentally reserved name")

    def test_manifest_cannot_redirect_the_reserved_acceptance_target(self):
        manifest = MANIFEST + ('\n[[test]]\nname = "__fieldnotes_acceptance"\n'
                               'path = "tests/acceptance.rs"\n')
        (self.project / "Cargo.toml").write_text(manifest)
        with patch.object(fieldwork, "run") as run, self.assertRaisesRegex(ValueError, "reserved"):
            fieldwork.check(self.project, "LEARNER PROJECT")
        run.assert_not_called()

    def test_missing_discovered_acceptance_cannot_report_success(self):
        for listing in ("", "unrelated_test: test\n"):
            with self.subTest(listing=listing), self.assertRaisesRegex(ValueError, "Missing course checks"):
                self.check_with(listed=listing)

    def test_ignored_acceptance_after_an_attribute_cannot_be_skipped_by_name_parser(self):
        canonical = CANONICAL + b'#[test]\n#[ignore = "unfinished"]\nfn must_run() {}\n'
        (self.reference / "tests/acceptance.rs").write_bytes(canonical)
        with self.assertRaisesRegex(ValueError, "must_run"):
            self.check_with(listed="course_contract: test\nmust_run: test\n",
                            executed="test course_contract ... ok\ntest must_run ... ignored, unfinished\n")

    def test_every_discovered_acceptance_must_pass_even_when_not_parsed_from_source(self):
        # For example, a test generated by a helper macro still appears in --list.
        with self.assertRaisesRegex(ValueError, "generated_check"):
            self.check_with(listed="course_contract: test\ngenerated_check: test\n",
                            executed="test course_contract ... ok\ntest generated_check ... ignored\n")

    def test_should_panic_acceptance_keeps_its_actual_name(self):
        canonical = b'#[test]\n#[should_panic]\nfn course_contract() { panic!("expected"); }\n'
        (self.reference / "tests/acceptance.rs").write_bytes(canonical)
        report, _, _ = self.check_with(executed="test course_contract - should panic ... ok\n")
        self.assertTrue(report["passed"])

    def test_failure_in_learner_targets_prevents_acceptance_success_report(self):
        def fail(command, package):
            if "--all-targets" in command:
                raise subprocess.CalledProcessError(101, command)
        with self.assertRaises(subprocess.CalledProcessError):
            self.check_with(fail)

    def test_saved_input_edit_or_new_test_during_check_invalidates_the_result(self):
        for change in ("edit", "new-test"):
            with self.subTest(change=change):
                def mutate(command, package):
                    if "--all-targets" in command:
                        if change == "edit":
                            (self.project / "src/lib.rs").write_text("pub fn reading() -> u32 { 99 }\n")
                        else:
                            (self.project / "tests/later.rs").write_text("// saved while checking\n")
                with self.assertRaisesRegex(ValueError, "source changed"):
                    self.check_with(mutate)

    def test_changed_course_acceptance_invalidates_the_result_too(self):
        def mutate(command, package):
            if "--all-targets" in command:
                (self.reference / "tests/acceptance.rs").write_bytes(CANONICAL + b"// updated contract\n")
        with self.assertRaisesRegex(ValueError, "acceptance suite changed"):
            self.check_with(mutate)


class CargoBoundary(Fixture):
    def test_cargo_failure_keeps_diagnostic_and_never_turns_into_empty_success(self):
        failed = subprocess.CompletedProcess([], 101, "running learner test\n", "error[E0382]: moved value\n")
        stdout, stderr = io.StringIO(), io.StringIO()
        with patch.object(fieldwork.subprocess, "run", return_value=failed), \
                redirect_stdout(stdout), redirect_stderr(stderr), \
                self.assertRaises(subprocess.CalledProcessError):
            fieldwork.run(["cargo", "test"], self.project)
        self.assertIn("running learner test", stdout.getvalue())
        self.assertIn("error[E0382]", stderr.getvalue())


POPULATION = (b'#[test]\nfn course_contract() {}\n'
              b'#[test]\nfn birth_contract() {}\n')


class MultipleAcceptanceSuites(Fixture):
    def setUp(self):
        super().setUp()
        (self.reference / "tests/population.rs").write_bytes(POPULATION)
        (self.project / "tests/population.rs").write_bytes(b"// learner's own population investigation\n")
        self.calls = []

    def population_run(self, command, package, probe=None, listings=None, executions=None):
        self.calls.append((command, package))
        if probe:
            probe(command, package)
        if "--all-targets" in command:
            return "learner targets passed\n"
        suite = command[command.index("--test") + 1].removeprefix("__fieldnotes_")
        default_names = {"acceptance": ["course_contract"],
                         "population": ["course_contract", "birth_contract"]}
        if "--list" in command:
            return (listings or {}).get(suite, "".join(f"{name}: test\n" for name in default_names[suite]))
        return (executions or {}).get(suite, "".join(f"test {name} ... ok\n" for name in default_names[suite]))

    def check_population(self, probe=None, listings=None, executions=None):
        output = io.StringIO()
        def run(command, package):
            return self.population_run(command, package, probe, listings, executions)
        with patch.object(fieldwork, "run", side_effect=run), redirect_stdout(output):
            report = fieldwork.check(self.project, "LEARNER PROJECT fixture", "population")
        return report, output.getvalue()

    def test_each_suite_is_injected_separately_and_learner_files_stay_intact(self):
        selected = fieldwork.source_files(self.project)
        def inspect(command, package):
            for name, contents in selected.items():
                self.assertEqual((package / name).read_bytes(), contents)
            self.assertEqual((package / "tests/__fieldnotes_acceptance.rs").read_bytes(), CANONICAL)
            if "__fieldnotes_population" in command or "--all-targets" in command:
                self.assertEqual((package / "tests/__fieldnotes_population.rs").read_bytes(), POPULATION)
        report, output = self.check_population(probe=inspect)
        self.assertEqual(fieldwork.source_files(self.project), selected)
        self.assertEqual(report["exercise"], "population")
        self.assertEqual(report["requiredBySuite"], {
            "acceptance": ["course_contract"],
            "population": ["birth_contract", "course_contract"],
        })
        self.assertEqual(report["requiredTests"], [
            "acceptance::course_contract", "population::birth_contract", "population::course_contract",
        ])
        self.assertEqual(report["acceptanceHash"], fieldwork.fingerprint({
            "acceptance": CANONICAL, "population": POPULATION,
        }))
        self.assertIn("3 required acceptance tests passed", output)
        self.assertEqual(len(self.calls), 5)  # List/run each suite, then the learner's targets.
        self.assertTrue(all(not package.exists() for _, package in self.calls))

    def test_shared_meadow_does_not_read_or_inject_population_contract(self):
        # The future API does not even have to exist to check a first-arc copy.
        (self.reference / "tests/population.rs").unlink()
        def inspect(command, package):
            self.assertNotIn("__fieldnotes_population", command)
            self.assertFalse((package / "tests/__fieldnotes_population.rs").exists())
            self.assertEqual((package / "tests/population.rs").read_bytes(),
                             b"// learner's own population investigation\n")
        report, _, _ = self.check_with(probe=inspect)
        self.assertEqual(report["exercise"], "shared-meadow")
        self.assertEqual(report["requiredTests"], ["course_contract"])
        self.assertEqual(report["acceptanceHash"], hashlib.sha256(CANONICAL).hexdigest())

    def test_missing_population_contract_stops_before_any_cargo_execution(self):
        (self.reference / "tests/population.rs").unlink()
        with patch.object(fieldwork, "run") as run, self.assertRaises(FileNotFoundError):
            fieldwork.check(self.project, "LEARNER PROJECT", "population")
        run.assert_not_called()

    def test_empty_or_undiscovered_population_suite_cannot_report_success(self):
        for listing in ("", "course_contract: test\n"):
            with self.subTest(listing=listing), self.assertRaisesRegex(ValueError, "birth_contract"):
                self.check_population(listings={"population": listing})
        (self.reference / "tests/population.rs").write_bytes(b"// no executable checks\n")
        with self.assertRaisesRegex(ValueError, "population suite has no named tests"):
            self.check_population()

    def test_ignored_population_check_is_required_despite_successful_base_suite(self):
        population = POPULATION.replace(b"fn birth_contract", b'#[ignore = "unfinished"]\nfn birth_contract')
        (self.reference / "tests/population.rs").write_bytes(population)
        with self.assertRaisesRegex(ValueError, "birth_contract"):
            self.check_population(executions={"population":
                "test course_contract ... ok\ntest birth_contract ... ignored, unfinished\n"})
        self.assertFalse(any("--all-targets" in command for command, _ in self.calls))

    def test_ignored_discovered_population_check_is_required_without_source_match(self):
        with self.assertRaisesRegex(ValueError, "generated_population_check"):
            self.check_population(listings={"population":
                "course_contract: test\nbirth_contract: test\ngenerated_population_check: test\n"})

    def test_colliding_test_name_failure_identifies_its_suite(self):
        # A base-suite pass of this name must neither mask nor mislabel the
        # same-named population test that failed to execute.
        with self.assertRaisesRegex(ValueError, r"population.*course_contract"):
            self.check_population(executions={"population":
                "test course_contract ... ignored\ntest birth_contract ... ok\n"})

    def test_second_suite_failure_does_not_create_a_passing_cli_report(self):
        destination = self.base / "population-report.json"
        def run(command, package):
            if "__fieldnotes_population" in command and "--list" not in command:
                self.calls.append((command, package))
                raise subprocess.CalledProcessError(101, command)
            return self.population_run(command, package)
        output = io.StringIO()
        with patch.object(fieldwork, "run", side_effect=run), \
                patch("sys.argv", ["fieldwork.py", "check", "--project", str(self.project),
                                   "--stage", "population", "--report", str(destination)]), \
                redirect_stdout(output), self.assertRaises(subprocess.CalledProcessError):
            fieldwork.main()
        self.assertFalse(destination.exists())
        self.assertNotIn("required acceptance tests passed", output.getvalue())
        self.assertFalse(any("--all-targets" in command for command, _ in self.calls))

    def test_both_canonical_files_are_rechecked_after_all_targets(self):
        for name in ("acceptance", "population"):
            with self.subTest(suite=name):
                path = self.reference / f"tests/{name}.rs"
                original = path.read_bytes()
                def mutate(command, package):
                    if "--all-targets" in command:
                        path.write_bytes(original + b"// changed during this check\n")
                with self.assertRaisesRegex(ValueError, "acceptance suite changed"):
                    self.check_population(probe=mutate)
                path.write_bytes(original)

    def test_population_source_is_not_reloaded_halfway_through_the_check(self):
        original = (self.reference / "tests/population.rs").read_bytes()
        def mutate(command, package):
            if "__fieldnotes_acceptance" in command and "--list" not in command:
                (self.reference / "tests/population.rs").write_bytes(original + b"// newer canonical generation\n")
            if "__fieldnotes_population" in command:
                self.assertEqual((package / "tests/__fieldnotes_population.rs").read_bytes(), original)
        with self.assertRaisesRegex(ValueError, "acceptance suite changed"):
            self.check_population(probe=mutate)

    def test_reserved_population_source_never_overwrites_saved_learner_work(self):
        reserved = self.project / "tests/__fieldnotes_population.rs"
        reserved.write_bytes(b"// learner work in an accidentally reserved path\n")
        with self.assertRaisesRegex(ValueError, "__fieldnotes_population.rs.*reserved"):
            self.check_population()
        self.assertEqual(reserved.read_bytes(), b"// learner work in an accidentally reserved path\n")
        self.assertFalse(any("__fieldnotes_population" in command for command, _ in self.calls))

    def test_cargo_target_cannot_redirect_population_checks_to_a_learner_file(self):
        (self.project / "Cargo.toml").write_text(MANIFEST + (
            '\n[[test]]\nname = "__fieldnotes_population"\npath = "tests/population.rs"\n'))
        with patch.object(fieldwork, "run") as run, self.assertRaisesRegex(ValueError, "reserved"):
            fieldwork.check(self.project, "LEARNER PROJECT", "population")
        run.assert_not_called()


class MobileAcceptanceSuites(Fixture):
    def setUp(self):
        super().setUp()
        (self.reference / "tests/population.rs").write_bytes(POPULATION)
        (self.reference / "tests/mobile.rs").write_bytes(b"#[test]\nfn travel_contract() {}\n")

    def test_mobile_requires_all_three_suites_and_preserves_learner_investigations(self):
        selected = fieldwork.source_files(self.project)
        executed = []
        names = {"acceptance": ["course_contract"], "population": ["course_contract", "birth_contract"],
                 "mobile": ["travel_contract"]}
        def run(command, package):
            self.assertEqual(Path(command[command.index("--target-dir") + 1]), package / "target")
            for name, contents in selected.items():
                self.assertEqual((package / name).read_bytes(), contents)
            if "--all-targets" in command:
                return "learner targets passed\n"
            suite = command[command.index("--test") + 1].removeprefix("__fieldnotes_")
            if "--list" in command:
                return "".join(f"{name}: test\n" for name in names[suite])
            executed.append(suite)
            return "".join(f"test {name} ... ok\n" for name in names[suite])
        with patch.object(fieldwork, "run", side_effect=run), redirect_stdout(io.StringIO()):
            report = fieldwork.check(self.project, "LEARNER PROJECT fixture", "mobile")
        self.assertEqual(executed, ["acceptance", "population", "mobile"])
        self.assertEqual(report["exercise"], "mobile")
        self.assertEqual(report["acceptanceHash"], fieldwork.fingerprint(fieldwork.acceptance_suites("mobile")))
        self.assertEqual(report["requiredTests"], ["acceptance::course_contract", "mobile::travel_contract",
                                                   "population::birth_contract", "population::course_contract"])
        self.assertEqual(fieldwork.source_files(self.project), selected)

    def test_missing_mobile_contract_cannot_fall_back_to_population_success(self):
        (self.reference / "tests/mobile.rs").unlink()
        with patch.object(fieldwork, "run") as run, self.assertRaises(FileNotFoundError):
            fieldwork.check(self.project, "LEARNER PROJECT fixture", "mobile")
        run.assert_not_called()


class RestingAcceptanceSuites(Fixture):
    def test_resting_checks_every_prior_contract_and_does_not_accept_ignored_rest(self):
        for name in ("population", "mobile", "rest"):
            (self.reference / f"tests/{name}.rs").write_text(f"#[test]\nfn {name}_contract() {{}}\n")
        executed = []
        def run(command, package):
            if "--all-targets" in command:
                return "learner targets passed\n"
            suite = command[command.index("--test") + 1].removeprefix("__fieldnotes_")
            name = "course_contract" if suite == "acceptance" else f"{suite}_contract"
            if "--list" in command:
                return f"{name}: test\n"
            executed.append(suite)
            return f"test {name} ... {'ignored' if suite == 'rest' else 'ok'}\n"
        with patch.object(fieldwork, "run", side_effect=run), redirect_stdout(io.StringIO()):
            with self.assertRaises(ValueError):
                fieldwork.check(self.project, "LEARNER PROJECT", "resting")
        self.assertEqual(executed, ["acceptance", "population", "mobile", "rest"])


class LaterAcceptanceSuites(Fixture):
    def setUp(self):
        super().setUp()
        for name in ("population", "mobile", "rest", "hunting", "refuge"):
            (self.reference / f"tests/{name}.rs").write_text(
                f"#[test]\nfn {name}_contract() {{}}\n")
        self.saved = fieldwork.source_files(self.project)
        self.executed = []
        self.ignore_refuge = False

    def run_suite(self, command, package):
        for name, contents in self.saved.items():
            self.assertEqual((package / name).read_bytes(), contents)
        if "--all-targets" in command:
            self.executed.append("learner")
            return "learner targets passed\n"
        suite = command[command.index("--test") + 1].removeprefix("__fieldnotes_")
        name = "course_contract" if suite == "acceptance" else f"{suite}_contract"
        if "--list" in command:
            return f"{name}: test\n"
        self.executed.append(suite)
        result = "ignored" if suite == "refuge" and self.ignore_refuge else "ok"
        return f"test {name} ... {result}\n"

    def test_later_stages_keep_prior_contracts_and_actual_learner_files(self):
        for stage in ("hunting", "refuge"):
            with self.subTest(stage=stage):
                self.executed.clear()
                with patch.object(fieldwork, "run", side_effect=self.run_suite), redirect_stdout(io.StringIO()):
                    report = fieldwork.check(self.project, "LEARNER PROJECT", stage)
                suites = ["acceptance", "population", "mobile", "rest", "hunting"]
                if stage == "refuge":
                    suites.append("refuge")
                self.assertEqual(self.executed, suites + ["learner"])
                self.assertEqual(len(report["requiredTests"]), len(suites))
                self.assertEqual(report["exercise"], stage)
                self.assertEqual(fieldwork.source_files(self.project), self.saved)

    def test_ignored_refuge_contract_cannot_pass_on_earlier_successes(self):
        self.ignore_refuge = True
        with patch.object(fieldwork, "run", side_effect=self.run_suite), redirect_stdout(io.StringIO()):
            with self.assertRaises(ValueError):
                fieldwork.check(self.project, "LEARNER PROJECT", "refuge")
        self.assertEqual(self.executed, ["acceptance", "population", "mobile", "rest", "hunting", "refuge"])
        self.assertEqual(fieldwork.source_files(self.project), self.saved)


if __name__ == "__main__":
    unittest.main()
