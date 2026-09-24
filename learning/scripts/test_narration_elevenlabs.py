"""Offline provider tests: synthetic responses only; never contact ElevenLabs."""
import base64
from contextlib import ExitStack, redirect_stdout, redirect_stderr
import copy
import io
import json
import math
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unittest
from unittest import mock
from urllib.error import HTTPError
import wave

import narration_elevenlabs as eleven
from narration import fingerprint, passages, read_current


def response_for(text):
    frames = len(text) * 100 + 200
    return {"audio_base64": base64.b64encode(f"frames:{frames}".encode()).decode(),
            "alignment": {"characters": list(text),
                          "character_start_times_seconds": [i * 100 / eleven.RATE for i in range(len(text))],
                          "character_end_times_seconds": [(i + 1) * 100 / eleven.RATE for i in range(len(text))]}}


def fake_decode(audio, directory):
    count = int(audio.decode().split(':')[1])
    path = directory / 'chunk.wav'
    with wave.open(str(path), 'wb') as out:
        out.setparams((1, 2, eleven.RATE, 0, 'NONE', 'not compressed'))
        out.writeframes(b'\0\0' * count)
    return path, count


def fake_encode(combined, destination, frames):
    destination.write_bytes(b'synthetic-test-aac:' + str(frames).encode())
    return frames / eleven.RATE


class ProviderTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / 'content').mkdir()
        self.course = {'modules': [{'id': '01-one', 'title': 'One'}, {'id': '02-two', 'title': 'Two'}],
                       'guides': {'index': 'Index', 'roadmap': 'Map'}}
        (self.root / 'course.json').write_text(json.dumps(self.course))
        for name in ('01-one', '02-two'):
            (self.root / 'content' / (name + '.html')).write_text('<p>A readable beginning.</p><h2>Next</h2><p>A second connected paragraph.</p>')

    def plan(self, selection='01-one', **kwargs):
        return eleven.plan_run(self.root, selection, 'ChosenVoice', **kwargs)

    def old_track(self, name='01-one'):
        title = next(m['title'] for m in self.course['modules'] if m['id'] == name)
        source = (self.root / 'content' / (name + '.html')).read_text()
        blocks, _ = passages(title, source)
        destination = self.root / 'work/narration' / name
        destination.mkdir(parents=True, exist_ok=True)
        audio = b'original local track'
        metadata = {'format': 1, 'sourceHash': fingerprint(title, source), 'audioHash': eleven.digest(audio),
                    'provider': 'macos', 'duration': len(blocks),
                    'cues': [{'passage': i, 'text': text, 'start': i, 'end': i + 1} for i, text in enumerate(blocks)]}
        (destination / 'narration.m4a').write_bytes(audio)
        (destination / 'cues.json').write_text(json.dumps(metadata))
        return destination, {p.name: p.read_bytes() for p in destination.iterdir()}

    def media_mocks(self, fetch=None, encode=fake_encode):
        stack = ExitStack()
        stack.enter_context(mock.patch.object(eleven, 'media_tool', return_value='synthetic-media-tool'))
        stack.enter_context(mock.patch.object(eleven, 'decode_mp3', side_effect=fake_decode))
        stack.enter_context(mock.patch.object(eleven, 'encode_track', side_effect=encode))
        stack.enter_context(mock.patch.dict(eleven.os.environ, {'ELEVENLABS_API_KEY': 'test-only-not-a-real-key'}))
        provider = stack.enter_context(mock.patch.object(eleven, 'request_chunk', side_effect=fetch or (lambda spec, key: response_for(spec['body']['text']))))
        stack.enter_context(redirect_stdout(io.StringIO()))
        return stack, provider

    def assert_old(self, destination, old):
        self.assertEqual({p.name: p.read_bytes() for p in destination.iterdir()}, old)

    def test_default_dry_run_writes_only_requested_plan_and_never_reads_credentials(self):
        destination, old = self.old_track()
        output = self.root / 'plan-output'
        with mock.patch.object(eleven, 'ROOT', self.root), mock.patch.object(eleven.os, 'environ') as env, \
                mock.patch.object(eleven, 'request_chunk') as fetch, mock.patch.object(eleven, 'media_tool') as media, redirect_stdout(io.StringIO()):
            def read_environment(name, default=None):
                if name == 'ELEVENLABS_API_KEY':
                    raise AssertionError('dry run read credentials')
                return default  # argparse may ask for locale/terminal presentation settings.
            env.get.side_effect = read_environment
            self.assertEqual(eleven.main(['01-one', '--voice', 'Placeholder', '--sample-passages', '2', '--output-directory', str(output)]), 0)
        fetch.assert_not_called(); media.assert_not_called()
        saved = json.loads((output / 'plan.json').read_text())
        self.assertEqual(saved['pages'][0]['passages'], 2)
        self.assertIn('One\n\nA readable beginning.', (output / '01-one.txt').read_text())
        self.assertIn('previous_text', saved['pages'][0]['chunks'][0]['request']['body'])
        self.assert_old(destination, old)
        self.assertFalse((self.root / 'work/narration-elevenlabs-cache').exists())

    def test_entire_multi_page_budget_is_checked_before_credentials_tools_or_requests(self):
        plan = self.plan('all')
        first_page_cost = sum(len(c['request']['body']['text']) for c in plan['pages'][0]['chunks'])
        with mock.patch.object(eleven.os, 'environ') as env, mock.patch.object(eleven, 'request_chunk') as fetch, mock.patch.object(eleven, 'media_tool') as media:
            env.get.side_effect = AssertionError('ceiling rejection read credentials')
            with self.assertRaisesRegex(eleven.NarrationError, 'Whole run'):
                eleven.generate(self.root, plan, first_page_cost)
        fetch.assert_not_called(); media.assert_not_called()
        self.assertFalse((self.root / 'work').exists())

    def test_section_chunks_limit_invalidation_to_changed_and_neighbor_context(self):
        source = '<p>Before.</p><h2>Middle</h2><p>Same middle.</p><h2>Last</h2><p>Same last.</p>'
        blocks, annotated = passages('Title', source)
        old = eleven.requests_for_chunks(eleven.split_chunks(blocks, annotated), 'v', 'm', eleven.DEFAULT_SETTINGS)
        changed, annotated_changed = passages('Title', source.replace('Before.', 'A longer beginning, still in this section.'))
        new = eleven.requests_for_chunks(eleven.split_chunks(changed, annotated_changed), 'v', 'm', eleven.DEFAULT_SETTINGS)
        self.assertEqual(len(old), 3)
        self.assertNotEqual(old[0]['key'], new[0]['key'])
        self.assertNotEqual(old[1]['key'], new[1]['key'])
        self.assertEqual(old[2]['key'], new[2]['key'])
        self.assertGreater(len(old[0]['spans']), 1)
        for chunk in new:
            self.assertLessEqual(len(chunk['request']['body']['text']), 4000)
            self.assertLessEqual(len(chunk['request']['body']['previous_text']), 250)
            self.assertLessEqual(len(chunk['request']['body']['next_text']), 250)

    def test_cache_identity_covers_model_voice_settings_context_and_output_format(self):
        chunk = self.plan()['pages'][0]['chunks'][0]
        original = chunk['request']
        variants = []
        for field, value in [('voice', 'other'), ('output_format', 'other')]:
            altered = copy.deepcopy(original); altered[field] = value; variants.append(altered)
        for field, value in [('model_id', 'other'), ('previous_text', 'new context'), ('next_text', 'new next'), ('voice_settings', {'speed': 0.9})]:
            altered = copy.deepcopy(original); altered['body'][field] = value; variants.append(altered)
        self.assertTrue(all(eleven.digest(eleven.canonical(v)) != chunk['key'] for v in variants))

    def test_alignment_requires_original_text_and_rejects_corruption_and_overlap(self):
        text = 'A é!'
        good = response_for(text)
        # Spaces and punctuation may have zero duration; original Unicode stays exact.
        good['alignment']['character_end_times_seconds'][1] = good['alignment']['character_start_times_seconds'][1]
        good['alignment']['character_end_times_seconds'][3] = good['alignment']['character_start_times_seconds'][3]
        eleven.validated_response(good, text, 1000)
        bads = []
        value = copy.deepcopy(good); value['alignment']['characters'][2] = 'e'; bads.append(value)
        value = copy.deepcopy(good); value['alignment']['character_start_times_seconds'][2] = float('nan'); bads.append(value)
        value = copy.deepcopy(good); value['alignment']['character_end_times_seconds'][2] = float('inf'); bads.append(value)
        value = copy.deepcopy(good); value['alignment']['character_start_times_seconds'][2] = 0; bads.append(value)
        value = copy.deepcopy(good); value['alignment']['character_end_times_seconds'].pop(); bads.append(value)
        value = copy.deepcopy(good); value['normalized_alignment'] = value.pop('alignment'); bads.append(value)
        value = copy.deepcopy(good); value['audio_base64'] = '%%%'; bads.append(value)
        value = copy.deepcopy(good); value['audio_base64'] = ''; bads.append(value)
        for value in bads:
            with self.subTest(value=value), self.assertRaises(eleven.NarrationError):
                eleven.validated_response(value, text, 1000)
        with self.assertRaisesRegex(eleven.NarrationError, 'beyond'):
            eleven.validated_response(good, text, 10)

    def test_long_passage_fragments_merge_into_exact_contiguous_passage_cues(self):
        source = '<p>' + ('a long phrase ' * 450) + '</p><p>End.</p>'
        (self.root / 'content/01-one.html').write_text(source)
        plan = self.plan()
        self.assertGreater(len(plan['pages'][0]['chunks']), 1)
        stack, fetch = self.media_mocks()
        with stack:
            eleven.generate(self.root, plan, plan['generationCharacters'])
        result = read_current(plan['pages'][0]['destination'], 'One', source)
        self.assertIsNotNone(result)
        cues = result[0]['cues']
        self.assertEqual([c['text'] for c in cues], passages('One', source)[0])
        self.assertEqual(len(cues), 3)
        self.assertEqual(cues[0]['start'], 0)
        self.assertEqual([c['end'] for c in cues[:-1]], [c['start'] for c in cues[1:]])
        self.assertEqual(cues[-1]['end'], result[0]['pcmFrames'] / eleven.RATE)
        self.assertGreater(fetch.call_count, 1)

    def test_successful_chunks_are_cached_and_a_stale_plan_does_not_pay_again(self):
        first = self.plan(); stale = self.plan()
        stack, fetch = self.media_mocks()
        with stack:
            eleven.generate(self.root, first, first['generationCharacters'])
            initial_calls = fetch.call_count
            eleven.generate(self.root, stale, stale['generationCharacters'])
            self.assertEqual(fetch.call_count, initial_calls)
            current = self.plan()
            self.assertTrue(current['pages'][0]['retained'])
        with mock.patch.object(eleven, 'media_tool') as media, mock.patch.object(eleven.os, 'environ') as env:
            env.get.side_effect = AssertionError('retained run read credentials')
            eleven.generate(self.root, current, 0)
        media.assert_not_called()
        self.assertEqual(current['generationCharacters'], 0)
        changed = self.plan(model='different_model')
        self.assertFalse(changed['pages'][0]['retained'])
        self.assertGreater(changed['generationCharacters'], 0)

    def test_failure_after_one_paid_chunk_keeps_old_track_and_caches_completed_work(self):
        destination, old = self.old_track()
        plan = self.plan()
        count = 0
        def fail_second(spec, key):
            nonlocal count
            count += 1
            if count == 2:
                raise eleven.NarrationError('simulated timeout; no retry')
            return response_for(spec['body']['text'])
        stack, fetch = self.media_mocks(fail_second)
        with stack, self.assertRaisesRegex(eleven.NarrationError, 'timeout'):
            eleven.generate(self.root, plan, plan['generationCharacters'])
        self.assertEqual(fetch.call_count, 2)
        self.assert_old(destination, old)
        resumed = self.plan()
        self.assertGreater(resumed['generationCharacters'], 0)
        self.assertLess(resumed['generationCharacters'], plan['generationCharacters'])
        self.assertFalse((self.root / 'work/narration-elevenlabs.lock').exists())

    def test_decode_failure_keeps_old_track_and_does_not_cache_invalid_audio(self):
        destination, old = self.old_track(); plan = self.plan()
        stack, fetch = self.media_mocks()
        with stack, mock.patch.object(eleven, 'decode_mp3', side_effect=eleven.NarrationError('bad audio')), self.assertRaises(eleven.NarrationError):
            eleven.generate(self.root, plan, plan['generationCharacters'])
        self.assertEqual(fetch.call_count, 1)
        self.assert_old(destination, old)
        self.assertFalse((self.root / 'work/narration-elevenlabs-cache').exists())

    def test_prose_or_registry_title_change_during_encoding_preserves_old_track(self):
        for change in ('source', 'title'):
            with self.subTest(change=change):
                destination, old = self.old_track(); plan = self.plan()
                def changed(combined, output, frames):
                    result = fake_encode(combined, output, frames)
                    if change == 'source':
                        (self.root / 'content/01-one.html').write_text('<p>Changed during generation.</p>')
                    else:
                        course = copy.deepcopy(self.course); course['modules'][0]['title'] = 'New title'
                        (self.root / 'course.json').write_text(json.dumps(course))
                    return result
                stack, _ = self.media_mocks(encode=changed)
                with stack, self.assertRaisesRegex(eleven.NarrationError, 'changed'):
                    eleven.generate(self.root, plan, plan['generationCharacters'])
                self.assert_old(destination, old)
                (self.root / 'course.json').write_text(json.dumps(self.course))

    def test_directory_publication_restores_original_pair_if_commit_fails(self):
        destination, old = self.old_track()
        work = self.root / 'staging'; staged = work / 'track'; staged.mkdir(parents=True)
        (staged / 'narration.m4a').write_bytes(b'new audio'); (staged / 'cues.json').write_text('{}')
        real_replace = eleven.os.replace
        def fail_commit(a, b):
            if Path(a) == staged:
                raise OSError('simulated replace failure')
            return real_replace(a, b)
        with mock.patch.object(eleven.os, 'replace', side_effect=fail_commit), self.assertRaises(OSError):
            eleven.publish_track(staged, destination, lambda: None)
        self.assert_old(destination, old)

    def test_audition_never_replaces_full_track_and_sample_mapping_is_explicit(self):
        destination, old = self.old_track(); plan = self.plan(sample_passages=2)
        stack, _ = self.media_mocks()
        with stack:
            eleven.generate(self.root, plan, plan['generationCharacters'])
        self.assert_old(destination, old)
        audition = plan['pages'][0]
        self.assertIn('narration-auditions', str(audition['destination']))
        result = read_current(audition['destination'], 'One', audition['validationSource'])
        self.assertEqual(result[0]['samplePassages'], 2)
        self.assertEqual(result[0]['fullSourceHash'], audition['sourceHash'])
        self.assertEqual([c['passage'] for c in result[0]['cues']], [0, 1])
        with self.assertRaisesRegex(eleven.NarrationError, 'one named page'):
            self.plan('all', sample_passages=2)

    def test_corrupt_cached_alignment_aborts_planning_without_paid_replacement(self):
        plan = self.plan(); chunk = plan['pages'][0]['chunks'][0]
        response = response_for(chunk['request']['body']['text'])
        cache = self.root / 'work/narration-elevenlabs-cache'
        eleven.cache_write(cache, chunk, response, len(chunk['request']['body']['text']) * 100 + 200)
        path = cache / (chunk['key'] + '.json'); saved = json.loads(path.read_text())
        saved['response']['alignment']['characters'][0] = 'X'; path.write_text(json.dumps(saved))
        with mock.patch.object(eleven, 'request_chunk') as fetch, self.assertRaisesRegex(eleven.NarrationError, 'no automatic paid replacement'):
            self.plan()
        fetch.assert_not_called()

    def test_existing_generation_lock_does_not_get_removed_or_send_requests(self):
        plan = self.plan(); lock = self.root / 'work/narration-elevenlabs.lock'; lock.parent.mkdir(); lock.write_text('other process')
        stack, fetch = self.media_mocks()
        with stack, self.assertRaisesRegex(eleven.NarrationError, 'holds'):
            eleven.generate(self.root, plan, plan['generationCharacters'])
        fetch.assert_not_called(); self.assertEqual(lock.read_text(), 'other process')

    def test_http_failures_are_sanitized_once_and_redirects_are_disabled(self):
        spec = self.plan()['pages'][0]['chunks'][0]['request']
        for failure in (HTTPError('https://example.invalid', 302, 'header-secret', {}, None), TimeoutError('header-secret')):
            opener = mock.Mock(); opener.open.side_effect = failure
            with mock.patch.object(eleven.request, 'build_opener', return_value=opener) as build:
                with self.assertRaises(eleven.NarrationError) as caught:
                    eleven.request_chunk(spec, 'header-secret')
            self.assertNotIn('header-secret', str(caught.exception))
            self.assertEqual(opener.open.call_count, 1)
            self.assertIsInstance(build.call_args.args[0], eleven.NoRedirect)
            req = opener.open.call_args.args[0]
            self.assertTrue(req.full_url.startswith('https://api.elevenlabs.io/v1/text-to-speech/ChosenVoice/with-timestamps?'))
            self.assertEqual(json.loads(req.data), spec['body'])
            self.assertIsNone(eleven.NoRedirect().redirect_request(req, None, 302, '', {}, 'https://other.invalid'))

    @unittest.skipUnless(shutil.which('ffmpeg') and shutil.which('ffprobe'), 'local codec integration requires ffmpeg and ffprobe')
    def test_real_mp3_pcm_concatenation_and_aac_keep_measured_cues(self):
        self.course['modules'] = [{'id': '01-one', 'title': 'A'}]
        (self.root / 'course.json').write_text(json.dumps(self.course))
        (self.root / 'content/01-one.html').write_text('<h2>B</h2>')
        tone = self.root / 'tone.wav'; encoded = self.root / 'tone.mp3'
        frames = eleven.RATE // 4
        samples = b''.join(struct.pack('<h', round(2000 * math.sin(i * 2 * math.pi * 440 / eleven.RATE))) for i in range(frames))
        with wave.open(str(tone), 'wb') as out:
            out.setparams((1, 2, eleven.RATE, 0, 'NONE', 'not compressed')); out.writeframes(samples)
        subprocess.run([shutil.which('ffmpeg'), '-v', 'error', '-nostdin', '-y', '-i', str(tone), '-c:a', 'libmp3lame', '-b:a', '128k', str(encoded)], check=True, capture_output=True)
        audio = base64.b64encode(encoded.read_bytes()).decode()
        def synthetic_provider(spec, key):
            text = spec['body']['text']; self.assertIn(text, {'A', 'B'})
            return {'audio_base64': audio, 'alignment': {'characters': list(text), 'character_start_times_seconds': [0.02], 'character_end_times_seconds': [0.20]}}
        plan = self.plan()
        with mock.patch.object(eleven, 'request_chunk', side_effect=synthetic_provider) as fetch, mock.patch.dict(eleven.os.environ, {'ELEVENLABS_API_KEY': 'synthetic-only'}), redirect_stdout(io.StringIO()):
            eleven.generate(self.root, plan, 2)
        self.assertEqual(fetch.call_count, 2)
        accepted = read_current(plan['pages'][0]['destination'], 'A', '<h2>B</h2>')
        self.assertIsNotNone(accepted)
        self.assertEqual(accepted[0]['pcmFrames'], 2 * frames)
        self.assertEqual(accepted[0]['duration'], .5)
        self.assertEqual([(c['start'], c['end']) for c in accepted[0]['cues']], [(0, .25), (.25, .5)])
        self.assertAlmostEqual(accepted[0]['encodedDuration'], .5, delta=.047)


if __name__ == '__main__':
    unittest.main()
