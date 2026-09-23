#!/usr/bin/env python3
"""Edition-package fixtures: no compiler, shared build, Git or network access."""
from __future__ import annotations

import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

import package_edition as edition
import package_starter as starter


class EditionTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory(prefix="moss-edition-test-")
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name).resolve() / "source"
        for path in set(starter.FIXED_FILES) | edition.REQUIRED:
            self.write(path, b"exact source fixture\n")
        for name in starter.CRATES:
            self.write(f"crates/{name}/Cargo.toml", b"[package]\n")
        self.write("crates/moss-sim/src/lessons.rs", (
            b"pub fn move_one_cell(\n) -> u32 {\n"
            b'    todo!("Paired movement exercise: fixture")\n}\n'
        ))
        self.write("crates/moss-sim/src/simulation.rs",
                   b"schedule.add_systems((lessons::spend_energy, complete_tick).chain());\n")
        self.write("crates/moss-web/src/main.rs", b"fn main() {}\n")
        for path, marker in (
            ("docs/tutorial/today-v2.md", "example: movement-v2-helper"),
            ("docs/tutorial/04-movement.md", "example: movement-helper"),
            *((f"docs/tutorial/path/{n:02d}-fixture.md", f"runnable: session-{n:02d}")
              for n in range(1, 17)),
        ):
            self.write(path, (f"# Exact canonical page\n<!-- {marker} -->\n"
                              "```rust\nfn reference() {}\n```\n").encode())
        self.write("book/theme/style.css", b"body {}\n")
        self.write("book/scripts/test_edition.py", b"# edition's tests belong in its source download\n")
        self.write("book/src/chapters/one.md", b"# One\n[Source](../../../docs/design/public.md)\n")
        self.write("docs/design/public.md", b"# Public\n[Not followed](../agents/private.md)\n")

    def write(self, relative: str, data: bytes) -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def contents(self) -> tuple[dict[str, bytes], dict]:
        data, manifest = edition.build_archive(self.root)
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            files = {name.removeprefix(edition.NAME + "/"): archive.read(name)
                     for name in archive.namelist()}
        return files, manifest

    def test_required_build_inputs_and_exact_sources_are_present(self) -> None:
        files, _ = self.contents()
        self.assertTrue(edition.REQUIRED <= files.keys())
        self.assertTrue(set(starter.FIXED_FILES) <= files.keys())
        for path in (
            "book/src/chapters/one.md", "book/theme/style.css",
            "book/scripts/package_edition.py", "book/scripts/test_edition.py",
            "crates/moss-sim/src/lessons.rs", "Cargo.lock",
            "docs/tutorial/authoring/check_docs.py", "docs/design/public.md",
            *(path for path, _ in starter.reference_paths(self.root)),
        ):
            self.assertEqual(files[path], (self.root / path).read_bytes(), path)

    def test_local_readmes_private_files_and_caches_are_excluded(self) -> None:
        for path in (
            "README.md", "book/README.md", "NOW.md", "AGENTS.md",
            ".git/config", ".idea/workspace.xml", ".run/Personal.run.xml",
            "book/editorial/STATUS.md", "docs/history/private.md",
            "docs/agents/private.md", "book/scripts/__pycache__/fixture.pyc",
            "book/artifacts/old.zip", "book/_site/index.html",
        ):
            self.write(path, b"PRIVATE WORK NOTE")
        files, _ = self.contents()
        self.assertTrue(all(b"PRIVATE WORK NOTE" not in data for data in files.values()))
        self.assertIn(b"Python **3.11 or later**", files["book/README.md"])
        self.assertIn(b"sh book/scripts/book.sh build", files["README.md"])
        self.assertNotIn(b"package_edition.py --copy-to-site", files["README.md"])
        self.assertFalse(any(part.startswith(".") for path in files for part in Path(path).parts))
        self.assertNotIn("docs/agents/private.md", files)
        self.assertNotIn("book/editorial/STATUS.md", files)

    def test_citations_are_direct_only_and_manifested(self) -> None:
        self.write("docs/agents/private.md", b"do not follow the guide's link")
        files, manifest = self.contents()
        self.assertEqual(manifest["direct_project_citations"], ["docs/design/public.md"])
        self.assertIn("docs/design/public.md", files)
        self.assertNotIn("docs/agents/private.md", files)

    def test_generated_download_links_do_not_copy_archive_into_itself(self) -> None:
        self.write("book/src/reference/download.md", (
            f"[Source](../downloads/{edition.NAME}.zip)\n"
            f"[Checksum](../downloads/{edition.NAME}.zip.sha256)\n"
            f"[Starter](../downloads/{starter.NAME}.zip)\n"
        ).encode())
        files, _ = self.contents()
        self.assertFalse(any(path.endswith(".zip") for path in files))

    def test_direct_private_authoring_and_outside_citations_fail_closed(self) -> None:
        for target in ("NOW.md", "book/editorial/STATUS.md", "docs/agents/private.md",
                       "docs/tutorial/authoring/check_docs.py", "../outside.md"):
            with self.subTest(target=target):
                self.write(target, b"do not publish")
                self.write("book/src/chapters/one.md", f"[Forbidden](../../../{target})\n".encode())
                with self.assertRaises(ValueError):
                    edition.build_archive(self.root)

    def test_source_and_cited_symlinks_fail_closed(self) -> None:
        alias = self.root / "book/theme/alias.css"
        alias.symlink_to(self.root / "book/theme/style.css")
        with self.assertRaisesRegex(ValueError, "Symlink"):
            edition.build_archive(self.root)
        alias.unlink()
        cited = self.root / "docs/design/public.md"
        cited.unlink()
        cited.symlink_to(self.root / "docs/tutorial/today-v2.md")
        with self.assertRaisesRegex(ValueError, "Symlink"):
            edition.build_archive(self.root)

    def test_unexpected_book_file_type_fails_closed(self) -> None:
        self.write("book/src/secret.txt", b"not an approved public source type")
        with self.assertRaisesRegex(ValueError, "Unexpected"):
            edition.build_archive(self.root)

    def test_missing_build_input_is_not_silently_omitted(self) -> None:
        (self.root / "book/scripts/package_starter.py").unlink()
        with self.assertRaisesRegex(ValueError, "Missing edition build inputs"):
            edition.build_archive(self.root)

    def test_manifest_hashes_and_modes_cover_every_other_file(self) -> None:
        files, manifest = self.contents()
        self.assertEqual(json.loads(files.pop("EDITION-MANIFEST.json")), manifest)
        self.assertEqual(set(files), {item["path"] for item in manifest["files"]})
        for item in manifest["files"]:
            data = files[item["path"]]
            self.assertEqual(starter.digest(data), item["sha256"])
            self.assertEqual(len(data), item["bytes"])
            expected_mode = "0755" if item["path"] in edition.EXECUTABLES else "0644"
            self.assertEqual(item["mode"], expected_mode)

    def test_repeated_and_extracted_source_rebuilds_are_byte_identical(self) -> None:
        first, _ = edition.build_archive(self.root)
        second, _ = edition.build_archive(self.root)
        self.assertEqual(first, second)
        destination = self.root.parent / "extracted"
        with zipfile.ZipFile(io.BytesIO(first)) as archive:
            archive.extractall(destination)
        third, _ = edition.build_archive(destination / edition.NAME)
        self.assertEqual(first, third)


if __name__ == "__main__":
    unittest.main()
