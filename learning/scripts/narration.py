#!/usr/bin/env python3
"""Optionally render original course prose with an installed macOS voice.

The ordinary static build needs no audio tools. Generation requires macOS say
and afconvert, and never downloads a voice. Code and answer disclosures are
omitted; tables are spoken with column context. Audio is generated, not a human
performance. Each paragraph has a cue measured from its actual PCM frames.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import subprocess
import tempfile
import wave
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FORMAT = 1
VOID = {"br", "hr", "img", "input", "meta", "link", "source", "wbr"}


def fingerprint(title, source):
    return hashlib.sha256(f"{FORMAT}\0{title}\0{source}".encode()).hexdigest()


class Passages(HTMLParser):
    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.source = source
        self.stack = []
        self.blocks = []
        self.active = None
        self.parts = []
        self.insertions = []
        self.line_offsets = [0]
        for line in source.splitlines(keepends=True):
            self.line_offsets.append(self.line_offsets[-1] + len(line))
        self.headers = []
        self.rows = []
        self.row = None
        self.cell = None
        self.caption = None

    def handle_starttag(self, tag, attrs):
        if self.active and tag in {"br", "p", "li"}:
            self.parts.append(" ")
        excluded = any(t in {"pre", "details", "script", "style", "svg"} for t in self.stack)
        if not excluded and self.active is None and tag in {"p", "h2", "h3", "li", "figcaption", "table"}:
            self.active = (tag, len(self.stack))
            self.parts = []
            line, column = self.getpos()
            self.insertions.append((self.line_offsets[line-1] + column + len(self.get_starttag_text()) - 1, len(self.blocks)+1))
            if tag == "table":
                self.headers, self.rows, self.row, self.cell, self.caption = [], [], None, None, None
        if self.active and self.active[0] == "table":
            if tag == "tr": self.row = []
            elif tag in {"td", "th"}: self.cell = []
            elif tag == "caption": self.caption = []
        if tag not in VOID:
            self.stack.append(tag)

    def handle_data(self, text):
        if self.active:
            self.parts.append(text)
            if self.active[0] == "table":
                if self.cell is not None: self.cell.append(text)
                elif self.caption is not None and "caption" in self.stack: self.caption.append(text)

    def handle_endtag(self, tag):
        if self.active and self.active[0] == "table":
            if tag in {"td", "th"} and self.cell is not None:
                self.row.append(" ".join("".join(self.cell).split()))
                self.cell = None
            elif tag == "tr" and self.row is not None:
                if "thead" in self.stack: self.headers = self.row
                else: self.rows.append(self.row)
                self.row = None
        if self.active == (tag, len(self.stack)-1):
            text = " ".join("".join(self.parts).split())
            if tag == "table":
                text = " ".join(self.caption or ["Table"]) + ". " + " ".join(
                    ". ".join(f"{self.headers[i] if i < len(self.headers) else 'Column ' + str(i+1)}: {cell}" for i, cell in enumerate(row)) + "."
                    for row in self.rows)
            if text.strip():
                self.blocks.append(text)
            else:
                # Empty runtime-status elements have nothing to speak. Removing
                # their annotation also keeps later passage IDs contiguous.
                self.insertions.pop()
            self.active = None
        if tag not in VOID and self.stack:
            self.stack.pop()
        if self.active and tag in {"p", "li"}:
            self.parts.append(" ")


def passages(title, source):
    parser = Passages(source)
    parser.feed(source)
    annotated = source
    for position, number in reversed(parser.insertions):
        annotated = annotated[:position] + f' data-narration="{number}"' + annotated[position:]
    return [title] + parser.blocks, annotated


def read_current(directory, title, source):
    """Read one cue/audio snapshot; reject mixed generations and stale prose."""
    try:
        metadata = json.loads((directory / "cues.json").read_text())
        audio = (directory / "narration.m4a").read_bytes()
        if (metadata.get("format") != FORMAT
                or metadata.get("sourceHash") != fingerprint(title, source)
                or metadata.get("audioHash") != hashlib.sha256(audio).hexdigest()):
            return None
        provider = metadata.get("provider", "macos")
        if not isinstance(provider, str) or provider not in {"macos", "elevenlabs"}:
            return None
        blocks, _ = passages(title, source)
        cues = metadata.get("cues")
        if not audio or not isinstance(cues, list) or len(cues) != len(blocks):
            return None
        previous_end = 0
        for index, (cue, text) in enumerate(zip(cues, blocks)):
            if (not isinstance(cue, dict) or type(cue.get("passage")) is not int or cue["passage"] != index
                    or cue.get("text") != text or cue.get("start") != previous_end
                    or type(cue.get("end")) not in {int, float}
                    or not math.isfinite(cue["end"]) or cue["end"] <= previous_end):
                return None
            previous_end = cue["end"]
        if metadata.get("duration") != previous_end:
            return None
        return metadata, audio
    except (OSError, ValueError, AttributeError, OverflowError):
        return None


def render(module, voice, rate, force=False):
    source = (ROOT / "content" / f"{module['id']}.html").read_text()
    blocks, _ = passages(module["title"], source)
    original_hash = fingerprint(module["title"], source)
    destination = ROOT / "work/narration" / module["id"]
    destination.parent.mkdir(parents=True, exist_ok=True)
    if not force and (destination / "narration.m4a").exists() and (destination / "cues.json").exists():
        prior = read_current(destination, module["title"], source)
        if (prior and prior[0].get("provider", "macos") == "macos"
                and prior[0].get("voice") == voice and prior[0].get("rate") == rate):
            print(f"{module['id']}: current narration retained", flush=True)
            return
    with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
        work = Path(temporary)
        cues = []
        frames = 0
        params = (1, 2, 22050)
        with wave.open(str(work / "combined.wav"), "wb") as combined:
            combined.setnchannels(params[0]); combined.setsampwidth(params[1]); combined.setframerate(params[2])
            for number, text in enumerate(blocks):
                (work / "passage.txt").write_text(text)
                audio = work / "passage.wav"
                subprocess.run(["say", "-v", voice, "-r", str(rate), "--file-format=WAVE", "--data-format=LEI16@22050", "-f", str(work / "passage.txt"), "-o", str(audio)], check=True, timeout=90)
                with wave.open(str(audio), "rb") as segment:
                    if segment.getnframes() == 0:
                        raise RuntimeError("Voice service produced empty audio; narration was not published")
                    current = (segment.getnchannels(), segment.getsampwidth(), segment.getframerate())
                    if params != current:
                        raise RuntimeError("Voice output format changed between passages")
                    start = frames / params[2]
                    frames += segment.getnframes()
                    cues.append({"passage":number,"start":start,"end":frames/params[2],"text":text})
                    combined.writeframes(segment.readframes(segment.getnframes()))
        subprocess.run(["afconvert", "-f", "m4af", "-d", "aac", str(work / "combined.wav"), str(work / "narration.m4a")], check=True, timeout=90)
        current_source = (ROOT / "content" / f"{module['id']}.html").read_text()
        if fingerprint(module["title"], current_source) != original_hash:
            raise RuntimeError("Lesson changed during narration; regenerate it")
        audio_bytes = (work / "narration.m4a").read_bytes()
        metadata = {"format":FORMAT,"sourceHash":original_hash,"audioHash":hashlib.sha256(audio_bytes).hexdigest(),"provider":"macos","voice":voice,"rate":rate,"duration":frames/params[2],"cues":cues}
        destination.mkdir(exist_ok=True)
        # Temporary paths are invocation-specific. During the two replacements,
        # a reader either gets a matching pair or rejects its audio digest.
        os.replace(work / "narration.m4a", destination / "narration.m4a")
        (work / "cues.json").write_text(json.dumps(metadata,ensure_ascii=False)+"\n")
        os.replace(work / "cues.json", destination / "cues.json")
        print(f"{module['id']}: {len(cues)} passages, {metadata['duration']:.1f} seconds", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("lesson", help="registered lesson ID or all")
    parser.add_argument("--voice", default="Samantha (English (US))")
    parser.add_argument("--rate", type=int, default=180)
    parser.add_argument("--force", action="store_true", help="regenerate matching audio too")
    args = parser.parse_args()
    if not 100 <= args.rate <= 300:
        parser.error("Choose 100–300 words per minute")
    if not shutil.which("say") or not shutil.which("afconvert"):
        parser.error("Generation requires installed macOS say and afconvert; browser listening remains available")
    course = json.loads((ROOT / "course.json").read_text())
    # Navigation pages expand generated cards/counts during the static build;
    # their raw authoring fragments are not stable narration transcripts.
    pages = course["modules"] + [{"id": name, "title": title} for name, title in course.get("guides", {}).items()
                                 if name not in {"index", "roadmap"}]
    selected = [m for m in pages if args.lesson == "all" or m["id"] == args.lesson]
    if not selected:
        parser.error("Choose a registered lesson or static guide (navigation pages are not narrated)")
    for module in selected:
        render(module,args.voice,args.rate,args.force)


if __name__ == "__main__":
    main()
