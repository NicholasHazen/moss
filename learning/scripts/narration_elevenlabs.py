#!/usr/bin/env python3
"""Plan optional ElevenLabs narration; generation is explicit and budget-limited.

Dry runs need neither credentials nor media tools. --max-characters bounds new
request text across the entire run, NOT currency or provider billing. Context is
reported separately. No request is automatically retried, including timeouts.
API contract checked 2026-09-23:
https://elevenlabs.io/docs/api-reference/text-to-speech/convert-with-timestamps
https://elevenlabs.io/docs/api-reference/voices/settings/get-default
"""
from __future__ import annotations

import argparse
import base64
import binascii
from contextlib import contextmanager
import hashlib
import html
from html.parser import HTMLParser
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from urllib import error, parse, request
import wave

from narration import FORMAT, fingerprint, passages, read_current

ROOT = Path(__file__).resolve().parents[1]
RATE = 44100
CHUNK_LIMIT = 4000
CONTEXT_LIMIT = 250
OUTPUT_FORMAT = "mp3_44100_128"
DEFAULT_MODEL = "eleven_multilingual_v2"
DEFAULT_SETTINGS = {"stability": 0.5, "similarity_boost": 0.75, "style": 0.0,
                    "use_speaker_boost": True, "speed": 1.0}


class NarrationError(ValueError):
    pass


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def settings_from_json(text):
    try:
        overrides = json.loads(text)
    except ValueError as exc:
        raise NarrationError("Voice settings must be a JSON object") from exc
    if not isinstance(overrides, dict) or overrides.keys() - DEFAULT_SETTINGS.keys():
        raise NarrationError("Use only stability, similarity_boost, style, use_speaker_boost, speed")
    settings = DEFAULT_SETTINGS | overrides
    for name in ("stability", "similarity_boost", "style", "speed"):
        value = settings[name]
        low, high = (0.7, 1.2) if name == "speed" else (0, 1)
        if type(value) not in {int, float} or not math.isfinite(value) or not low <= value <= high:
            raise NarrationError(f"{name} must be between {low} and {high}")
    if type(settings["use_speaker_boost"]) is not bool:
        raise NarrationError("use_speaker_boost must be true or false")
    return settings


class Headings(HTMLParser):
    def __init__(self):
        super().__init__()
        self.starts = {0}

    def handle_starttag(self, tag, attrs):
        number = dict(attrs).get("data-narration")
        if tag in {"h2", "h3"} and number is not None:
            self.starts.add(int(number))


def split_chunks(blocks, annotated, limit=CHUNK_LIMIT):
    """Keep sections stable; spans refer to original passage IDs and characters."""
    parser = Headings()
    parser.feed(annotated)
    starts = sorted(n for n in parser.starts if n < len(blocks)) + [len(blocks)]
    chunks = []
    for first, last in zip(starts, starts[1:]):
        text, spans = "", []
        for number in range(first, last):
            if text:
                text += "\n\n"
            begin = len(text)
            text += blocks[number]
            spans.append({"passage": number, "start": begin, "end": len(text)})
        cursor = 0
        while cursor < len(text):
            end = min(cursor + limit, len(text))
            if end < len(text):
                boundaries = [s["start"] for s in spans if cursor < s["start"] <= end]
                if boundaries:
                    end = max(boundaries)
                else:
                    space = text.rfind(" ", cursor + 1, end)
                    if space > cursor:
                        end = space + 1
            selected = [{"passage": s["passage"], "start": max(cursor, s["start"]) - cursor,
                         "end": min(end, s["end"]) - cursor}
                        for s in spans if max(cursor, s["start"]) < min(end, s["end"])]
            if not selected or not text[cursor:end].strip():
                raise NarrationError("An empty narration chunk cannot be aligned")
            chunks.append({"text": text[cursor:end], "spans": selected})
            cursor = end
    return chunks


def requests_for_chunks(chunks, voice, model, settings):
    planned = []
    for i, chunk in enumerate(chunks):
        body = {"text": chunk["text"], "model_id": model, "voice_settings": settings,
                "previous_text": chunks[i-1]["text"][-CONTEXT_LIMIT:] if i else "",
                "next_text": chunks[i+1]["text"][:CONTEXT_LIMIT] if i+1 < len(chunks) else "",
                "apply_text_normalization": "auto"}
        spec = {"voice": voice, "output_format": OUTPUT_FORMAT, "body": body}
        planned.append({"key": digest(canonical(spec)), "request": spec, "spans": chunk["spans"]})
    return planned


def validated_response(response, text, frames=None):
    """Use original alignment, never normalized text guessed back onto prose."""
    try:
        if not isinstance(response, dict) or not isinstance(response.get("audio_base64"), str):
            raise NarrationError("Missing audio_base64 in provider response")
        audio = base64.b64decode(response["audio_base64"], validate=True)
        alignment = response.get("alignment")
        if not audio or not isinstance(alignment, dict):
            raise NarrationError("Empty audio or missing original-character alignment")
        chars = alignment.get("characters")
        starts = alignment.get("character_start_times_seconds")
        ends = alignment.get("character_end_times_seconds")
        if (not isinstance(chars, list) or not chars
                or any(not isinstance(c, str) or len(c) != 1 for c in chars)
                or "".join(chars) != text or not isinstance(starts, list) or not isinstance(ends, list)
                or len(chars) != len(starts) or len(chars) != len(ends)):
            raise NarrationError("Original alignment does not exactly match request text")
        previous = 0.0
        for start, end in zip(starts, ends):
            if (type(start) not in {int, float} or type(end) not in {int, float}
                    or not math.isfinite(start) or not math.isfinite(end)
                    or start < previous or end < start):
                raise NarrationError("Character alignment is nonfinite, negative, reversed or overlapping")
            previous = end
        if not any(end > start for c, start, end in zip(chars, starts, ends) if not c.isspace()):
            raise NarrationError("Alignment contains no timed speech")
        if frames is not None and (type(frames) is not int or frames <= 0 or ends[-1] > (frames + 1) / RATE):
            raise NarrationError("Alignment extends beyond decoded PCM audio")
        return audio, starts
    except (binascii.Error, UnicodeError) as exc:
        raise NarrationError("Provider audio is not valid base64") from exc


def cache_read(directory, chunk):
    path = directory / (chunk["key"] + ".json")
    if not path.exists():
        return None
    try:
        saved = json.loads(path.read_text())
        if (not isinstance(saved, dict) or saved.get("format") != 1
                or saved.get("request") != chunk["request"]):
            raise NarrationError("Cache request mismatch")
        if type(saved.get("frames")) is not int or saved["frames"] <= 0:
            raise NarrationError("Cached frame count is invalid")
        audio, _ = validated_response(saved["response"], chunk["request"]["body"]["text"], saved["frames"])
        if digest(audio) != saved.get("audioHash"):
            raise NarrationError("Cached audio digest mismatch")
        return saved
    except (OSError, ValueError, KeyError, TypeError) as exc:
        raise NarrationError(f"Invalid cached chunk {path}; no automatic paid replacement") from exc


def cache_write(directory, chunk, response, frames):
    directory.mkdir(parents=True, exist_ok=True)
    audio, _ = validated_response(response, chunk["request"]["body"]["text"], frames)
    saved = {"format": 1, "request": chunk["request"], "response": response,
             "frames": frames, "audioHash": digest(audio)}
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=directory, delete=False) as handle:
            temporary = Path(handle.name)
            handle.write(canonical(saved))
        os.replace(temporary, directory / (chunk["key"] + ".json"))
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    return saved


def registered_pages(root):
    course = json.loads((root / "course.json").read_text())
    return course["modules"] + [{"id": name, "title": title} for name, title in course.get("guides", {}).items()
                                if name not in {"index", "roadmap"}]


def plan_run(root, selection, voice, model=DEFAULT_MODEL, settings=None, sample_passages=None):
    settings = dict(DEFAULT_SETTINGS if settings is None else settings)
    selected = [p for p in registered_pages(root) if selection == "all" or p["id"] == selection]
    if not selected:
        raise NarrationError("Choose a registered lesson or static guide; index and roadmap are excluded")
    if sample_passages is not None and (type(sample_passages) is not int or sample_passages < 1 or selection == "all"):
        raise NarrationError("--sample-passages needs a positive count and one named page")
    pages, caches, missing = [], {}, {}
    cache_directory = root / "work/narration-elevenlabs-cache"
    for module in selected:
        source = (root / "content" / (module["id"] + ".html")).read_text()
        blocks, annotated = passages(module["title"], source)
        source_count = len(blocks)
        if sample_passages is not None:
            blocks = blocks[:sample_passages]
        chunks = requests_for_chunks(split_chunks(blocks, annotated), voice, model, settings)
        plan_hash = digest(canonical({"chunks": [c["key"] for c in chunks], "samplePassages": sample_passages}))
        destination = (root / "work/narration" / module["id"] if sample_passages is None else
                       root / "work/narration-auditions" / module["id"] / plan_hash)
        validation_source = source if sample_passages is None else "".join(f"<p>{html.escape(b)}</p>" for b in blocks[1:])
        current = read_current(destination, module["title"], validation_source)
        retained = bool(current and current[0].get("provider") == "elevenlabs"
                        and current[0].get("planHash") == plan_hash
                        and (sample_passages is None or current[0].get("fullSourceHash") == fingerprint(module["title"], source)))
        page = {"id": module["id"], "title": module["title"], "source": source,
                "sourceHash": fingerprint(module["title"], source), "validationSource": validation_source,
                "blocks": blocks, "sourcePassageCount": source_count, "chunks": chunks,
                "planHash": plan_hash, "destination": destination, "retained": retained}
        pages.append(page)
        if not retained:
            for chunk in chunks:
                if chunk["key"] not in caches:
                    caches[chunk["key"]] = cache_read(cache_directory, chunk)
                if caches[chunk["key"]] is None:
                    missing[chunk["key"]] = chunk
    return {"pages": pages, "cache": caches, "missing": missing, "voice": voice, "model": model,
            "settings": settings, "samplePassages": sample_passages,
            "generationCharacters": sum(len(c["request"]["body"]["text"]) for c in missing.values()),
            "contextCharacters": sum(len(c["request"]["body"]["previous_text"]) + len(c["request"]["body"]["next_text"])
                                     for c in missing.values())}


def public_plan(plan):
    return {"provider": "elevenlabs", "voice": plan["voice"], "model": plan["model"],
            "settings": plan["settings"], "outputFormat": OUTPUT_FORMAT,
            "maximumChunkCharacters": CHUNK_LIMIT, "maximumContextCharactersPerSide": CONTEXT_LIMIT,
            "samplePassages": plan["samplePassages"], "uncachedGenerationCharacters": plan["generationCharacters"],
            "uncachedContextCharacters": plan["contextCharacters"], "uncachedRequests": len(plan["missing"]),
            "budgetMeaning": "New request text characters, not a currency or billing guarantee; context is separate.",
            "pages": [{"id": p["id"], "sourceHash": p["sourceHash"], "passages": len(p["blocks"]),
                       "retained": p["retained"], "destination": str(p["destination"]),
                       "chunks": [c | {"cached": plan["cache"].get(c["key"]) is not None} for c in p["chunks"]]} for p in plan["pages"]]}


class NoRedirect(request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request_chunk(spec, api_key):
    """One request only. Never log request headers, credentials, or response bodies."""
    url = "https://api.elevenlabs.io/v1/text-to-speech/" + parse.quote(spec["voice"], safe="") + "/with-timestamps"
    url += "?" + parse.urlencode({"output_format": spec["output_format"]})
    req = request.Request(url, data=canonical(spec["body"]), method="POST",
                          headers={"Content-Type": "application/json", "xi-api-key": api_key})
    try:
        with request.build_opener(NoRedirect()).open(req, timeout=120) as response:
            raw = response.read(32 * 1024 * 1024 + 1)
        if len(raw) > 32 * 1024 * 1024:
            raise NarrationError("Provider response exceeds the bounded chunk response size")
        result = json.loads(raw)
        validated_response(result, spec["body"]["text"])
        # Ignore normalized alignment and any unrelated provider response fields.
        return {"audio_base64": result["audio_base64"], "alignment": result["alignment"]}
    except error.HTTPError as exc:
        raise NarrationError(f"ElevenLabs HTTP {exc.code}; no retry attempted") from None
    except (error.URLError, TimeoutError, OSError) as exc:
        raise NarrationError("ElevenLabs request failed or timed out; no retry attempted") from None
    except (ValueError, UnicodeError) as exc:
        raise NarrationError("ElevenLabs returned invalid audio/alignment; no retry attempted") from None


def media_tool(name):
    path = shutil.which(name)
    if not path and (Path("/opt/homebrew/bin") / name).is_file():
        path = str(Path("/opt/homebrew/bin") / name)
    if not path:
        raise NarrationError(f"Generation needs {name} on PATH")
    return path


def media_command(command):
    try:
        subprocess.run(command, check=True, capture_output=True, timeout=180)
    except (OSError, subprocess.SubprocessError) as exc:
        raise NarrationError("Audio conversion failed; the previous published track was retained") from exc


def decode_mp3(audio, directory):
    encoded, decoded = directory / "chunk.mp3", directory / "chunk.wav"
    encoded.write_bytes(audio)
    media_command([media_tool("ffmpeg"), "-v", "error", "-nostdin", "-y", "-protocol_whitelist", "file,pipe", "-f", "mp3", "-i", str(encoded),
                   "-map", "0:a:0", "-ac", "1", "-ar", str(RATE), "-c:a", "pcm_s16le", str(decoded)])
    try:
        with wave.open(str(decoded), "rb") as segment:
            if (segment.getnchannels(), segment.getsampwidth(), segment.getframerate()) != (1, 2, RATE) or segment.getnframes() <= 0:
                raise NarrationError("Decoded chunk is not nonempty mono 44100-Hz PCM")
            frames = segment.getnframes()
            if len(segment.readframes(frames)) != frames * 2:
                raise NarrationError("Decoded PCM data is truncated")
    except (OSError, wave.Error, EOFError) as exc:
        raise NarrationError("Decoded chunk is not readable PCM") from exc
    return decoded, frames


def encode_track(combined, output, frames):
    media_command([media_tool("ffmpeg"), "-v", "error", "-nostdin", "-y", "-i", str(combined),
                   "-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart", str(output)])
    try:
        probe = subprocess.run([media_tool("ffprobe"), "-v", "error", "-show_entries",
                                "stream=codec_name,sample_rate,channels,duration", "-of", "json", str(output)],
                               check=True, capture_output=True, timeout=30)
        streams = json.loads(probe.stdout)["streams"]
        duration = float(streams[0]["duration"])
        if (len(streams) != 1 or streams[0]["codec_name"] != "aac" or streams[0]["sample_rate"] != str(RATE)
                or streams[0]["channels"] != 1 or not math.isfinite(duration)
                or duration <= 0 or abs(duration - frames / RATE) > 2 * 1024 / RATE
                or not output.read_bytes()):
            raise NarrationError("Encoded AAC timing or stream format differs from measured PCM")
        return duration
    except (OSError, subprocess.SubprocessError, ValueError, KeyError, IndexError, TypeError) as exc:
        raise NarrationError("Final AAC verification failed; the previous track was retained") from exc


def chunk_cues(chunk, response, frames, offset, blocks):
    _, starts = validated_response(response, chunk["request"]["body"]["text"], frames)
    spans = chunk["spans"]
    boundaries = [0] + [round(starts[s["start"]] * RATE) for s in spans[1:]] + [frames]
    if any(a >= b for a, b in zip(boundaries, boundaries[1:])):
        raise NarrationError("Alignment cannot give every original passage a positive contiguous cue")
    return [{"passage": s["passage"], "text": blocks[s["passage"]],
             "start": (offset + a) / RATE, "end": (offset + b) / RATE}
            for s, a, b in zip(spans, boundaries, boundaries[1:])]


def assert_source_current(root, page):
    current = next((p for p in registered_pages(root) if p["id"] == page["id"]), None)
    source = (root / "content" / (page["id"] + ".html")).read_text()
    if current is None or fingerprint(current["title"], source) != page["sourceHash"]:
        raise NarrationError(f"{page['id']}: title or prose changed; no new track published")


def publish_track(staged, destination, verify):
    """Move complete pairs together and restore the old directory on failure."""
    backup = staged.parent / "previous-track"
    verify()
    had_previous = destination.exists()
    try:
        if had_previous:
            os.replace(destination, backup)
        verify()
        os.replace(staged, destination)
    except BaseException:
        if had_previous and backup.exists():
            os.replace(backup, destination)
        raise
    if backup.exists():
        shutil.rmtree(backup, ignore_errors=True)


@contextmanager
def generation_lock(root):
    path = root / "work/narration-elevenlabs.lock"
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        handle = path.open("x")
    except FileExistsError as exc:
        raise NarrationError(f"Another generation or interrupted run holds {path}; inspect before removing it") from exc
    with handle:
        handle.write(str(os.getpid()))
    try:
        yield
    finally:
        path.unlink()


def generate(root, plan, max_characters):
    if type(max_characters) is not int or max_characters < 0:
        raise NarrationError("--generate requires a nonnegative --max-characters ceiling")
    if plan["generationCharacters"] > max_characters:
        raise NarrationError(f"Whole run needs {plan['generationCharacters']} uncached request characters; ceiling is {max_characters}. No requests sent")
    if all(page["retained"] for page in plan["pages"]):
        return
    media_tool("ffmpeg"); media_tool("ffprobe")
    with generation_lock(root):
        for page in plan["pages"]:
            assert_source_current(root, page)
        # A different completed invocation may have filled the cache after this
        # plan was read. Recheck under the lock rather than paying for it again.
        pending = {}
        for page in plan["pages"]:
            if not page["retained"]:
                for chunk in page["chunks"]:
                    if plan["cache"].get(chunk["key"]) is None:
                        plan["cache"][chunk["key"]] = cache_read(root / "work/narration-elevenlabs-cache", chunk)
                    if plan["cache"][chunk["key"]] is None:
                        pending[chunk["key"]] = chunk
        required = sum(len(c["request"]["body"]["text"]) for c in pending.values())
        if required > max_characters:
            raise NarrationError("Whole-run request text exceeds the ceiling; no requests sent")
        # Read only this variable, only when the explicit run needs requests.
        api_key = os.environ.get("ELEVENLABS_API_KEY") if pending else None
        if pending and not api_key:
            raise NarrationError("Set ELEVENLABS_API_KEY only when ready for the explicit generated run")
        sent_characters = 0
        for page in plan["pages"]:
            if page["retained"]:
                continue
            destination = page["destination"]
            destination.parent.mkdir(parents=True, exist_ok=True)
            with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
                work = Path(temporary)
                staged = work / "track"
                staged.mkdir()
                frames, cues = 0, []
                with wave.open(str(work / "combined.wav"), "wb") as combined:
                    combined.setnchannels(1); combined.setsampwidth(2); combined.setframerate(RATE)
                    for chunk in page["chunks"]:
                        assert_source_current(root, page)
                        saved = plan["cache"].get(chunk["key"])
                        if saved:
                            response = saved["response"]
                        else:
                            sent_characters += len(chunk["request"]["body"]["text"])
                            if sent_characters > max_characters:
                                raise NarrationError("Request ceiling reached; no further requests sent")
                            response = request_chunk(chunk["request"], api_key)
                        audio, _ = validated_response(response, chunk["request"]["body"]["text"])
                        decoded, count = decode_mp3(audio, work)
                        validated_response(response, chunk["request"]["body"]["text"], count)
                        if saved and count != saved["frames"]:
                            raise NarrationError("Cached chunk's decoded frame count changed; no paid retry")
                        if not saved:
                            saved = cache_write(root / "work/narration-elevenlabs-cache", chunk, response, count)
                            plan["cache"][chunk["key"]] = saved
                        for cue in chunk_cues(chunk, response, count, frames, page["blocks"]):
                            if cues and cues[-1]["passage"] == cue["passage"]:
                                cues[-1]["end"] = cue["end"]
                            else:
                                cues.append(cue)
                        with wave.open(str(decoded), "rb") as segment:
                            combined.writeframes(segment.readframes(count))
                        frames += count
                encoded_duration = encode_track(work / "combined.wav", staged / "narration.m4a", frames)
                audio = (staged / "narration.m4a").read_bytes()
                metadata = {"format": FORMAT, "sourceHash": fingerprint(page["title"], page["validationSource"]),
                            "audioHash": digest(audio), "provider": "elevenlabs", "voice": plan["voice"],
                            "model": plan["model"], "settings": plan["settings"], "planHash": page["planHash"],
                            "outputFormat": OUTPUT_FORMAT, "duration": frames / RATE,
                            "encodedDuration": encoded_duration, "sampleRate": RATE, "pcmFrames": frames, "cues": cues}
                if plan["samplePassages"] is not None:
                    metadata.update(samplePassages=len(page["blocks"]), sourcePassageCount=page["sourcePassageCount"],
                                    fullSourceHash=page["sourceHash"])
                (staged / "cues.json").write_text(json.dumps(metadata, ensure_ascii=False, allow_nan=False) + "\n")
                if read_current(staged, page["title"], page["validationSource"]) is None:
                    raise NarrationError("Final original-passage cue validation failed; old track retained")
                publish_track(staged, destination, lambda: assert_source_current(root, page))
                print(f"{page['id']}: {len(cues)} passages, {frames / RATE:.1f} seconds → {destination}", flush=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("lesson", help="registered lesson/static guide ID, or all")
    parser.add_argument("--voice", required=True, help="chosen ElevenLabs voice ID; no voice lookup is performed")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--voice-settings", default="{}", help="JSON overrides for explicit per-request voice settings")
    parser.add_argument("--generate", action="store_true", help="explicitly permit paid API requests within the character ceiling")
    parser.add_argument("--max-characters", type=int, help="hard whole-run ceiling for uncached request text; not a currency cap")
    parser.add_argument("--sample-passages", type=int, help="first N passages including title, one page, separate audition output")
    parser.add_argument("--output-directory", type=Path, help="dry run only: write exact request plan and selected transcripts")
    args = parser.parse_args(argv)
    try:
        if not all(re.fullmatch(r"[A-Za-z0-9_-]{1,128}", token) for token in (args.voice, args.model)):
            raise NarrationError("Use a voice ID and model ID, not a URL")
        if args.generate and args.output_directory:
            raise NarrationError("--output-directory is only for dry-run plans; audition/full destinations are fixed")
        if args.generate and (args.max_characters is None or args.max_characters < 0):
            raise NarrationError("--generate requires nonnegative --max-characters")
        plan = plan_run(ROOT, args.lesson, args.voice, args.model, settings_from_json(args.voice_settings), args.sample_passages)
        summary = public_plan(plan)
        print(json.dumps({k: v for k, v in summary.items() if k != "pages"}, ensure_ascii=False, indent=2))
        for page in plan["pages"]:
            print(f"{page['id']}: {len(page['blocks'])} passages, {len(page['chunks'])} chunks; "
                  + ("current track retained" if page["retained"] else "planned"))
        if not args.generate:
            if args.output_directory:
                args.output_directory.mkdir(parents=True, exist_ok=True)
                (args.output_directory / "plan.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
                for page in plan["pages"]:
                    (args.output_directory / (page["id"] + ".txt")).write_text("\n\n".join(page["blocks"]) + "\n")
            print("Dry run only: no credentials read, network requests, audio generation, or track changes.")
            return 0
        generate(ROOT, plan, args.max_characters)
        return 0
    except (NarrationError, OSError, ValueError) as exc:
        parser.error(str(exc))


if __name__ == "__main__":
    main()
