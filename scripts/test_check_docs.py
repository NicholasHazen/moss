"""Focused fixtures for the guide's local-navigation checker."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from check_docs import check


class DocumentationChecks(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory(prefix="moss-doc-check-")
        self.addCleanup(self.workspace.cleanup)
        self.root = Path(self.workspace.name)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def test_local_page_image_and_reference_links(self):
        self.write("README.md", """# Home
[Lesson](docs/lesson.md#first-steps)
![Map](docs/map.svg)
[Companion][next]
[External](https://example.invalid/no-network-check)
[next]: docs/lesson.md#first-steps
""")
        self.write("docs/lesson.md", "# First steps\n[Home](../README.md)\n")
        self.write("docs/map.svg", '<svg xmlns="http://www.w3.org/2000/svg"/>')
        report = check(self.root)
        self.assertEqual(report.errors, [])
        self.assertEqual(report.files, 2)
        self.assertGreaterEqual(report.links, 4)

    def test_fieldnotes_entry_and_authoring_are_checked_without_generated_copies(self):
        self.write("README.md", "# Project\n")
        self.write("learning/README.md", "[Contract](authoring/README.md#write)\n")
        contract = self.write("learning/authoring/README.md", "# Write\n[Home](../README.md)\n")
        self.write("learning/_site/example.md", "[Generated](missing.md)\n")
        self.write("learning/work/scratch.md", "[Scratch](missing.md)\n")
        self.assertEqual(check(self.root).errors, [])
        contract.write_text("# Renamed\n")
        self.assertTrue(any("#write" in error for error in check(self.root).errors))

    def test_missing_file_and_missing_heading_are_separate_errors(self):
        self.write("README.md", "[File](gone.md)\n[Heading](docs/lesson.md#absent)\n")
        self.write("docs/lesson.md", "# Present\n")
        errors = check(self.root).errors
        self.assertEqual(len(errors), 2)
        self.assertTrue(any("gone.md" in error for error in errors))
        self.assertTrue(any("absent" in error for error in errors))

    def test_fenced_headings_and_explicit_anchors_do_not_exist(self):
        self.write("README.md", "[Heading](docs/lesson.md#ghost)\n[Alias](docs/lesson.md#pretend)\n")
        self.write("docs/lesson.md", """# Actual
```markdown
# Ghost
<a id="pretend"></a>
[Not a link](missing.md)
```
""")
        errors = check(self.root).errors
        self.assertEqual(len(errors), 2)
        self.assertTrue(any("ghost" in error for error in errors))
        self.assertTrue(any("pretend" in error for error in errors))
        self.assertFalse(any("missing.md" in error for error in errors))

    def test_duplicate_headings_and_retained_alias(self):
        self.write("README.md", """[First](docs/lesson.md#repeat)
[Second](docs/lesson.md#repeat-1)
[Old bookmark](docs/lesson.md#older-title)
""")
        self.write("docs/lesson.md", """# Repeat
## Repeat
<a id="older-title"></a>
## Current title
""")
        self.assertEqual(check(self.root).errors, [])

    def test_reference_label_normalization_and_missing_definition(self):
        self.write("README.md", """[Good][NEXT   chapter]
[Mistyped][next-chaptr]
[next chapter]: docs/lesson.md#start
""")
        self.write("docs/lesson.md", "# Start\n")
        errors = check(self.root).errors
        self.assertEqual(len(errors), 1)
        self.assertIn("next-chaptr", errors[0])

    def test_encoded_and_angle_delimited_paths(self):
        self.write("README.md", """[Encoded](docs/A%20note%20%28draft%29.md#r%C3%A9sum%C3%A9)
[Angle](<docs/A note (draft).md#résumé>)
[Parentheses](docs/example(1).md#read)
""")
        self.write("docs/A note (draft).md", "# Résumé\n")
        self.write("docs/example(1).md", "# Read\n")
        self.assertEqual(check(self.root).errors, [])

    def test_outer_fence_and_inline_code_hide_literal_links(self):
        self.write("README.md", """````markdown
```rust
# Not a heading
```
[Not a link](missing.md)
````
# Real
`[Literal](missing-inline.md)`
[Works](#real)
""")
        report = check(self.root)
        self.assertEqual(report.errors, [])
        self.assertEqual(report.links, 1)

    def test_unclosed_fence_reports_the_document(self):
        self.write("docs/unfinished.md", "# Start\n```rust\nlet x = 1;\n~~~\n")
        errors = check(self.root).errors
        self.assertTrue(errors)
        self.assertTrue(any("unfinished.md" in error for error in errors))

    def test_no_documents_is_not_a_successful_check(self):
        report = check(self.root)
        self.assertEqual(report.files, 0)
        self.assertTrue(report.errors)

    def test_cli_works_from_another_directory_and_returns_failure(self):
        self.write("README.md", "[Broken](missing.md)\n")
        checker = Path(__file__).with_name("check_docs.py")
        result = subprocess.run(
            [sys.executable, "-B", str(checker), "--root", str(self.root)],
            cwd=self.root,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing.md", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
