#!/usr/bin/env python3
"""Build Fieldnotes using only Python's standard library. No content network calls."""
from __future__ import annotations

import html
import hashlib
import json
import re
import shutil
import difflib
import posixpath
import zipfile
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import quote, unquote, urlsplit, urlunsplit
from runtime import runtime_fingerprint
from narration import read_current as read_narration, passages
from preview import copy_current as copy_preview
from fieldwork_preview import copy_current as copy_fieldwork_preview

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "_site"


class Fragment(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.text = []
        self.examples = {}
        self.current_example = None
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if "id" in a:
            if a["id"] in self.ids:
                raise ValueError(f"Duplicate id: {a['id']}")
            self.ids.add(a["id"])
        example = a.get("data-runnable") or a.get("data-practice-starter") or a.get("data-practice-answer")
        if tag == "code" and example:
            self.current_example = example
            if not re.fullmatch(r"[a-z][a-z0-9-]*", self.current_example):
                raise ValueError("Invalid example id")
            if self.current_example in self.examples:
                raise ValueError("Duplicate example")
            self.examples[self.current_example] = ""
        if tag in {"script", "iframe", "object"}:
            raise ValueError(f"Use the controlled enhancement for {tag}")
        if any(k.startswith("on") for k in a):
            raise ValueError("Inline event handlers are not allowed")

    def handle_endtag(self, tag):
        if tag == "code":
            self.current_example = None

    def handle_data(self, data):
        self.text.append(data)
        if self.current_example:
            self.examples[self.current_example] += data


def esc(value):
    return html.escape(str(value), quote=True)


def relative_url(page, target):
    return quote(posixpath.relpath(target.as_posix(), page.parent.as_posix()), safe="/")


def source_views(checkpoints, guides, assets_version):
    """Add readable wrappers; raw exports and deterministic ZIPs remain unchanged."""
    guide_for = {"shared-meadow": "fieldwork", "population": "population", "mobile": "mobile", "resting": "resting", "hunting": "hunting", "refuge": "refuge"}
    views = {}
    for name in checkpoints:
        directory = OUT / "reference" / name
        # Snapshot the raw file set before writing any .html wrappers.
        raw_files = sorted(path for path in directory.rglob("*") if path.is_file())
        raw_paths = set(raw_files)
        for raw in raw_files:
            data = raw.read_bytes()
            try:
                code = data.decode("utf-8")
            except UnicodeDecodeError:
                continue
            if "\0" in code:
                continue
            page = raw.with_name(raw.name + ".html")
            if page in raw_paths:
                raise ValueError(f"Source view would replace a raw export: {page}")
            relative = raw.relative_to(directory).as_posix()
            guide = guide_for[name] if guide_for[name] in guides else "index"
            guide_label = guides.get(guide, "Field guide")
            purpose = ("This review-only diff maps the previous checkpoint to this one. Use it to plan deliberate edits; it is not an automatic migration over your saved project."
                       if relative == "changes.patch" else
                       "This complete file is a worked answer. Open it beside the same file in your saved project; keep your own investigation and tests.")
            # Encode CR separately: HTML's input preprocessing would otherwise
            # normalize CRLF, changing the source text copied by the browser.
            escaped_code = html.escape(code, quote=False).replace("\r", "&#13;")
            language = {".rs": "rust", ".md": "markdown", ".toml": "toml",
                        ".lock": "toml", ".patch": "diff"}.get(raw.suffix, "text")
            page.write_text(f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark"><title>{esc(relative)} · {esc(name)} · Moss Fieldnotes</title>
<link rel="stylesheet" href="{relative_url(page, OUT / 'assets/source.css')}?v={assets_version}">
<script src="{relative_url(page, OUT / 'assets/source.js')}?v={assets_version}" defer></script></head>
<body data-source-view="{esc(name)}"><a class="skip-link" href="#source">Skip to source</a>
<header><a href="{relative_url(page, OUT / 'index.html')}">Moss Fieldnotes</a><span>Complete course reference</span></header>
<main id="source" tabindex="-1"><p class="eyebrow">{esc(name)} checkpoint</p><h1>{esc(relative)}</h1>
<p>{purpose}</p>
<nav class="source-actions" aria-label="Source file actions"><a href="{relative_url(page, OUT / (guide + '.html'))}">← {esc(guide_label)}</a>
<a href="{relative_url(page, raw)}" download>Download raw file</a>
<a href="{relative_url(page, OUT / 'downloads' / (name + '-reference.zip'))}" download>Download checkpoint ZIP</a>
<button type="button" data-copy-source hidden>Copy complete file</button></nav>
<p id="copy-status" role="status" aria-live="polite"></p>
<p class="source-help" id="source-help">The source below is selectable text. Long lines scroll horizontally. The raw download preserves the original file bytes.</p>
<pre tabindex="0" role="region" aria-label="Complete source file" aria-describedby="source-help"><code id="source-code" class="language-{language}" data-source-file="{relative_url(page, raw)}">{escaped_code}</code></pre>
</main><noscript><p>The complete source and downloads work without JavaScript. Copy with your browser’s text selection.</p></noscript></body></html>''', encoding="utf-8")
            views[raw.resolve()] = page.resolve()
    return views


def source_links(fragment, views):
    """Change only authored anchor hrefs after narration has verified its input."""
    class Links(HTMLParser):
        def __init__(self):
            super().__init__(convert_charrefs=False)
            self.changes = []
            self.offsets = [0]
            for match in re.finditer("\n", fragment):
                self.offsets.append(match.end())

        def handle_starttag(self, tag, attributes):
            attrs = dict(attributes)
            if tag != "a" or "href" not in attrs or "download" in attrs:
                return
            url = urlsplit(attrs["href"])
            if url.scheme or url.netloc or not url.path:
                return
            view = views.get((OUT / unquote(url.path)).resolve())
            if view is None:
                return
            href = urlunsplit(("", "", quote(view.relative_to(OUT.resolve()).as_posix(), safe="/"),
                               url.query, url.fragment))
            raw_tag = self.get_starttag_text()
            # Consume whole attribute tokens so a title containing `href="..."`
            # cannot be mistaken for the actual link attribute.
            attributes = re.compile(r'''(?P<name>[^\s"'<>/=]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+))?''')
            match = next((attribute for attribute in attributes.finditer(raw_tag)
                          if attribute.group("name").lower() == "href"), None)
            if match:
                line, column = self.getpos()
                start = self.offsets[line - 1] + column
                self.changes.append((start + match.start(), start + match.end(), f'href="{esc(href)}"'))

    parser = Links()
    parser.feed(fragment)
    for start, end, replacement in reversed(parser.changes):
        fragment = fragment[:start] + replacement + fragment[end:]
    return fragment


def nav(modules, current):
    groups = {}
    for n, module in enumerate(modules, 1):
        active = ' aria-current="page"' if module["id"] == current else ""
        groups.setdefault(module["part"], []).append(f'<li><a href="{module["id"]}.html"{active}><span class="number">{n:02}</span><span>{esc(module["title"])}<small>{module["minutes"]} min · {esc(module["concepts"][0])}</small></span><span class="progress-dot" data-progress-for="{module["id"]}" aria-hidden="true"></span></a></li>')
    active_part = next((m["part"] for m in modules if m["id"] == current), modules[0]["part"])
    return "".join(f'<li class="nav-group"><details{" open" if part == active_part else ""}><summary>{esc(part)}</summary><ol>{"".join(items)}</ol></details></li>' for part, items in groups.items())


class ChapterOutline(HTMLParser):
    """Read authored section headings without indexing hidden answer headings."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.skipped = 0
        self.heading = None
        self.items = []

    def handle_starttag(self, tag, attributes):
        if tag in {"details", "pre", "nav"}:
            self.skipped += 1
        if tag == "h2" and not self.skipped:
            identity = dict(attributes).get("id")
            if identity and identity != "chapter-overview":
                self.heading = (identity, [])

    def handle_data(self, text):
        if self.heading:
            self.heading[1].append(text)

    def handle_endtag(self, tag):
        if tag == "h2" and self.heading:
            identity, words = self.heading
            self.items.append((identity, "".join(words)))
            self.heading = None
        if tag in {"details", "pre", "nav"}:
            self.skipped -= 1


def chapter_contents(body):
    outline = ChapterOutline()
    outline.feed(body)
    if len(outline.items) < 2:
        return ""
    links = "".join(f'<li><a href="#{esc(identity)}">{esc(title)}</a></li>'
                    for identity, title in outline.items)
    return f'<details class="chapter-contents"><summary>In this chapter <span>{len(outline.items)} sections</span></summary><nav aria-label="In this chapter"><ol>{links}</ol></nav></details>'


def shell(title, body, modules, current="index", description="", lab=None, narration_version="", assets_version="", *, edition, narration_provider="macos"):
    module = next((m for m in modules if m["id"] == current), None)
    meta = f'<p class="eyebrow">{esc(module["part"])} · {module["minutes"]} minutes</p>' if module else '<p class="eyebrow">A living world. An inspectable program.</p>'
    stepper = ""
    if module:
        i = modules.index(module)
        previous = modules[i - 1] if i else None
        following = modules[i + 1] if i + 1 < len(modules) else None
        stepper = '<nav class="chapter-nav" aria-label="Lesson navigation">'
        stepper += f'<a href="{previous["id"]}.html">← {esc(previous["title"])}</a>' if previous else '<a href="index.html">← Field guide</a>'
        stepper += f'<a href="{following["id"]}.html">{esc(following["title"])} →</a>' if following else '<a href="roadmap.html">Explore the curriculum →</a>'
        stepper += '</nav>'
    controls = f'''<section class="practice-state" hidden aria-labelledby="practice-title"><h2 id="practice-title">Your place in this lesson</h2>
      <label for="lesson-state">Progress</label><select id="lesson-state"><option value="new">Not started</option><option value="reading">Reading</option><option value="practicing">Practicing</option><option value="reviewed">Reviewed</option></select>
      <p class="quiet">This is your own reading record. It does not certify tests or change Moss’s active assignment.</p>
      <label for="lesson-note">Field note</label><textarea id="lesson-note" rows="5" placeholder="A prediction, a question, or the evidence you want to remember…"></textarea><p id="save-status" role="status"></p>
      <button id="save-selection" type="button">Keep selected text</button><div id="clippings" aria-label="Saved passages"></div></section>''' if module or current in {"fieldwork", "returns", "population", "mobile", "resting", "hunting", "refuge"} else ''
    notes = f'<details class="reading-notes" hidden><summary>Reading notes</summary>{controls}</details>' if controls else ""
    model = f'<section class="experiment" data-lab="{esc(lab)}" aria-label="Interactive teaching experiment"></section>' if lab else ''
    narrator = {"macos": "Generated with an installed macOS voice.",
                "elevenlabs": "Generated with ElevenLabs."}.get(narration_provider, "Generated narration.")
    narration = f'<section class="narration-player" data-narration-source="narration/{esc(current)}/cues.json?v={esc(narration_version)}" aria-label="Generated narration"><p><strong>Listen with passage highlighting</strong></p><audio controls preload="metadata" aria-label="Lesson narration"><source src="narration/{esc(current)}/narration.m4a?v={esc(narration_version)}" type="audio/mp4"></audio><p class="quiet">{narrator} Prose and tables are read; code and answer disclosures are skipped. Playback uses the audio files included with this course.</p><p data-narration-status role="status"></p></section>' if narration_version else ''
    # Establish the destination before asking the reader to operate a model.
    contents = chapter_contents(body) if current not in {"index", "roadmap"} else ""
    overview = re.search(r'<section\b[^>]*class="lesson-overview"[^>]*>.*?</section>', body, re.S)
    if overview:
        at = overview.end()
        body = body[:at] + contents + narration + model + body[at:]
    else:
        end = body.find('</p>')
        at = end + 4 if end >= 0 else 0
        body = body[:at] + contents + narration + model + body[at:]
    cover = '<div class="atlas-cover"><img src="assets/identity/atlas-meadow-v1.png" width="1984" height="794" alt="" fetchpriority="high"></div>' if current == "index" else ""
    return f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="description" content="{esc(description)}"><meta name="color-scheme" content="light"><meta name="theme-color" content="#164f50">
<title>{esc(title)} · Moss Fieldnotes</title><link rel="icon" href="assets/identity/atlas-mark-v1.png" type="image/png"><link rel="stylesheet" href="assets/fieldnotes.css?v={assets_version}">
<script src="assets/records.js?v={assets_version}" defer></script><script src="assets/models.js?v={assets_version}" defer></script><script src="assets/terrarium.js?v={assets_version}" defer></script><script src="assets/evidence.js?v={assets_version}" defer></script><script src="assets/ecosystem.js?v={assets_version}" defer></script><script src="assets/population.js?v={assets_version}" defer></script><script src="assets/mobile.js?v={assets_version}" defer></script><script src="assets/fieldnotes.js?v={assets_version}" defer></script><script src="assets/narration.js?v={assets_version}" defer></script></head>
<body data-page="{esc(current)}"><a class="skip-link" href="#reading">Skip to reading</a>
<header class="masthead"><a class="brand" href="index.html"><img class="brand-mark" src="assets/identity/atlas-mark-v1.png" width="44" height="44" alt=""><span>Moss <span class="brand-edition">Fieldnotes</span></span></a>
<span class="edition-label">{esc(edition)}</span><a href="roadmap.html">The learning path <span aria-hidden="true">↗</span></a></header>{cover}
<div class="workspace"><aside class="sidebar"><details class="course-browser" open><summary>Browse chapters and guides</summary><nav aria-label="Learning path"><p class="eyebrow">The field guide</p><ol>{nav(modules,current)}</ol><a class="path-link" href="roadmap.html">See the full curriculum →</a><a class="path-link" href="setup.html">Set up the local workbench →</a><a class="path-link" href="fieldwork.html">Build one continuing meadow →</a><a class="path-link" href="population.html">Let a population develop →</a><a class="path-link" href="mobile.html">Follow a local opportunity →</a><a class="path-link" href="resting.html">Investigate a costly pause →</a><a class="path-link" href="hunting.html">Follow a contested capture →</a><a class="path-link" href="refuge.html">Give shelter a rule you can test →</a><a class="path-link" href="returns.html">Return to a different case →</a><a class="path-link" href="shelf.html">Visit the reading &amp; viewing shelf →</a></nav>
</details></aside>
<main id="reading" tabindex="-1"><div class="reading-toolbar" hidden aria-label="Reading controls"><button type="button" id="focus-mode" aria-pressed="false">Focus</button><label for="text-size">Text</label><select id="text-size"><option value="normal">Normal</option><option value="large">Large</option><option value="larger">Larger</option></select><button type="button" id="listen">Listen</button><button type="button" id="pause-reading" hidden>Pause</button><button type="button" id="stop-reading" hidden>Stop</button><button type="button" id="print-lesson">Print</button><span id="speech-status" role="status"></span><span id="reading-status" role="status"></span></div>
<article>{meta}<h1 data-narration="0">{esc(title)}</h1>{body}</article>{notes}{stepper}</main>
</div>
<footer class="site-footer"><span>Moss Fieldnotes · Learn the rule. Follow the consequence.</span><a href="about.html">Sources, evidence &amp; reading support</a></footer>
<noscript><p class="noscript-note">The full lessons, answers and navigation work without JavaScript. Interactive models, local notes and listening need JavaScript.</p></noscript>
</body></html>'''


def copy_checkpoints():
    """Publish immutable worked answers, never the mutable learner directory."""
    saved = {}
    sequence = ("shared-meadow", "population", "mobile", "resting", "hunting", "refuge")
    for name in sequence:
        source = ROOT / "checkpoints" / name
        if not source.is_dir():
            # A removed checkpoint must not leave old answer pages available.
            stale = OUT / "reference" / name
            if stale.exists():
                shutil.rmtree(stale)
            (OUT / "downloads" / f"{name}-reference.zip").unlink(missing_ok=True)
            continue
        files = {str(path.relative_to(source)): path.read_bytes()
                 for path in sorted(source.rglob("*")) if path.is_file()
                 and not {"target", ".git", ".DS_Store"}.intersection(path.relative_to(source).parts)}
        saved[name] = files
        destination = OUT / "reference" / name
        if destination.exists():
            shutil.rmtree(destination)
        for relative, data in files.items():
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        (OUT / "downloads").mkdir(exist_ok=True)
        with zipfile.ZipFile(OUT / "downloads" / f"{name}-reference.zip", "w", zipfile.ZIP_DEFLATED) as archive:
            for relative, data in files.items():
                info = zipfile.ZipInfo(f"{name}/{relative}")
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                archive.writestr(info, data)
    for previous, name in zip(sequence, sequence[1:]):
        if not {previous, name} <= saved.keys():
            continue
        before, after = saved[previous], saved[name]
        changes = []
        for relative in sorted(before.keys() | after.keys()):
            if relative == "CHECKPOINT.md":
                continue
            old, new = before.get(relative, b""), after.get(relative, b"")
            if old != new:
                changes.extend(difflib.unified_diff(old.decode().splitlines(keepends=True),
                                                   new.decode().splitlines(keepends=True),
                                                   fromfile=f"a/{relative}", tofile=f"b/{relative}"))
        (OUT / "reference" / name / "changes.patch").write_text("".join(changes))
    return saved


def build():
    course = json.loads((ROOT / "course.json").read_text())
    modules = course["modules"]
    guides = course.get("guides", {"index":"A small world, understood.", "roadmap":"A path through a living world", "setup":"Open the workbench", "about":"Read, listen, investigate", "shelf":"Good company for the next question", "terrarium":"The Rust world, in the page"})
    assets_version = hashlib.sha256(b"".join(path.name.encode() + path.read_bytes() for path in sorted((ROOT / "assets").iterdir()) if path.suffix in {".js", ".css"})).hexdigest()[:12]
    if len({m["id"] for m in modules}) != len(modules):
        raise ValueError("Duplicate module id")
    OUT.mkdir(exist_ok=True)
    shutil.copytree(ROOT / "assets", OUT / "assets", dirs_exist_ok=True)
    copy_preview(OUT)
    copy_fieldwork_preview(OUT)
    copy_fieldwork_preview(OUT, "population")
    copy_fieldwork_preview(OUT, "mobile")
    copy_fieldwork_preview(OUT, "resting")
    copy_fieldwork_preview(OUT, "hunting")
    # Never silently serve a binary from before a simulation/source change.
    runtime = ROOT / "work/runtime"
    target = OUT / "runtime"
    if target.exists():
        shutil.rmtree(target)
    if (runtime / "build.json").exists() and (runtime / "moss.wasm").exists():
        try:
            metadata = json.loads((runtime / "build.json").read_text())
            current = (isinstance(metadata, dict) and metadata.get("api") == 1
                       and metadata.get("sourceHash") == runtime_fingerprint())
        except (OSError, ValueError):
            current = False
        if current:
            shutil.copytree(runtime, target)
        else:
            print("Runtime is stale or its metadata is unreadable: rebuild with python3 learning/scripts/runtime.py build")
    (OUT / "examples").mkdir(exist_ok=True)
    (OUT / "reference").mkdir(exist_ok=True)
    shutil.copyfile(ROOT.parent / "docs/tutorial/today-v2.md", OUT / "reference/today-v2.md")
    checkpoints = copy_checkpoints()
    views = source_views(checkpoints, guides, assets_version)
    if (OUT / "narration").exists():
        shutil.rmtree(OUT / "narration")
    search = []
    for module in modules:
        fragment = (ROOT / "content" / f'{module["id"]}.html').read_text()
        parsed = Fragment()
        parsed.feed(fragment)
        narration_version = ""
        narration_provider = "macos"
        audio = ROOT / "work/narration" / module["id"]
        snapshot = read_narration(audio, module["title"], fragment)
        if snapshot:
            metadata, audio_bytes = snapshot
            narration_provider = metadata.get("provider", "macos")
            narration_version = metadata["sourceHash"][:12] + metadata["audioHash"][:12]
            destination = OUT / "narration" / module["id"]
            destination.mkdir(parents=True)
            # Publish the bytes that were verified, rather than reading mutable
            # source paths again after a concurrent narrator can replace them.
            (destination / "narration.m4a").write_bytes(audio_bytes)
            (destination / "cues.json").write_text(json.dumps(metadata, ensure_ascii=False) + "\n")
            _, fragment = passages(module["title"], fragment)
        for name, code in parsed.examples.items():
            (OUT / "examples" / f"{name}.rs").write_text(code.strip() + "\n")
        fragment = source_links(fragment, views)
        page = shell(module["title"], fragment, modules, module["id"], module["summary"], module.get("lab"), narration_version, assets_version, edition=course["edition"], narration_provider=narration_provider)
        (OUT / f'{module["id"]}.html').write_text(page)
        search.append({"id": module["id"], "title": module["title"], "summary": module["summary"], "text": " ".join(" ".join(parsed.text).split())})
    for name, title in guides.items():
        body = (ROOT / "content" / f"{name}.html").read_text()
        body = body.replace('<!--AVAILABLE_COUNT-->', str(len(modules)))
        if name == "index":
            cards = ''.join(f'<a class="lesson-card" href="{m["id"]}.html"><span class="eyebrow">Fieldnote {i:02} · {m["minutes"]} min</span><h2>{esc(m["title"])}</h2><p>{esc(m["summary"])}</p><span class="card-link">Open fieldnote <span aria-hidden="true">↗</span></span></a>' for i, m in enumerate(modules, 1))
            body = body.replace('<!--LESSON_CARDS-->', f'<div class="lesson-cards">{cards}</div>')
        parsed = Fragment()
        parsed.feed(body)
        for example, code in parsed.examples.items():
            (OUT / "examples" / f"{example}.rs").write_text(code.strip() + "\n")
        narration_version = ""
        narration_provider = "macos"
        snapshot = read_narration(ROOT / "work/narration" / name, title, body)
        if snapshot:
            metadata, audio_bytes = snapshot
            narration_provider = metadata.get("provider", "macos")
            narration_version = metadata["sourceHash"][:12] + metadata["audioHash"][:12]
            destination = OUT / "narration" / name
            destination.mkdir(parents=True)
            (destination / "narration.m4a").write_bytes(audio_bytes)
            (destination / "cues.json").write_text(json.dumps(metadata, ensure_ascii=False) + "\n")
            _, body = passages(title, body)
        body = source_links(body, views)
        (OUT / f"{name}.html").write_text(shell(title, body, modules, name, course["description"], narration_version=narration_version, assets_version=assets_version, edition=course["edition"], narration_provider=narration_provider))
        search.append({"id": name, "title": title, "summary": "Course guide", "text": " ".join(" ".join(parsed.text).split())})
    (OUT / "assets" / "search.json").write_text(json.dumps(search, ensure_ascii=False))
    print(f"Built {len(modules)} lessons, {len(guides)} guide pages and {len(views)} source views in {OUT}")


if __name__ == "__main__":
    build()
