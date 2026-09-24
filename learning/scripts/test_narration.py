import unittest
import hashlib
import json
import tempfile
from unittest.mock import patch
from pathlib import Path
from narration import fingerprint, passages, read_current


class NarrationExtraction(unittest.TestCase):
    def test_inline_text_is_preserved_and_code_answers_are_omitted(self):
        source = '<p id="intro">Read <code>Some(0)</code> &amp; keep it.</p><pre><code>fn main() {}</code></pre><details><summary>Answer</summary><p>Hidden answer.</p></details><h2>Try again</h2>'
        blocks, annotated = passages("Title", source)
        self.assertEqual(blocks, ["Title", "Read Some(0) & keep it.", "Try again"])
        self.assertIn('<p id="intro" data-narration="1">',annotated)
        self.assertIn('<h2 data-narration="2">',annotated)
        self.assertIn('<p>Hidden answer.</p>',annotated)

    def test_table_reads_column_context_and_nested_list_is_one_passage(self):
        source = '<table><caption>Reserves</caption><thead><tr><th>Name</th><th>Units</th></tr></thead><tbody><tr><th>Fern</th><td>0</td></tr></tbody></table><ul><li><p>First.</p><p>Second.</p></li></ul>'
        blocks, _ = passages("Title",source)
        self.assertEqual(blocks[1],"Reserves. Name: Fern. Units: 0.")
        self.assertEqual(blocks[2],"First. Second.")

    def test_title_or_prose_change_invalidates_generated_audio(self):
        self.assertNotEqual(fingerprint("Old","<p>Same</p>"),fingerprint("New","<p>Same</p>"))
        self.assertNotEqual(fingerprint("Same","<p>Old</p>"),fingerprint("Same","<p>New</p>"))

    def test_empty_runtime_status_is_not_a_spoken_passage_or_a_cue_gap(self):
        source = '<p>Begin.</p><p role="status"></p><p> \n </p><h2>Continue</h2><p>Read <code>zero</code>.</p>'
        blocks, annotated = passages("Title", source)
        self.assertEqual(blocks, ["Title", "Begin.", "Continue", "Read zero."])
        self.assertIn('<p role="status"></p>', annotated)
        self.assertIn('<p> \n </p>', annotated)
        self.assertIn('<h2 data-narration="2">Continue</h2>', annotated)
        self.assertIn('<p data-narration="3">Read', annotated)
        self.assertEqual(annotated.count('data-narration='), 3)

    def test_mixed_audio_and_cues_are_rejected_and_accepted_bytes_are_snapshots(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            audio = b"first generation audio fixture"
            metadata = {"format": 1, "sourceHash": fingerprint("Title", "<p>Text.</p>"),
                        "audioHash": hashlib.sha256(audio).hexdigest(), "duration": 4,
                        "cues": [{"passage":0,"text":"Title","start":0,"end":1},
                                 {"passage":1,"text":"Text.","start":1,"end":4}]}
            (path / "narration.m4a").write_bytes(audio)
            (path / "cues.json").write_text(json.dumps(metadata))
            accepted = read_current(path, "Title", "<p>Text.</p>")
            self.assertIsNotNone(accepted)
            (path / "narration.m4a").write_bytes(b"new voice with different timing")
            self.assertIsNone(read_current(path, "Title", "<p>Text.</p>"))
            self.assertEqual(accepted[1], audio)
            self.assertEqual(accepted[0]["duration"], 4)

    def test_malformed_cues_do_not_enable_broken_highlighting(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            audio = b"audio fixture"
            (path / "narration.m4a").write_bytes(audio)
            for cues in (None, [], [{"passage":"bad"}],
                         [{"passage":0,"text":"Title","start":0,"end":float("nan")} ]):
                metadata = {"format":1,"sourceHash":fingerprint("Title",""),
                            "audioHash":hashlib.sha256(audio).hexdigest(),"duration":1,"cues":cues}
                (path / "cues.json").write_text(json.dumps(metadata))
                self.assertIsNone(read_current(path,"Title",""))

    def test_build_uses_the_provider_from_the_verified_audio_snapshot(self):
        import build
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "learning"
            (root / "content").mkdir(parents=True)
            (root / "assets").mkdir()
            (root.parent / "docs/tutorial").mkdir(parents=True)
            (root.parent / "docs/tutorial/today-v2.md").write_text("Fixture guide")
            source = '<p>One saved reading.</p>'
            (root / "content/about.html").write_text(source)
            (root / "content/01-fixture.html").write_text(source)
            course = {"edition": "Test edition", "description": "Test", "modules": [
                          {"id": "01-fixture", "title": "About", "summary": "Test", "minutes": 1,
                           "part": "Test", "concepts": ["borrowing"]}],
                      "guides": {"about": "About"}}
            (root / "course.json").write_text(json.dumps(course))
            audio_dir = root / "work/narration/about"
            audio_dir.mkdir(parents=True)
            audio = b"verified provider audio fixture"
            (audio_dir / "narration.m4a").write_bytes(audio)
            metadata = {"format": 1, "sourceHash": fingerprint("About", source),
                        "audioHash": hashlib.sha256(audio).hexdigest(),
                        "provider": "elevenlabs", "duration": 3,
                        "cues": [{"passage": 0, "text": "About", "start": 0, "end": 1},
                                 {"passage": 1, "text": "One saved reading.", "start": 1, "end": 3}]}
            (audio_dir / "cues.json").write_text(json.dumps(metadata))
            module_audio = root / "work/narration/01-fixture"
            module_audio.mkdir()
            (module_audio / "narration.m4a").write_bytes(audio)
            (module_audio / "cues.json").write_text(json.dumps(metadata))
            accepted = read_current(audio_dir, "About", source)
            self.assertIsNotNone(accepted)
            def render_site():
                with patch.object(build, "ROOT", root), patch.object(build, "OUT", root / "_site"), \
                        patch.object(build, "copy_preview"), patch.object(build, "copy_fieldwork_preview"):
                    build.build()
                return (root / "_site/about.html").read_text()
            page = render_site()
            self.assertIn("Generated with ElevenLabs.", page)
            self.assertIn("Generated with ElevenLabs.", (root / "_site/01-fixture.html").read_text())
            self.assertNotIn("Generated with an installed macOS voice.", page)
            self.assertIn('narration/about/narration.m4a?v=', page)
            self.assertIn('narration/about/cues.json?v=', page)
            self.assertEqual((root / "_site/narration/about/narration.m4a").read_bytes(), audio)
            # A local provider label cannot turn stale or mismatched audio into
            # a publishable track, nor trigger a paid service during a build.
            self.assertIsNone(read_current(audio_dir, "About", source + '<p>New.</p>'))
            (root / "content/about.html").write_text(source + '<p>New.</p>')
            self.assertNotIn('class="narration-player"', render_site())
            self.assertFalse((root / "_site/narration/about").exists())
            for invalid in ([], {}, "unknown-service"):
                metadata["provider"] = invalid
                (audio_dir / "cues.json").write_text(json.dumps(metadata))
                self.assertIsNone(read_current(audio_dir, "About", source))


if __name__ == "__main__":
    unittest.main()
