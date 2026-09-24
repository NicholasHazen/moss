"""Keep optional runtime failures separate from static reading availability."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import build
import runtime


class RuntimeBuildBoundary(unittest.TestCase):
    def test_source_change_during_compile_preserves_previous_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            root = repository / "learning"
            work = root / "work"
            files = {
                "Cargo.toml": "workspace fixture",
                "rust-toolchain.toml": "pinned fixture",
                "learning/runtime/Cargo.toml": "runtime fixture",
                "learning/runtime/Cargo.lock": "locked fixture",
                "learning/runtime/src/lib.rs": "fn host() {}",
                "crates/moss-sim/Cargo.toml": "simulation fixture",
                "crates/moss-sim/src/lib.rs": "const RATE: u32 = 1;",
            }
            for name, text in files.items():
                path = repository / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            output = work / "runtime"
            output.mkdir(parents=True)
            (output / "moss.wasm").write_bytes(b"previous accepted binary")
            (output / "build.json").write_text("previous accepted metadata")

            def compile_then_save(command, cwd):
                self.assertIn("--offline", command)
                self.assertIn("--locked", command)
                self.assertEqual(cwd, root / "runtime")
                artifact = work / "target/wasm32-unknown-unknown/release/moss_fieldnotes_runtime.wasm"
                artifact.parent.mkdir(parents=True)
                artifact.write_bytes(b"compiled before the edit")
                (repository / "crates/moss-sim/src/lib.rs").write_text("const RATE: u32 = 2;")

            with patch.object(runtime, "ROOT", root), patch.object(runtime, "WORK", work), \
                    patch.object(runtime, "run", side_effect=compile_then_save), \
                    patch("sys.argv", ["runtime.py", "build"]):
                with self.assertRaisesRegex(SystemExit, "inputs changed during compilation"):
                    runtime.main()
            self.assertEqual((output / "moss.wasm").read_bytes(), b"previous accepted binary")
            self.assertEqual((output / "build.json").read_text(), "previous accepted metadata")

    def make_static_fixture(self, repository):
        root = repository / "learning"
        (root / "assets").mkdir(parents=True)
        (root / "content").mkdir()
        (root / "work/runtime").mkdir(parents=True)
        (repository / "docs/tutorial").mkdir(parents=True)
        (repository / "docs/tutorial/today-v2.md").write_text("Review fixture reference")
        course = {"edition": "Review fixture", "description": "Review fixture", "modules": [{
            "id": "01-example", "title": "Example", "part": "Review", "minutes": 1,
            "concepts": ["review"], "summary": "Review",
        }]}
        (root / "course.json").write_text(json.dumps(course))
        for name in ("01-example", "index", "roadmap", "setup", "about", "shelf", "terrarium"):
            (root / "content" / f"{name}.html").write_text("<p>Static explanation remains readable.</p>")
        (root / "work/runtime/moss.wasm").write_bytes(b"\0asm\x01\0\0\0")
        return root

    def test_bad_metadata_omits_runtime_but_builds_static_lessons(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.make_static_fixture(Path(directory))
            output = root / "_site"
            cases = [
                ("current", '{"api":1,"sourceHash":"current"}', True),
                ("stale", '{"api":1,"sourceHash":"old"}', False),
                ("truncated", '{"api":1,', False),
                ("missing hash", '{"api":1}', False),
                ("array", '[]', False),
                ("unknown API", '{"api":99,"sourceHash":"current"}', False),
            ]
            with patch.object(build, "ROOT", root), patch.object(build, "OUT", output), \
                    patch.object(build, "runtime_fingerprint", return_value="current"), \
                    contextlib.redirect_stdout(io.StringIO()):
                for name, metadata, expected_runtime in cases:
                    with self.subTest(name=name):
                        (root / "work/runtime/build.json").write_text(metadata)
                        build.build()
                        self.assertTrue((output / "01-example.html").exists())
                        self.assertTrue((output / "index.html").exists())
                        self.assertEqual((output / "runtime/moss.wasm").exists(), expected_runtime)
                        if expected_runtime:
                            self.assertEqual((output / "runtime/moss.wasm").read_bytes(), b"\0asm\x01\0\0\0")

    def test_unreadable_metadata_keeps_static_build_available(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.make_static_fixture(Path(directory))
            metadata = root / "work/runtime/build.json"
            metadata.write_text('{"api":1,"sourceHash":"current"}')
            read_text = Path.read_text

            def deny_metadata(path, *args, **kwargs):
                if path == metadata:
                    raise PermissionError("Review fixture denies metadata read")
                return read_text(path, *args, **kwargs)

            with patch.object(build, "ROOT", root), patch.object(build, "OUT", root / "_site"), \
                    patch.object(build, "runtime_fingerprint", return_value="current"), \
                    patch.object(Path, "read_text", deny_metadata), \
                    contextlib.redirect_stdout(io.StringIO()):
                build.build()
            self.assertTrue((root / "_site/terrarium.html").exists())
            self.assertFalse((root / "_site/runtime").exists())


if __name__ == "__main__":
    unittest.main()
