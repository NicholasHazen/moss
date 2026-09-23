#!/usr/bin/env python3
"""Small fixture tests for starter export; no Rust builds or network requests."""
from __future__ import annotations

import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import tempfile
import unittest
import zipfile

import package_starter as starter


class StarterTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory(prefix="moss-starter-test-")
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name).resolve()
        for relative in starter.FIXED_FILES:
            self.write(relative, f"// Exact fixture for {relative}\n".encode())
        for name in starter.CRATES:
            self.write(f"crates/{name}/Cargo.toml", b"[package]\n")
        self.lessons = (b"// UTF-8 fern: \xf0\x9f\x8c\xbf\r\n"
                        b"pub fn move_one_cell(\n) -> u32 {\n"
                        b'    todo!("Paired movement exercise: fixture")\n}\n')
        self.write("crates/moss-sim/src/lessons.rs", self.lessons)
        self.write("crates/moss-sim/src/simulation.rs", (
            b"fn install() {\n"
            b"    schedule.add_systems((lessons::spend_energy, complete_tick).chain());\n}\n"
        ))
        self.write("crates/moss-web/src/main.rs", b"fn main() {}\n")
        for path, marker in (
            ("docs/tutorial/today-v2.md", "example: movement-v2-helper"),
            ("docs/tutorial/04-movement.md", "example: movement-helper"),
            *((f"docs/tutorial/path/{n:02d}-fixture.md", f"runnable: session-{n:02d}")
              for n in range(1, 17)),
        ):
            self.write(path, ("# Private surrounding fixture prose\n\n"
                              f"<!-- {marker} -->\n```rust\nfn example() {{}}\n```\n").encode())

    def write(self, relative: str, data: bytes) -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def contents(self) -> tuple[dict[str, bytes], dict, zipfile.ZipFile]:
        data, manifest = starter.build_archive(self.root)
        archive = zipfile.ZipFile(io.BytesIO(data))
        self.addCleanup(archive.close)
        prefix = starter.NAME + "/"
        files = {name.removeprefix(prefix): archive.read(name) for name in archive.namelist()}
        return files, manifest, archive

    def test_only_allowed_files_are_bundled(self) -> None:
        for private in ("NOW.md", "AGENTS.md", ".git/config", ".idea/workspace.xml",
                        ".run/Local.run.xml", "docs/agents/collaboration.md",
                        "book/editorial/STATUS.md", "scripts/private.sh"):
            self.write(private, b"private fixture")
        files, _, archive = self.contents()
        self.assertEqual(len(files), len(starter.FIXED_FILES) + 5 + 18 + 2)
        for path in files:
            self.assertFalse(any(part.startswith(".") for part in Path(path).parts), path)
            self.assertNotIn("..", Path(path).parts)
            self.assertNotIn("private", path)
            self.assertNotIn("AGENTS.md", path)
            self.assertNotIn("NOW.md", path)
        self.assertTrue(all(name.startswith(starter.NAME + "/") for name in archive.namelist()))

    def test_rust_and_lockfile_bytes_are_preserved_exactly(self) -> None:
        files, _, _ = self.contents()
        self.assertEqual(files["crates/moss-sim/src/lessons.rs"], self.lessons)
        self.assertEqual(files["Cargo.lock"], (self.root / "Cargo.lock").read_bytes())
        self.assertIn(b'todo!("Paired movement exercise:', files["crates/moss-sim/src/lessons.rs"])
        self.assertNotIn(b"lessons::move_to_food", files["crates/moss-sim/src/simulation.rs"])

    def test_references_copy_only_the_exact_marked_rust_block(self) -> None:
        files, _, _ = self.contents()
        for path, marker in starter.reference_paths(self.root):
            extracted = files[path].decode()
            self.assertIn("not the full tutorial page", extracted)
            self.assertNotIn("Private surrounding fixture prose", extracted)
            blocks = re.findall(r"```rust\n(.*?)\n```", extracted, re.DOTALL)
            self.assertEqual(blocks, ["fn example() {}"])
            self.assertIn(f"<!-- {marker} -->", extracted)

    def test_scripts_have_executable_unix_modes_and_fixed_dates(self) -> None:
        _, _, archive = self.contents()
        for item in archive.infolist():
            path = item.filename.removeprefix(starter.NAME + "/")
            mode = item.external_attr >> 16
            self.assertTrue(stat.S_ISREG(mode))
            self.assertEqual(stat.S_IMODE(mode), 0o755 if path in starter.EXECUTABLES else 0o644)
            self.assertEqual(item.date_time, starter.ZIP_TIMESTAMP)
            self.assertEqual(item.compress_type, zipfile.ZIP_STORED)

    def test_archive_is_deterministic_despite_source_mtime_and_mode(self) -> None:
        first, _ = starter.build_archive(self.root)
        path = self.root / "crates/moss-sim/src/lessons.rs"
        os.utime(path, (1_000_000_000, 1_000_000_000))
        path.chmod(0o600)
        second, _ = starter.build_archive(self.root)
        self.assertEqual(first, second)

    def test_manifest_covers_every_other_file_and_its_exact_hash(self) -> None:
        files, manifest, _ = self.contents()
        self.assertEqual(json.loads(files.pop("STARTER-MANIFEST.json")), manifest)
        self.assertEqual({item["path"] for item in manifest["files"]}, set(files))
        for item in manifest["files"]:
            self.assertEqual(item["sha256"], hashlib.sha256(files[item["path"]]).hexdigest())
            self.assertEqual(item["bytes"], len(files[item["path"]]))
        self.assertEqual(manifest["reference_checkpoint"], starter.REFERENCE_CHECKPOINT)
        self.assertIn("local modifications", manifest["source_state"])

    def test_source_file_symlink_is_rejected(self) -> None:
        path = self.root / "Cargo.lock"
        path.unlink()
        path.symlink_to(self.root / "Cargo.toml")
        with self.assertRaisesRegex(ValueError, "Symlink"):
            starter.build_archive(self.root)

    def test_source_directory_symlink_is_rejected(self) -> None:
        alias = self.root / "crates/moss-sim/alias"
        alias.symlink_to(self.root / "crates/moss-web", target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "Symlink"):
            starter.build_archive(self.root)

    def test_unexpected_extension_and_hidden_crate_file_are_rejected(self) -> None:
        for relative in ("crates/moss-sim/notes.md", "crates/moss-web/.local.rs"):
            with self.subTest(relative=relative):
                path = self.write(relative, b"must not export")
                with self.assertRaises(ValueError):
                    starter.build_archive(self.root)
                path.unlink()

    def test_new_crate_requires_explicit_allowlist_review(self) -> None:
        self.write("crates/private/Cargo.toml", b"[package]\n")
        with self.assertRaisesRegex(ValueError, "crate inventory"):
            starter.build_archive(self.root)

    def test_solved_helper_and_activated_schedule_are_rejected(self) -> None:
        helper = self.root / "crates/moss-sim/src/lessons.rs"
        helper.write_bytes(self.lessons.replace(b'todo!("Paired movement exercise: fixture")', b"0"))
        with self.assertRaisesRegex(ValueError, "unfinished"):
            starter.build_archive(self.root)
        helper.write_bytes(self.lessons)
        schedule = self.root / "crates/moss-sim/src/simulation.rs"
        schedule.write_bytes(schedule.read_bytes() + b"schedule.add_systems(lessons::move_to_food);\n")
        with self.assertRaisesRegex(ValueError, "maintenance-only"):
            starter.build_archive(self.root)

    def test_duplicate_reference_sources_and_markers_fail_closed(self) -> None:
        duplicate = self.write("docs/tutorial/path/01-extra.md", b"not selected silently")
        with self.assertRaisesRegex(ValueError, "one canonical source"):
            starter.build_archive(self.root)
        duplicate.unlink()
        self.write("docs/tutorial/path/01-fixture.md", b"no canonical Rust fence")
        with self.assertRaisesRegex(ValueError, "one complete"):
            starter.build_archive(self.root)


if __name__ == "__main__":
    unittest.main()
