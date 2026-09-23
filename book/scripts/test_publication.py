#!/usr/bin/env python3
"""Isolated publication regression tests; no shared build, browser or network.

Run with ``python3 -B -m unittest discover -s book/scripts -p test_publication.py``.
Each case constructs a tiny repository/site in a temporary directory. Failures
are actual assertions, including defects found by the publication audit; they
are not hidden behind expected-failure decorators.
"""
from __future__ import annotations

from contextlib import redirect_stdout
import hashlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


def load_script(name: str):
    path = Path(__file__).with_name(name + ".py")
    spec = importlib.util.spec_from_file_location("publication_test_" + name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


finish = load_script("finish_site")
checker = load_script("check_book")


class PublicationTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="moss-publication-test-")
        self.addCleanup(temporary.cleanup)
        self.scratch = Path(temporary.name).resolve()
        self.root = self.scratch / "repo"
        self.book = self.root / "book"
        self.site = self.book / "_site"
        self.site.mkdir(parents=True)
        allowed = tuple(self.root / area for area in (
            "crates/moss-sim", "crates/moss-web", "docs/tutorial",
            "docs/development", "docs/design", "docs/research",
        ))
        self.enterContext(patch.multiple(finish, ROOT=self.root, SITE=self.site,
                                        ALLOWED=allowed))
        self.enterContext(patch.multiple(checker, ROOT=self.root, BOOK=self.book,
                                        SITE=self.site))
        self.write("book/book.toml", '[output.html]\nsite-url = "/"\n')
        self.write("book/_site/index.html", '<h1 id="home">Home</h1>')

    def write(self, relative: str, content: str) -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def package(self) -> None:
        with redirect_stdout(io.StringIO()):
            finish.main()

    def html_errors(self) -> list[str]:
        errors: list[str] = []
        checker.check_html(errors)
        return errors

    def test_rust_source_is_escaped_and_hashed(self) -> None:
        text = 'let literal = "<script>alert(1)</script> & value";\n'
        source = self.write("crates/moss-sim/src/example.rs", text)
        output = finish.snapshot(source).read_text()
        self.assertIn("&lt;script&gt;alert(1)&lt;/script&gt; &amp; value", output)
        self.assertNotIn("<script>", output)
        self.assertIn(hashlib.sha256(text.encode()).hexdigest(), output)
        self.assertIn('id="L1"', output)

    def test_markdown_heading_and_line_anchors_survive(self) -> None:
        source = self.write("docs/tutorial/example.md", "# A step\n\n# A step\n")
        output = finish.snapshot(source)
        parsed = checker.Page()
        parsed.feed(output.read_text())
        self.assertTrue({"a-step", "a-step-1", "L1", "L2", "L3"} <= parsed.ids)
        self.assertEqual(parsed.duplicates, set())

    def test_citation_is_rewritten_and_deduplicated(self) -> None:
        self.write("docs/tutorial/example.md", "# A step\n")
        self.write("book/_site/chapters/one.html", (
            '<a href="../../../docs/tutorial/example.md#a-step">First</a>'
            '<a href="../../../docs/tutorial/example.md#L1">Second</a>'
        ))
        self.package()
        page = (self.site / "chapters/one.html").read_text()
        self.assertIn('../source/docs/tutorial/example.md.html#a-step', page)
        self.assertIn('../source/docs/tutorial/example.md.html#L1', page)
        self.assertEqual(len(list((self.site / "source").rglob("*.html"))), 1)
        self.assertEqual(self.html_errors(), [])

    def test_mdbook_html_extension_can_resolve_markdown_citation(self) -> None:
        self.write("docs/design/example.md", "# Design\n")
        self.write("book/_site/index.html", '<a href="../../docs/design/example.html#design">Design</a>')
        self.package()
        self.assertIn('source/docs/design/example.md.html#design',
                      (self.site / "index.html").read_text())
        self.assertEqual(self.html_errors(), [])

    def test_external_urls_and_internal_assets_are_not_rewritten(self) -> None:
        original = ('<a href="https://example.test/other?q=1&amp;v=2">Other</a>'
                    '<script src="assets/lab.js"></script>')
        self.write("book/_site/index.html", original)
        self.write("book/_site/assets/lab.js", "// local model\n")
        self.package()
        self.assertEqual((self.site / "index.html").read_text(), original)
        self.assertEqual(self.html_errors(), [])

    def test_private_and_authoring_areas_are_rejected(self) -> None:
        for relative in ("NOW.md", "AGENTS.md", "docs/agents/collaboration.md",
                         "docs/tutorial/authoring/verification.md",
                         "book/editorial/STATUS.md"):
            with self.subTest(relative=relative):
                self.write(relative, "Private fixture\n")
                self.write("book/_site/index.html", f'<a href="../../{relative}">No</a>')
                with self.assertRaises(ValueError):
                    self.package()
        self.assertFalse((self.site / "source").exists())

    def test_symlink_cannot_disguise_disallowed_source(self) -> None:
        private = self.write("NOW.md", "Private fixture\n")
        alias = self.root / "docs/tutorial/alias.md"
        alias.parent.mkdir(parents=True)
        alias.symlink_to(private)
        self.write("book/_site/index.html", '<a href="../../docs/tutorial/alias.md">No</a>')
        with self.assertRaises(ValueError):
            self.package()

    def test_traversal_cannot_publish_outside_repository(self) -> None:
        (self.scratch / "outside.md").write_text("Outside fixture\n")
        self.write("book/_site/index.html", '<a href="../../../outside.md">No</a>')
        with self.assertRaises(ValueError):
            self.package()

    def test_unsupported_source_types_are_rejected(self) -> None:
        source = self.write("docs/tutorial/fixture.txt", "Unsupported\n")
        with self.assertRaisesRegex(ValueError, "Unsupported"):
            finish.snapshot(source)

    def test_snapshot_does_not_follow_links_in_markdown(self) -> None:
        self.write("NOW.md", "Private fixture\n")
        source = self.write("docs/tutorial/example.md", "# Example\n[Private](../../NOW.md)\n")
        snapshot = finish.snapshot(source)
        self.assertIn("[Private](../../NOW.md)", snapshot.read_text())
        self.assertFalse((self.site / "source/NOW.md.html").exists())
        self.assertEqual(self.html_errors(), [])

    def test_checker_reports_missing_output_and_fragment(self) -> None:
        self.write("book/_site/index.html", (
            '<a href="absent.html">Missing</a>'
            '<a href="other.html#absent">Bad fragment</a>'
        ))
        self.write("book/_site/other.html", '<h1 id="present">Present</h1>')
        errors = self.html_errors()
        self.assertTrue(any("unresolved output link absent.html" in error for error in errors))
        self.assertTrue(any("missing output fragment other.html#absent" in error for error in errors))

    def test_checker_reports_duplicate_ids(self) -> None:
        self.write("book/_site/index.html", '<h1 id="twice">One</h1><p id="twice">Two</p>')
        self.assertTrue(any("duplicate IDs" in error for error in self.html_errors()))

    def test_checker_rejects_link_that_escapes_export(self) -> None:
        self.write("NOW.md", "Outside the site\n")
        self.write("book/_site/index.html", '<a href="../../NOW.md">No</a>')
        self.assertTrue(any("unresolved output link" in error for error in self.html_errors()))

    def test_attribute_looking_text_is_not_a_publication_request(self) -> None:
        original = ('<pre><code>&lt;a href="../../NOW.md"&gt;example&lt;/a&gt;</code></pre>'
                    '<!-- <a href="../../NOW.md">comment</a> -->'
                    '<script>const example = \'<a href="../../NOW.md">literal</a>\';</script>')
        self.write("NOW.md", "Private fixture\n")
        self.write("book/_site/index.html", original)
        self.package()
        self.assertEqual((self.site / "index.html").read_text(), original)
        self.assertFalse((self.site / "source").exists())

    def test_active_svg_is_rejected_instead_of_exported_as_inert(self) -> None:
        source = self.write("docs/tutorial/active.svg", (
            '<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)">'
            '<script>alert(2)</script></svg>'
        ))
        with self.assertRaises(ValueError):
            finish.snapshot(source)

    def test_checker_accepts_configured_repository_base(self) -> None:
        self.write("book/book.toml", '[output.html]\nsite-url = "/moss/"\n')
        self.write("book/_site/404.html", '<base href="/moss/"><link href="assets/site.css"><a href="index.html#home">Home</a>')
        self.write("book/_site/assets/site.css", "body {}\n")
        self.assertEqual(self.html_errors(), [])

    def test_error_page_heading_survives_site_root_base(self) -> None:
        self.write("book/book.toml", '[output.html]\nsite-url = "/moss/"\n')
        self.write("book/_site/404.html", '<base href="/moss/"><h1 id="missing"><a href="#missing">Missing</a></h1>')
        self.package()
        self.assertIn('href="404.html#missing"', (self.site / "404.html").read_text())
        self.assertEqual(self.html_errors(), [])

    def test_svg_rejects_external_resources_and_entity_declarations(self) -> None:
        for body in ('<style>@import "https://example.test/a.css";</style>',
                     '<path fill="url(https://example.test/a.svg)"/>',
                     '<foreignObject/>', '<path onload="run()"/>'):
            with self.subTest(body=body):
                with self.assertRaises(ValueError):
                    finish.validate_figure(('<svg xmlns="http://www.w3.org/2000/svg">' + body + '</svg>').encode())
        with self.assertRaises(ValueError):
            finish.validate_figure(b'<!DOCTYPE svg [<!ENTITY a "value">]><svg xmlns="http://www.w3.org/2000/svg"/>')

    def test_svg_allows_local_markers_and_passive_dark_theme(self) -> None:
        finish.validate_figure(b'<svg xmlns="http://www.w3.org/2000/svg"><style>@media (prefers-color-scheme: dark) { path { fill: #fff; } }</style><path marker-end="url(#arrow)"/></svg>')

    def test_checker_rejects_base_that_escapes_repository_site(self) -> None:
        self.write("book/book.toml", '[output.html]\nsite-url = "/moss/"\n')
        self.write("book/_site/404.html", '<base href="/"><link href="assets/site.css"><a href="index.html#home">Home</a>')
        self.write("book/_site/assets/site.css", "body {}\n")
        self.assertNotEqual(self.html_errors(), [],
                            "The browser requests /assets and /index, outside /moss/.")


if __name__ == "__main__":
    unittest.main()
