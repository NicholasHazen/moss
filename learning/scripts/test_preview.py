"""Never publish an unchecked or stale learner preview."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import preview


class PreviewBoundary(unittest.TestCase):
    def test_another_build_cannot_replace_the_private_compilation_result(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            data = ("source A", "reference", "host", "manifest", b"lock", "a" * 64)
            def compile_with_competitor(command, package):
                self.assertIn("--target-dir", command)
                target = Path(command[command.index("--target-dir") + 1])
                self.assertEqual(target.parent, package)
                local = target / "wasm32-unknown-unknown/release/moss_fieldnote_lab.wasm"
                local.parent.mkdir(parents=True)
                local.write_bytes(b"checked A")
                shared = work / "target/wasm32-unknown-unknown/release/moss_fieldnote_lab.wasm"
                shared.parent.mkdir(parents=True)
                shared.write_bytes(b"concurrent unrelated B")
            with patch.object(preview,"WORK",work), patch.object(preview,"inputs",return_value=data), \
                    patch.object(preview,"check"), patch.object(preview,"run",side_effect=compile_with_competitor):
                preview.build()
            output = work / "previews/evidence"
            metadata = json.loads((output / "build.json").read_text())
            self.assertEqual((output / metadata["wasm"]).read_bytes(), b"checked A")
            self.assertEqual(sorted(p.name for p in output.iterdir()), ["build.json", "evidence-" + "a" * 64 + ".wasm"])

    def test_failed_check_preserves_accepted_build(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            output = work / "previews/evidence"
            output.mkdir(parents=True)
            (output / "build.json").write_text("accepted metadata")
            data = ("source", "reference", "host", "manifest", b"lock", "a" * 64)
            with patch.object(preview, "WORK", work), patch.object(preview, "inputs", return_value=data), \
                    patch.object(preview, "check", side_effect=ValueError("Required tests did not pass")), \
                    patch.object(preview, "run") as compile_program:
                with self.assertRaisesRegex(ValueError, "Required tests"):
                    preview.build()
                compile_program.assert_not_called()
            self.assertEqual((output / "build.json").read_text(), "accepted metadata")

    def test_changed_source_during_compile_is_not_published(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            before = ("source", "reference", "host", "manifest", b"lock", "a" * 64)
            after = (*before[:-1], "b" * 64)
            with patch.object(preview, "WORK", work), patch.object(preview, "inputs", side_effect=[before, after]), \
                    patch.object(preview, "check"), patch.object(preview, "run"):
                with self.assertRaisesRegex(ValueError, "inputs changed"):
                    preview.build()
            self.assertFalse((work / "previews/evidence/build.json").exists())

    def test_invalid_or_stale_metadata_cannot_be_current(self):
        data = ("source", "reference", "host", "manifest", b"lock", "a" * 64)
        with patch.object(preview, "inputs", return_value=data):
            for invalid in ([], {}, {"api": 1, "sourceKind": "learner", "sourceHash": "a" * 64},
                            {"api": 1, "sourceKind": "reference", "sourceHash": "old"}):
                self.assertFalse(preview.current(invalid))
            self.assertTrue(preview.current({"api": 1, "sourceKind": "reference", "sourceHash": "a" * 64}))

    def test_missing_learner_file_does_not_break_static_build(self):
        metadata = {"api": 1, "sourceKind": "learner", "sourcePath": "removed.rs", "sourceHash": "a" * 64}
        with patch.object(preview, "inputs", side_effect=FileNotFoundError()):
            self.assertFalse(preview.current(metadata))


if __name__ == "__main__":
    unittest.main()
