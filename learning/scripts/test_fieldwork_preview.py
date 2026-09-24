"""Preview orchestration checks with temporary projects and mocked Cargo.

These tests establish input/artifact binding and publication failure behavior.
They do not establish that the Rust host compiles or that a browser renders it.
"""
from contextlib import redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import fieldwork
import fieldwork_preview as preview


MANIFEST = '[package]\nname = "meadow"\nversion = "0.1.0"\nedition = "2024"\n[workspace]\n'
LOCK = b'version = 4\n\n[[package]]\nname = "meadow"\nversion = "0.1.0"\n'
HOST = b'#[cfg(test)]\nmod tests {\n#[test]\nfn reads_do_not_step() {}\n#[test]\nfn invalid_scalar_index() {}\n}\n'
CANONICAL = b'#[test]\nfn course_contract() {}\n'
LISTED = 'tests::reads_do_not_step: test\ntests::invalid_scalar_index: test\n'
PASSED = 'test tests::reads_do_not_step ... ok\ntest tests::invalid_scalar_index ... ok\n'
BINARY = b'\x00asm-exact-private-build-fixture'


class Fixture(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="moss-preview-tests-")
        self.addCleanup(self.directory.cleanup)
        self.base = Path(self.directory.name)
        self.root = self.base / "learning"
        (self.root / "preview").mkdir(parents=True)
        (self.root / "preview/ecosystem-host.rs").write_bytes(HOST)
        (self.root / "preview/population-host.rs").write_bytes(HOST + b"// population adapter\n")
        (self.root / "preview/mobile-host.rs").write_bytes(HOST + b"// mobile adapter\n")
        self.reference = self.make_project("reference")
        (self.reference / "tests/acceptance.rs").write_bytes(CANONICAL)
        (self.reference / "tests/population.rs").write_bytes(b"#[test]\nfn population_contract() {}\n")
        (self.reference / "tests/mobile.rs").write_bytes(b"#[test]\nfn travel_contract() {}\n")
        self.project = self.make_project("learner")
        (self.project / "tests/my_test.rs").write_bytes(b"// learner's own test\n")
        self.work = self.base / "work"
        self.work.mkdir()
        self.output = self.base / "site"
        for module, name, value in (
            (preview, "ROOT", self.root), (preview, "WORK", self.work),
            (fieldwork, "REFERENCE", self.reference),
        ):
            context = patch.object(module, name, value)
            context.start()
            self.addCleanup(context.stop)
        self.calls = []

    def make_project(self, name):
        project = self.base / name
        (project / "src").mkdir(parents=True)
        (project / "tests").mkdir()
        (project / "Cargo.toml").write_text(MANIFEST)
        (project / "Cargo.lock").write_bytes(LOCK)
        (project / "src/lib.rs").write_bytes(b"pub fn amount() -> u32 { 2 }\n")
        return project

    def checked(self, project, label, stage="shared-meadow"):
        return {"passed": True, "sourceHash": preview.inputs(project)[-2],
                "acceptanceHash": hashlib.sha256(CANONICAL).hexdigest(),
                "requiredTests": ["course_contract"]}

    def build(self, probe=None, listed=LISTED, executed=PASSED, check=None,
              project="learner", mode="ecosystem"):
        def run(command, package):
            self.calls.append((command, package))
            if probe:
                probe(command, package)
            if "--list" in command:
                return listed
            if "test" in command:
                return executed
            self.assertIn("build", command)
            target = Path(command[command.index("--target-dir") + 1])
            binary = target / "wasm32-unknown-unknown/release/fieldnotes_ecosystem_host.wasm"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(BINARY)
            return ""
        with patch.object(fieldwork, "check", side_effect=check or self.checked), \
                patch.object(fieldwork, "run", side_effect=run), redirect_stdout(io.StringIO()):
            preview.build(self.project if project == "learner" else None, mode)

    def metadata(self, mode="ecosystem"):
        return json.loads((self.work / "previews" / mode / "build.json").read_text())

    def assert_omitted(self, mode="ecosystem"):
        # An obsolete previously published preview must disappear as well.
        stale = self.output / "previews" / mode / "old.wasm"
        stale.parent.mkdir(parents=True, exist_ok=True)
        stale.write_bytes(b"obsolete")
        preview.copy_current(self.output, mode)
        self.assertFalse(stale.parent.exists())


class ExactBuild(Fixture):
    def test_build_copies_selected_sources_and_lock_into_private_package(self):
        selected = fieldwork.source_files(self.project)
        shared = self.work / "fieldwork-target/wasm32-unknown-unknown/release/fieldnotes_ecosystem_host.wasm"
        shared.parent.mkdir(parents=True)
        shared.write_bytes(b"unrelated concurrent artifact")
        def inspect(command, package):
            for name, data in selected.items():
                self.assertEqual((package / "course" / name).read_bytes(), data)
            self.assertEqual((package / "src/lib.rs").read_bytes(), HOST)
            expected_lock = LOCK + (b'\n[[package]]\nname = "fieldnotes_ecosystem_host"\n'
                                    b'version = "0.1.0"\ndependencies = ["meadow"]\n')
            self.assertEqual((package / "Cargo.lock").read_bytes(), expected_lock)
            self.assertIn('exclude = ["course"]', (package / "Cargo.toml").read_text())
            self.assertIn("--offline", command)
            self.assertIn("--locked", command)
            self.assertEqual(Path(command[command.index("--target-dir") + 1]), package / "target")
        self.build(probe=inspect)
        metadata = self.metadata()
        self.assertEqual(metadata["sourceKind"], "learner")
        self.assertEqual(metadata["sourcePath"], str(self.project.resolve()))
        self.assertEqual(metadata["sourceHash"], fieldwork.fingerprint(selected))
        self.assertEqual(metadata["acceptanceHash"], hashlib.sha256(CANONICAL).hexdigest())
        self.assertEqual(metadata["requiredTests"], ["course_contract"])
        artifact = self.work / "previews/ecosystem" / metadata["wasm"]
        self.assertEqual(artifact.read_bytes(), BINARY)
        self.assertEqual(metadata["wasmHash"], hashlib.sha256(BINARY).hexdigest())
        self.assertEqual(fieldwork.source_files(self.project), selected)
        self.assertTrue(all(not package.exists() for _, package in self.calls))

    def test_reference_build_is_labeled_without_learner_path(self):
        self.build(project="reference")
        metadata = self.metadata()
        self.assertEqual(metadata["sourceKind"], "reference")
        self.assertIsNone(metadata["sourcePath"])
        self.assertTrue(preview.current(metadata))

    def test_failed_course_check_never_starts_host_build(self):
        def reject(project, label):
            raise ValueError("learner acceptance failed")
        with self.assertRaisesRegex(ValueError, "acceptance failed"):
            self.build(check=reject)
        self.assertEqual(self.calls, [])
        self.assertFalse((self.work / "previews/ecosystem/build.json").exists())

    def test_mismatched_checked_source_hash_cannot_publish(self):
        def wrong_report(project, label):
            return dict(self.checked(project, label), sourceHash="0" * 64)
        with self.assertRaisesRegex(ValueError, "changed during checking"):
            self.build(check=wrong_report)
        self.assertEqual(self.calls, [])

    def test_source_or_adapter_edit_during_check_stops_before_compilation(self):
        for relative in ("src/lib.rs", "host"):
            with self.subTest(changed=relative):
                self.calls.clear()
                def edit(project, label):
                    report = self.checked(project, label)
                    path = (self.root / "preview/ecosystem-host.rs" if relative == "host"
                            else self.project / relative)
                    path.write_bytes(path.read_bytes() + b"// saved during check\n")
                    return report
                with self.assertRaisesRegex(ValueError, "changed during checking"):
                    self.build(check=edit)
                self.assertEqual(self.calls, [])

    def test_input_edit_during_compilation_preserves_previous_build_record(self):
        self.build()
        prior = (self.work / "previews/ecosystem/build.json").read_bytes()
        def edit(command, package):
            if "build" in command:
                (self.project / "src/lib.rs").write_bytes(b"pub fn amount() -> u32 { 99 }\n")
        with self.assertRaisesRegex(ValueError, "changed during compilation"):
            self.build(probe=edit)
        self.assertEqual((self.work / "previews/ecosystem/build.json").read_bytes(), prior)
        self.assertFalse(preview.current(self.metadata()))


class NativeEvidence(Fixture):
    def test_missing_native_checks_are_not_success(self):
        for listed in ("", "tests::unrelated: test\n"):
            with self.subTest(listing=listed), self.assertRaisesRegex(ValueError, "checks are missing"):
                self.build(listed=listed)
        self.assertFalse((self.work / "previews/ecosystem/build.json").exists())

    def test_ignored_or_unexecuted_native_check_cannot_publish(self):
        for output in ("test tests::reads_do_not_step ... ok\n",
                       "test tests::reads_do_not_step ... ok\ntest tests::invalid_scalar_index ... ignored\n"):
            with self.subTest(output=output), self.assertRaisesRegex(ValueError, "did not execute"):
                self.build(executed=output)
        self.assertFalse(any("build" in command for command, _ in self.calls))

    def test_discovered_checks_beyond_source_regex_are_also_required(self):
        with self.assertRaisesRegex(ValueError, "did not execute"):
            self.build(listed=LISTED + "tests::generated: test\n")

    def test_should_panic_output_is_a_real_named_success(self):
        self.build(executed=PASSED.replace("tests::invalid_scalar_index ...", "tests::invalid_scalar_index - should panic ..."))
        self.assertTrue(preview.current(self.metadata()))

    def test_host_compile_failure_preserves_previous_published_record(self):
        self.build()
        prior = (self.work / "previews/ecosystem/build.json").read_bytes()
        def fail(command, package):
            if "build" in command:
                raise subprocess.CalledProcessError(101, command)
        with self.assertRaises(subprocess.CalledProcessError):
            self.build(probe=fail)
        self.assertEqual((self.work / "previews/ecosystem/build.json").read_bytes(), prior)


class PopulationStage(Fixture):
    def test_population_build_requires_population_stage_and_uses_its_host(self):
        checks = []
        def checked(project, label, stage):
            checks.append(stage)
            return self.checked(project, label, stage)
        def inspect(command, package):
            self.assertEqual((package / "src/lib.rs").read_bytes(), HOST + b"// population adapter\n")
        self.build(mode="population", check=checked, probe=inspect)
        self.assertEqual(checks, ["population"])
        metadata = self.metadata("population")
        self.assertEqual(metadata["mode"], "population")
        self.assertEqual(metadata["wasm"], f"population-{metadata['buildHash']}.wasm")
        self.assertTrue(preview.current(metadata, "population"))
        self.assertFalse(preview.current(metadata))

    def test_population_failure_cannot_publish_or_start_adapter(self):
        def reject(project, label, stage):
            self.assertEqual(stage, "population")
            raise ValueError("population acceptance failed")
        with self.assertRaisesRegex(ValueError, "population acceptance failed"):
            self.build(mode="population", check=reject)
        self.assertFalse(self.calls)
        self.assertFalse((self.work / "previews/population/build.json").exists())

    def test_each_population_contract_and_host_change_invalidates_its_build(self):
        for path in (self.reference / "tests/acceptance.rs", self.reference / "tests/population.rs",
                     self.root / "preview/population-host.rs"):
            with self.subTest(changed=path):
                self.build(mode="population")
                before = path.read_bytes()
                path.write_bytes(before + b"// changed\n")
                self.assertFalse(preview.current(self.metadata("population"), "population"))
                self.assert_omitted("population")
                path.write_bytes(before)

    def test_modes_publish_independently_and_population_edits_do_not_stale_meadow(self):
        self.build()
        self.build(mode="population")
        preview.copy_current(self.output)
        preview.copy_current(self.output, "population")
        for mode in ("ecosystem", "population"):
            self.assertEqual((self.output / "previews" / mode / self.metadata(mode)["wasm"]).read_bytes(), BINARY)
        path = self.reference / "tests/population.rs"
        path.write_bytes(path.read_bytes() + b"// changed\n")
        self.assertTrue(preview.current(self.metadata()))
        self.assert_omitted("population")
        self.assertTrue((self.output / "previews/ecosystem/build.json").exists())


class MobileStage(Fixture):
    def test_mobile_uses_its_host_and_requires_the_cumulative_mobile_stage(self):
        stages = []
        def checked(project, label, stage):
            stages.append(stage)
            return self.checked(project, label, stage)
        def inspect(command, package):
            self.assertEqual((package / "src/lib.rs").read_bytes(), HOST + b"// mobile adapter\n")
        self.build(mode="mobile", check=checked, probe=inspect)
        self.assertEqual(stages, ["mobile"])
        metadata = self.metadata("mobile")
        self.assertEqual(metadata["mode"], "mobile")
        self.assertTrue(preview.current(metadata, "mobile"))
        self.assertFalse(preview.current(metadata, "population"))

    def test_mobile_binds_all_three_contracts_without_invalidating_prior_modes(self):
        self.build(mode="population")
        for name in ("acceptance", "population", "mobile"):
            path = self.reference / f"tests/{name}.rs"
            with self.subTest(contract=name):
                self.build(mode="mobile")
                original = path.read_bytes()
                path.write_bytes(original + b"// changed requirement\n")
                self.assertFalse(preview.current(self.metadata("mobile"), "mobile"))
                self.assert_omitted("mobile")
                if name == "mobile":
                    self.assertTrue(preview.current(self.metadata("population"), "population"))
                path.write_bytes(original)


class RestingStage(Fixture):
    def test_rest_requirement_invalidates_only_the_later_preview(self):
        (self.root / "preview/resting-host.rs").write_bytes(HOST + b"// resting adapter\n")
        contract = self.reference / "tests/rest.rs"
        contract.write_bytes(b"#[test]\nfn commitment_contract() {}\n")
        self.build(mode="mobile")
        stages = []
        def checked(project, label, stage):
            stages.append(stage)
            return self.checked(project, label, stage)
        self.build(mode="resting", check=checked)
        self.assertEqual(stages, ["resting"])
        self.assertTrue(preview.current(self.metadata("resting"), "resting"))
        contract.write_bytes(contract.read_bytes() + b"// revised requirement\n")
        self.assertFalse(preview.current(self.metadata("resting"), "resting"))
        self.assertTrue(preview.current(self.metadata("mobile"), "mobile"))
        self.assert_omitted("resting")


class Publication(Fixture):
    def test_current_copy_preserves_exact_verified_artifact_and_metadata(self):
        self.build()
        preview.copy_current(self.output)
        metadata = self.metadata()
        destination = self.output / "previews/ecosystem"
        self.assertEqual((destination / metadata["wasm"]).read_bytes(), BINARY)
        self.assertEqual(json.loads((destination / "build.json").read_text()), metadata)

    def test_source_lock_adapter_and_canonical_changes_all_invalidate_preview(self):
        paths = [self.project / "src/lib.rs", self.project / "Cargo.lock",
                 self.root / "preview/ecosystem-host.rs", self.reference / "tests/acceptance.rs"]
        for path in paths:
            with self.subTest(changed=path.name):
                self.build()
                original = path.read_bytes()
                path.write_bytes(original + b"\n// later saved change\n")
                self.assertFalse(preview.current(self.metadata()))
                self.assert_omitted()
                path.write_bytes(original)

    def test_changed_artifact_or_filename_is_omitted(self):
        self.build()
        metadata = self.metadata()
        (self.work / "previews/ecosystem" / metadata["wasm"]).write_bytes(b"other build")
        self.assert_omitted()
        self.build()
        path = self.work / "previews/ecosystem/build.json"
        metadata = self.metadata()
        metadata["wasm"] = "unbound-artifact.wasm"
        path.write_text(json.dumps(metadata))
        self.assert_omitted()

    def test_missing_learner_project_and_malformed_metadata_fail_closed(self):
        self.build()
        shutil.rmtree(self.project)
        self.assert_omitted()
        path = self.work / "previews/ecosystem/build.json"
        for invalid in ("{", "null", "[]", '{"api": 1, "sourceKind": "learner"}'):
            with self.subTest(metadata=invalid):
                path.write_text(invalid)
                self.assert_omitted()


if __name__ == "__main__":
    unittest.main()
