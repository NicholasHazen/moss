#!/usr/bin/env python3
"""Package explicit repository citations as inert source snapshots for a book build.

Markdown source links still work in the IDE. The exported book needs its own
copy of each cited file, rather than broken paths outside the publishing root.
Only directly cited files under the allowed project areas are packaged; their
contents are escaped, not executed or recursively published. Editorial and
collaboration material are excluded. Run after mdBook, before distribution.
"""
from __future__ import annotations

import hashlib
from html import escape
from html.parser import HTMLParser
import os
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parents[2]
SITE = ROOT / "book/_site"
sys.path.insert(0, str(ROOT / "docs/tutorial/authoring"))
from check_docs import _anchors, _visible_lines  # noqa: E402

ALLOWED = (
    ROOT / "crates/moss-sim", ROOT / "crates/moss-web",
    ROOT / "docs/tutorial", ROOT / "docs/development",
    ROOT / "docs/design", ROOT / "docs/research",
)


def validate_figure(content: bytes) -> None:
    """Accept our deliberately small, passive SVG vocabulary, not arbitrary SVG."""
    text = content.decode("utf-8")
    if re.search(r"<!DOCTYPE|<!ENTITY|<\?(?!xml\s)", text, re.IGNORECASE):
        raise ValueError("SVG declarations and processing instructions are not allowed")
    tags = {"svg", "g", "defs", "title", "desc", "rect", "circle", "ellipse",
            "path", "line", "polyline", "polygon", "text", "tspan", "marker", "pattern", "style"}
    attributes = set("id class role aria-labelledby width height viewBox x y x1 y1 x2 y2 "
                     "cx cy r rx ry d points fill stroke stroke-width stroke-dasharray "
                     "stroke-linecap stroke-linejoin opacity fill-opacity stroke-opacity "
                     "font-family font-size font-weight font-style text-anchor letter-spacing "
                     "transform marker-end marker-start markerWidth markerHeight refX refY "
                     "orient patternUnits style".split())
    root = ElementTree.fromstring(text)
    if root.tag != "{http://www.w3.org/2000/svg}svg":
        raise ValueError("Expected an SVG root in the SVG namespace")
    for element in root.iter():
        if not element.tag.startswith("{http://www.w3.org/2000/svg}") or element.tag.split("}")[1] not in tags:
            raise ValueError(f"Unsupported SVG element: {element.tag}")
        if not set(element.attrib) <= attributes:
            raise ValueError("Unsupported SVG attributes: " + ", ".join(set(element.attrib) - attributes))
        values = list(element.attrib.values())
        if element.tag.endswith("}style"):
            values.append(element.text or "")
        for value in values:
            # mdBook's official favicon uses this passive color-scheme rule.
            styling = re.sub(r"@media\s*\(\s*prefers-color-scheme:\s*(dark|light)\s*\)", "", value)
            if re.search(r"[@\\\\]|expression\s*\(|behavior\s*:|-moz-binding", styling, re.IGNORECASE):
                raise ValueError("Active or escaped SVG styling is not allowed")
            for target in re.findall(r"url\s*\((.*?)\)", value, re.IGNORECASE):
                if not re.fullmatch(r"#[A-Za-z_][\w.-]*", target.strip()):
                    raise ValueError("Only local SVG paint/marker references are allowed")


def snapshot(target: Path) -> Path:
    target = target.resolve()
    relative = target.relative_to(ROOT)
    if not any(target.is_relative_to(area) for area in ALLOWED):
        raise ValueError(f"Publication source is not allowed: {relative}")
    if "authoring" in relative.parts:
        raise ValueError(f"Keep authoring machinery outside the reading edition: {relative}")
    output = SITE / "source" / relative
    if target.suffix == ".svg":
        content = target.read_bytes()
        validate_figure(content)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(content)
        return output
    if target.suffix not in {".md", ".rs", ".toml"}:
        raise ValueError(f"Unsupported citation snapshot: {relative}")
    output = output.with_suffix(output.suffix + ".html")
    output.parent.mkdir(parents=True, exist_ok=True)
    content = target.read_text()
    digest = hashlib.sha256(content.encode()).hexdigest()
    by_line: dict[int, str] = {}
    if target.suffix == ".md":
        visible = _visible_lines(target, [])
        seen: set[str] = set()
        for index, (number, _) in enumerate(visible):
            current = _anchors(visible[:index + 1])
            by_line[number] = "".join(f'<span id="{escape(anchor, quote=True)}"></span>' for anchor in sorted(current - seen))
            seen = current
    lines = "\n".join(f'{by_line.get(n, "")}<span id="L{n}">{escape(line)}</span>' for n, line in enumerate(content.splitlines(), 1))
    back = Path(os.path.relpath(SITE / "index.html", output.parent)).as_posix()
    output.write_text(f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>{escape(str(relative))} · Moss source snapshot</title>
<style>body{{max-width:1000px;margin:40px auto;padding:0 24px;color:#25392b;background:#fcfbf6;font:16px/1.65 system-ui}}a{{color:#285943}}pre{{overflow:auto;padding:20px;border:1px solid #cbd5c6;background:#f0f3e9;font:13px/1.7 ui-monospace,monospace}}code{{overflow-wrap:anywhere}}pre span:target{{background:#dee9c0}}h1{{font:32px Georgia,serif}}.meta{{color:#55654c;font-size:14px}}</style></head>
<body><a href="{back}">← Textbook home</a><h1>Source snapshot</h1>
<p><code>{escape(str(relative))}</code></p>
<p>This file was captured from the local working tree when this book was built.
It is reference material; the live checkout may have changed. Markdown is shown
as source, so it does not publish the pages it links to.</p>
<p class="meta">SHA-256: <code>{digest}</code></p><pre>{lines}</pre></body></html>
''')
    return output


class CitationRewriter(HTMLParser):
    """Rewrite actual link attributes; leave code, comments and script strings alone."""
    def __init__(self, source: str, rewrite) -> None:
        super().__init__(convert_charrefs=False)
        self.source = source
        self.rewrite = rewrite
        self.edits: list[tuple[int, int, str]] = []
        self.offsets = [0]
        for line in source.splitlines(keepends=True):
            self.offsets.append(self.offsets[-1] + len(line))

    def handle_starttag(self, tag: str, attrs) -> None:
        self.visit_tag(tag, attrs, False)

    def handle_startendtag(self, tag: str, attrs) -> None:
        self.visit_tag(tag, attrs, True)

    def visit_tag(self, tag: str, attrs, closed: bool) -> None:
        updated = [(key, self.rewrite(value) if key in {"href", "src"} and value else value)
                   for key, value in attrs]
        if updated == attrs:
            return
        rendered = "".join(" " + key + (f'="{escape(value, quote=True)}"' if value is not None else "")
                           for key, value in updated)
        line, column = self.getpos()
        start = self.offsets[line - 1] + column
        original = self.get_starttag_text()
        self.edits.append((start, start + len(original), f'<{tag}{rendered}{" /" if closed else ""}>'))

    def result(self) -> str:
        self.feed(self.source)
        result = self.source
        for start, end, replacement in reversed(self.edits):
            result = result[:start] + replacement + result[end:]
        return result


def main() -> None:
    packaged: dict[Path, Path] = {}
    for page in sorted(SITE.rglob("*.html")):
        if page.is_relative_to(SITE / "source"):
            continue
        def replace(raw: str) -> str:
            # mdBook's error page has a site-root <base>; a bare heading hash
            # otherwise resolves to the home page instead of the error page.
            if page == SITE / "404.html" and raw.startswith("#"):
                return "404.html" + raw
            url = urlsplit(raw)
            if url.scheme or url.netloc or not url.path:
                return raw
            if url.path.startswith("/"):
                return raw
            target = (page.parent / unquote(url.path)).resolve()
            if target.is_relative_to(SITE):
                return raw
            if target.suffix == ".html" and not target.exists():
                target = target.with_suffix(".md")
            if not target.is_relative_to(ROOT) or not target.is_file():
                raise ValueError(f"{page.relative_to(SITE)}: unresolved repository citation {raw}")
            if target not in packaged:
                packaged[target] = snapshot(target)
            destination = Path(os.path.relpath(packaged[target], page.parent)).as_posix()
            if url.fragment:
                destination += "#" + url.fragment
            return destination
        source = page.read_text()
        page.write_text(CitationRewriter(source, replace).result())
    for figure in SITE.rglob("*.svg"):
        validate_figure(figure.read_bytes())
    print(f"Packaged {len(packaged)} directly cited project files as source snapshots.")


if __name__ == "__main__":
    main()
