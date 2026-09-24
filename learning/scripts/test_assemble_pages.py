import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from assemble_pages import LEGACY, assemble


class AssemblePages(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.export = self.root / "export"
        self.destination = self.root / "site"
        names = {"index.html", "introduction.html", "setup.html"} | {
            target.split("#")[0] for target in LEGACY.values()}
        self.files = {name: f"<h1>{name}</h1>".encode() for name in names}
        self.files["downloads/source.zip"] = b"exact reviewed download"
        self.flush()

    def flush(self):
        for name, data in self.files.items():
            path = self.export / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.manifest = {"publicEdition": True, "edition": "fixture", "files": {
            name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
            for name, data in self.files.items()}}
        (self.export / "manifest.json").write_text(json.dumps(self.manifest))

    def test_root_is_exact_and_old_routes_redirect_with_same_origin(self):
        result = assemble(self.export, self.destination)
        for name, data in self.files.items():
            self.assertEqual((self.destination / name).read_bytes(), data)
        old = (self.destination / "fieldnotes/introduction.html").read_text()
        self.assertIn('location.replace("/moss/introduction.html" + location.search + location.hash)', old)
        self.assertIn('href="/moss/introduction.html"', old)
        self.assertEqual((self.destination / "fieldnotes/downloads/source.zip").read_bytes(),
                         self.files["downloads/source.zip"])
        self.assertIn('/moss/14-movement.html',
                      (self.destination / "chapters/01-one-affordable-step.html").read_text())
        self.assertTrue((self.destination / ".nojekyll").is_file())
        for name, evidence in result["files"].items():
            data = (self.destination / name).read_bytes()
            self.assertEqual(len(data), evidence["bytes"])
            self.assertEqual(hashlib.sha256(data).hexdigest(), evidence["sha256"])

    def test_tampered_export_leaves_destination_absent(self):
        (self.export / "index.html").write_text("unreviewed")
        with self.assertRaisesRegex(ValueError, "Changed export file"):
            assemble(self.export, self.destination)
        self.assertFalse(self.destination.exists())

    def test_unreviewed_extra_file_is_rejected(self):
        (self.export / "private.txt").write_text("not publication input")
        with self.assertRaisesRegex(ValueError, "Unreviewed files"):
            assemble(self.export, self.destination)
        self.assertFalse(self.destination.exists())

    def test_existing_destination_is_not_replaced(self):
        self.destination.mkdir()
        (self.destination / "keep.txt").write_text("existing work")
        with self.assertRaisesRegex(ValueError, "new or empty"):
            assemble(self.export, self.destination)
        self.assertEqual((self.destination / "keep.txt").read_text(), "existing work")

    def test_reserved_output_cannot_silently_replace_reviewed_content(self):
        self.files["fieldnotes/index.html"] = b"reviewed but colliding"
        self.flush()
        with self.assertRaisesRegex(ValueError, "Reserved site output"):
            assemble(self.export, self.destination)
        self.assertFalse(self.destination.exists())

    def test_external_redirect_base_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "same-site"):
            assemble(self.export, self.destination, "//another-site.example/")


if __name__ == "__main__":
    unittest.main()
