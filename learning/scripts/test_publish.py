import io
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

import publish


def zipped(files):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in files.items():
            archive.writestr(name, data)
    return buffer.getvalue()


class PublicExport(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.root = self.directory / "learning"
        self.site = self.root / "_site"
        self.site.mkdir(parents=True)
        self.course = json.loads((publish.ROOT / "course.json").read_text())
        (self.root / "course.json").write_text(json.dumps(self.course))
        self.files = {}
        ids = {m["id"] for m in self.course["modules"]} | set(self.course["guides"])
        for name in ids:
            paragraph = ('<p data-narration="4">Where generated narration is available, '
                         'use <strong>Play audio</strong> to listen.</p>') if name == "about" else '<p>Original prose.</p>'
            self.files[name + ".html"] = ("<!doctype html><html><head>"
                "<link href='assets/fieldnotes.css' rel='stylesheet'>"
                "<script defer src='assets/narration.js?v=fixture'></script></head><body>"
                "<button id='listen'>Listen</button><article><h1>Title</h1>"
                "<section aria-label='Generated narration' class='extra narration-player' data-narration-source='narration/about/cues.json'>"
                "<section><p>Nested player explanation.</p></section><audio controls><source src='narration/about/narration.m4a'></audio></section>"
                + paragraph + "<pre><code>fn exact() { println!(&quot;&lt;/section&gt;&quot;); }</code></pre>"
                "<a href='about.html'>About</a></article></body></html>").encode()
        self.files.update({"assets/fieldnotes.css": b"body{color:black}",
                           "assets/narration.js": b"/* exact optional audio player */",
                           "assets/search.json": b"[]", "examples/lesson.rs": b"fn main() {}\n",
                           "narration/about/cues.json": b'{"provider":"macos"}',
                           "narration/about/narration.m4a": b"PRIVATE MAC RECORDING"})
        for mode in publish.MODES:
            binary = b"\0asm\x01\0\0\0" + mode.encode() + b"/Users/build/panic.rs"
            metadata = {"api": 1, "mode": mode, "sourceKind": "learner", "sourcePath": "/Users/private/project",
                        "wasm": mode + ".wasm", "sourceHash": "source-fixture", "buildHash": "build-fixture",
                        "wasmHash": publish.digest(binary)}
            self.files[f"previews/{mode}/build.json"] = json.dumps(metadata).encode()
            self.files[f"previews/{mode}/{mode}.wasm"] = binary
        self.files["runtime/build.json"] = b'{"api":1,"sourceHash":"runtime-fixture"}'
        self.files["runtime/moss.wasm"] = b"\0asm\x01\0\0\0runtime"
        for checkpoint in publish.CHECKPOINTS:
            files = {f"{checkpoint}/{name}": "frozen " + name for name in
                     ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/lib.rs", "tests/acceptance.rs", "README.md")}
            self.files[f"downloads/{checkpoint}-reference.zip"] = zipped(files)
        self.flush()
        self.enterContext(patch.object(publish, "ROOT", self.root))
        # The fixture tests export behavior without pretending its tiny fake WASM
        # has real source provenance. Separate tests exercise the current-input gate.
        self.current = self.enterContext(patch.object(publish, "validate_current"))
        self.destination = self.directory / "public"

    def flush(self):
        for name, data in self.files.items():
            target = self.site / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)

    def test_export_preserves_local_files_and_binary_source_bytes_and_manifest(self):
        source = self.directory / "source.zip"
        source.write_bytes(zipped({"moss/README.md": "Public source", "moss/learning/published-preview/src/lib.rs": "pub fn world() {}"}))
        before = publish.read_site(self.site)
        manifest = publish.publish(self.destination, source)
        self.current.assert_called_once()
        self.assertEqual(publish.read_site(self.site), before)
        output = publish.read_site(self.destination)
        self.assertFalse(any(n.startswith("narration/") for n in output))
        self.assertNotIn("assets/narration.js", output)
        self.assertEqual(output[publish.SOURCE_DOWNLOAD], source.read_bytes())
        for name, data in before.items():
            if name.endswith((".wasm", ".rs", ".zip", ".css")):
                self.assertEqual(output[name], data, name)
        for name, data in output.items():
            if name.endswith(".html"):
                self.assertNotIn(b"narration-player", data, name)
                self.assertNotIn(b"narration/about", data, name)
                self.assertNotIn(b"assets/narration.js", data, name)
                self.assertIn(b"id='listen'", data, name)
                self.assertIn(b'fn exact() { println!(&quot;&lt;/section&gt;&quot;); }', data)
        setup = output["setup.html"]
        self.assertIn(b'href="downloads/moss-fieldnotes-source.zip"', setup)
        self.assertIn(b"recordings made with macOS", output["about.html"])
        for mode in publish.MODES:
            original = json.loads(before[f"previews/{mode}/build.json"])
            revised = json.loads(output[f"previews/{mode}/build.json"])
            self.assertEqual(revised.pop("sourcePath"), "verified-course-project")
            original.pop("sourcePath")
            self.assertEqual(revised, original)
        self.assertEqual(set(manifest["files"]), set(output) - {"manifest.json"})
        for name, expected in manifest["files"].items():
            self.assertEqual(expected, {"bytes": len(output[name]), "sha256": publish.digest(output[name])})

    def test_optional_source_download_never_creates_a_dead_link(self):
        publish.publish(self.destination)
        setup = (self.destination / "setup.html").read_text()
        self.assertNotIn('href="downloads/moss-fieldnotes-source.zip"', setup)
        self.assertIn("not included in this export", setup)

    def test_missing_wasm_or_checkpoint_or_course_page_leaves_destination_untouched(self):
        for missing in ("runtime/moss.wasm", "downloads/refuge-reference.zip", "setup.html"):
            with self.subTest(missing=missing):
                path = self.site / missing
                data = path.read_bytes()
                path.unlink()
                with self.assertRaises((ValueError, KeyError)):
                    publish.publish(self.destination)
                self.assertFalse(self.destination.exists())
                path.write_bytes(data)

    def test_broken_extra_audio_reference_is_not_silently_published(self):
        self.files["about.html"] = self.files["about.html"].replace(b"</article>", b"<a href='narration/about/cues.json'>Old audio</a></article>")
        self.flush()
        with self.assertRaisesRegex(ValueError, "Missing/escaping local URL|Removed narration"):
            publish.publish(self.destination)
        self.assertFalse(self.destination.exists())

    def test_broken_input_link_private_notes_and_symlinks_are_rejected(self):
        for name in ("../notes.json", "work/reading-record.json", "learning/STATUS.md"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                publish.validate_export(self.files | {name: b"private"}, self.course)
        link = self.site / "assets/private.txt"
        link.symlink_to(self.root / "course.json")
        with self.assertRaisesRegex(ValueError, "symlinks"):
            publish.publish(self.destination)

    def test_source_zip_rejects_traversal_and_private_course_notes(self):
        source = self.directory / "source.zip"
        for member in ("../escape", "/absolute", "moss/learning/design/private.md", "moss/learning/work/record.json"):
            with self.subTest(member=member):
                source.write_bytes(zipped({member: "private"}))
                with self.assertRaises(ValueError):
                    publish.publish(self.destination, source)
                self.assertFalse(self.destination.exists())
        # Existing public architectural docs are not the private course notes.
        self.assertEqual(publish.check_zip(zipped({"moss/docs/design/architecture.md": "public"}), "fixture"),
                         {"moss/docs/design/architecture.md"})

    def test_nonempty_destination_and_changed_inputs_are_not_overwritten(self):
        self.destination.mkdir()
        owned = self.destination / "my-notes.txt"
        owned.write_text("keep")
        with self.assertRaisesRegex(ValueError, "new or empty"):
            publish.publish(self.destination)
        self.assertEqual(owned.read_text(), "keep")
        owned.unlink()
        # Change the input after validation; publication must not use the stale snapshot.
        self.current.side_effect = lambda *_: (self.site / "examples/lesson.rs").write_text("changed")
        with self.assertRaisesRegex(ValueError, "changed during export"):
            publish.publish(self.destination)
        self.assertEqual(list(self.destination.iterdir()), [])

    def test_current_provenance_rejection_happens_before_publication(self):
        self.current.side_effect = ValueError("Preview is not source-current: hunting")
        with self.assertRaisesRegex(ValueError, "not source-current"):
            publish.publish(self.destination)
        self.assertFalse(self.destination.exists())

    def test_modified_wasm_bytes_fail_the_declared_digest(self):
        self.files["previews/hunting/hunting.wasm"] += b"different"
        self.flush()
        with self.assertRaisesRegex(ValueError, "WASM hash mismatch: hunting"):
            publish.publish(self.destination)
        self.assertFalse(self.destination.exists())

    def test_legacy_hosts_require_exact_local_build_bytes_and_metadata(self):
        names = ("runtime/build.json", "runtime/moss.wasm", "previews/evidence/build.json", "previews/evidence/evidence.wasm")
        for name in names:
            path = self.root / "work" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(self.files[name])
        publish.validate_built_bytes(self.files)
        for name in ("runtime/moss.wasm", "previews/evidence/evidence.wasm"):
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "built WASM"):
                publish.validate_built_bytes(self.files | {name: self.files[name] + b"tampered"})
        changed = json.loads(self.files["runtime/build.json"])
        changed["sourceHash"] = "invented"
        with self.assertRaisesRegex(ValueError, "build metadata"):
            publish.validate_built_bytes(self.files | {"runtime/build.json": json.dumps(changed).encode()})

    def test_public_gitignore_names_and_actual_urls_change_but_code_and_zips_do_not(self):
        raw = b'/target/\n# href=".gitignore.html" stays literal source text\n'
        view = (b"<html><body><h1>.gitignore</h1><a href='.gitignore?download=1&amp;raw=1'>Raw</a>"
                b"<a href='.gitignore.html?v=2#code'>View</a>"
                b"<pre><code id='code' data-source-file='.gitignore?raw=1'>/target/\n"
                b"# href=&quot;.gitignore.html&quot; stays literal source text\n</code></pre></body></html>")
        self.files["reference/refuge/.gitignore"] = raw
        self.files["reference/refuge/.gitignore.html"] = view
        self.files["setup.html"] = self.files["setup.html"].replace(b"</article>",
            b"<a href='reference/refuge/.gitignore.html?v=2#code'>Source</a></article>")
        self.flush()
        before_code = [n["text"] for n in publish.parsed(view.decode()).nodes if n["tag"] == "code"]
        publish.publish(self.destination)
        output = publish.read_site(self.destination)
        self.assertFalse(any(part.startswith(".") for name in output for part in Path(name).parts))
        self.assertEqual(output["reference/refuge/gitignore.txt"], raw)
        revised = output["reference/refuge/gitignore.txt.html"].decode()
        self.assertIn('href="gitignore.txt?download=1&amp;raw=1"', revised)
        self.assertIn('href="gitignore.txt.html?v=2#code"', revised)
        self.assertIn('data-source-file="gitignore.txt?raw=1"', revised)
        self.assertIn('href="reference/refuge/gitignore.txt.html?v=2#code"', output["setup.html"].decode())
        self.assertEqual([n["text"] for n in publish.parsed(revised).nodes if n["tag"] == "code"], before_code)
        for name, data in self.files.items():
            if name.endswith(".zip"):
                self.assertEqual(output[name], data)
        public_files = {name: data for name, data in output.items() if name != "manifest.json"}
        publish.validate_export(public_files, self.course, public=True)
        with self.assertRaisesRegex(ValueError, "dotfile"):
            publish.validate_export(public_files | {"reference/refuge/.unexpected": b"hidden"}, self.course, public=True)


if __name__ == "__main__":
    unittest.main()
