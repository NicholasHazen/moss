#!/usr/bin/env python3
"""Check generated local links, HTML structure, labels and exact extracted examples.

This is a structural check, not a browser or screen-reader audit.
"""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit
import json
import sys

from build import OUT, ROOT, build
from labs import examples

VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}


class Page(HTMLParser):
    def __init__(self, path):
        super().__init__(convert_charrefs=True)
        self.path = path
        self.ids = set()
        self.links = []
        self.labels = set()
        self.controls = []
        self.stack = []
        self.h1 = 0
        self.main = 0
        self.errors = []
        self.source_view = False
        self.sources = []
        self.current_source = None

    def handle_starttag(self, tag, attributes):
        a = dict(attributes)
        if tag not in VOID:
            self.stack.append(tag)
        if tag == "html" and a.get("lang") != "en":
            self.errors.append("Missing document language")
        if tag == "h1": self.h1 += 1
        if tag == "main": self.main += 1
        if tag == "body" and "data-source-view" in a:
            self.source_view = True
        if tag == "code" and "data-source-file" in a:
            self.current_source = {"href": a["data-source-file"], "text": []}
            self.sources.append(self.current_source)
        if "id" in a:
            if a["id"] in self.ids:
                self.errors.append(f"Duplicate id: {a['id']}")
            self.ids.add(a["id"])
        if tag == "label" and "for" in a:
            self.labels.add(a["for"])
        if tag in {"input", "select", "textarea"} and a.get("type") != "file":
            self.controls.append(a)
        for key in ("href", "src"):
            if key in a: self.links.append(a[key])

    def handle_endtag(self, tag):
        if tag == "code": self.current_source = None
        if tag in VOID: return
        if not self.stack or self.stack[-1] != tag:
            self.errors.append(f"Unexpected closing tag: {tag}; stack={self.stack[-3:]}")
        else:
            self.stack.pop()

    def handle_data(self, data):
        if self.current_source is not None:
            self.current_source["text"].append(data)


def check():
    build()
    pages = {}
    errors = []
    for path in sorted(OUT.rglob("*.html")):
        page = Page(path)
        page.feed(path.read_text())
        pages[path.resolve()] = page
        if page.h1 != 1 or page.main != 1:
            page.errors.append("Expected one h1 and one main")
        if page.stack:
            page.errors.append(f"Unclosed tags: {page.stack}")
        for control in page.controls:
            if control.get("id") not in page.labels and not control.get("aria-label"):
                page.errors.append(f"Unlabelled control: {control}")
        if page.source_view and len(page.sources) != 1:
            page.errors.append("Expected one exact source block")
        for source in page.sources:
            raw = (path.parent / unquote(source["href"])).resolve()
            if not raw.is_relative_to((OUT / "reference").resolve()) or not raw.is_file():
                page.errors.append(f"Invalid raw source: {source['href']}")
            elif "".join(source["text"]) != raw.read_bytes().decode("utf-8"):
                page.errors.append(f"Displayed source differs from raw file: {source['href']}")
        errors.extend(f"{path.relative_to(OUT)}: {error}" for error in page.errors)
    for path, page in pages.items():
        for href in page.links:
            url = urlsplit(href)
            if url.scheme or url.netloc: continue
            target = (path.parent / unquote(url.path)).resolve() if url.path else path
            if not target.is_relative_to(OUT.resolve()):
                errors.append(f"{path.relative_to(OUT.resolve())}: link escapes export: {href}")
            elif not target.is_file():
                errors.append(f"{path.relative_to(OUT.resolve())}: missing target: {href}")
            elif url.fragment and target in pages and unquote(url.fragment) not in pages[target].ids:
                errors.append(f"{path.relative_to(OUT.resolve())}: missing anchor: {href}")
    for (kind, name), code in examples().items():
        if kind in {"data-runnable", "data-practice-starter", "data-practice-answer"} and (OUT / f"examples/{name}.rs").read_text() != code:
            errors.append(f"Extracted reference differs: {name}")
    returns = ROOT / "practice/returns.rs"
    if returns.exists() and (OUT / "examples/mixed-returns.rs").read_text() != returns.read_text():
        errors.append("Mixed return answer differs from its native practice file")
    index = json.loads((OUT / "assets/search.json").read_text())
    course = json.loads((ROOT / "course.json").read_text())
    if {item["id"] for item in index} != {m["id"] for m in course["modules"]} | set(course.get("guides", {})):
        errors.append("Search index does not cover every lesson and guide")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    source_count = sum(page.source_view for page in pages.values())
    print(f"Checked {len(pages) - source_count} course pages and {source_count} source views: balanced HTML, local links/anchors, landmarks, static labels, exact references/source text and search coverage.")
    print("Dynamic controls, layout, speech and external links require separate verification.")
    return 0


if __name__ == "__main__":
    sys.exit(check())
