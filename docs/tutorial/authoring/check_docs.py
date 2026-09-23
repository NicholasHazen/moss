#!/usr/bin/env python3
"""Check Moss's local Markdown navigation using only the Python standard library.

Scope: root *.md, docs/**/*.md and prompts/**/*.md; single-line inline links
(including images), explicit/collapsed reference links, reference definitions,
ATX headings and quoted HTML <a id/name> aliases. Destinations may use angle
brackets or balanced parentheses, percent escapes and an optional quoted title.
Fences use backticks or tildes with at most three leading spaces. Fenced text,
HTML comments and inline backtick literals do not supply links or anchors.

Heading slugs follow the punctuation/duplicate conventions used in Moss; this
is not a complete CommonMark parser or a renderer compatibility test. Setext
headings, shortcut references, indented code blocks, blockquote fences, multiline
links and arbitrary HTML navigation are outside its scope. Remote URLs are not
fetched; fragments on non-Markdown files and executable examples are not checked.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from html import unescape
from pathlib import Path
import re
import unicodedata
from urllib.parse import unquote, urlsplit


@dataclass
class Report:
    files: int = 0
    links: int = 0
    errors: list[str] = field(default_factory=list)


def _escaped(text: str, index: int) -> bool:
    start = index
    while start and text[start - 1] == "\\":
        start -= 1
    return (index - start) % 2 == 1


def _mask_code(text: str) -> str:
    """Preserve positions while hiding matched inline backtick spans."""
    chars = list(text)
    runs = list(re.finditer(r"`+", text))
    index = 0
    while index < len(runs):
        opening = runs[index]
        if _escaped(text, opening.start()):
            index += 1
            continue
        closing = next(
            (j for j in range(index + 1, len(runs))
             if runs[j].group() == opening.group()),
            None,
        )
        if closing is None:
            index += 1
            continue
        end = runs[closing].end()
        chars[opening.start():end] = " " * (end - opening.start())
        index = closing + 1
    return "".join(chars)


def _visible_lines(path: Path, errors: list[str]) -> list[tuple[int, str]]:
    result = []
    fence = None
    opening_line = 0
    in_comment = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if fence is not None:
            if (marker and marker[1][0] == fence[0]
                    and len(marker[1]) >= len(fence) and not marker[2].strip()):
                fence = None
            continue
        # Comments can span lines, but fenced examples are already excluded.
        visible = ""
        remaining = line
        while remaining:
            if in_comment:
                _, found, remaining = remaining.partition("-->")
                if not found:
                    break
                in_comment = False
            else:
                start = _mask_code(remaining).find("<!--")
                if start == -1:
                    visible += remaining
                    break
                visible += remaining[:start]
                remaining = remaining[start + 4:]
                in_comment = True
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", visible)
        if marker and (marker[1][0] != "`" or "`" not in marker[2]):
            fence, opening_line = marker[1], number
        else:
            result.append((number, visible))
    if fence is not None:
        errors.append(f"{path}:{opening_line}: unclosed {fence[0]} fence")
    return result


def _anchors(lines: list[tuple[int, str]]) -> set[str]:
    anchors = set()
    heading_ids = set()
    for _, line in lines:
        for tag in re.findall(r"<a\b[^>]*>", _mask_code(line), re.IGNORECASE):
            anchors.update(value for _, value in re.findall(
                r"\s(?:id|name)\s*=\s*(['\"])(.*?)\1", tag, re.IGNORECASE
            ))
        heading = re.match(r"^ {0,3}#{1,6}[ \t]+(.+?)\s*$", line)
        if not heading:
            continue
        title = re.sub(r"[ \t]+#+[ \t]*$", "", heading[1])
        title = re.sub(r"!?\[([^]]+)\]\([^)]*\)", r"\1", title)
        title = re.sub(r"<[^>]*>", "", title).replace("`", "")
        title = unescape(title).lower()
        slug = "".join(
            char for char in title
            if char in " -_" or unicodedata.category(char)[0] in "LNM"
        ).replace(" ", "-")
        candidate, suffix = slug, 0
        while candidate in heading_ids:
            suffix += 1
            candidate = f"{slug}-{suffix}"
        heading_ids.add(candidate)
        anchors.add(candidate)
    return anchors


def _destination(text: str, start: int, inline: bool) -> tuple[str, int] | None:
    """Read a destination and optional quoted title; return its ending offset."""
    index = start
    while index < len(text) and text[index].isspace():
        index += 1
    begin = index
    if index < len(text) and text[index] == "<":
        begin = index = index + 1
        while index < len(text) and (text[index] != ">" or _escaped(text, index)):
            index += 1
        if index == len(text):
            return None
        value = text[begin:index]
        index += 1
    else:
        depth = 0
        while index < len(text):
            char = text[index]
            if not _escaped(text, index):
                if char.isspace():
                    break
                if char == "(":
                    depth += 1
                elif char == ")":
                    if depth == 0:
                        break
                    depth -= 1
            index += 1
        if depth:
            return None
        value = text[begin:index]
    while index < len(text) and text[index].isspace():
        index += 1
    if index < len(text) and text[index] in "\"'":
        quote = text[index]
        index += 1
        while index < len(text) and (text[index] != quote or _escaped(text, index)):
            index += 1
        if index == len(text):
            return None
        index += 1
        while index < len(text) and text[index].isspace():
            index += 1
    if inline:
        if index == len(text) or text[index] != ")":
            return None
        index += 1
    elif index != len(text):
        return None
    return re.sub(r"\\([^\w\s])", r"\1", value), index


def _label(value: str) -> str:
    return " ".join(value.split()).casefold()


def _links(lines: list[tuple[int, str]], path: Path, errors: list[str]):
    definitions = {}
    body = []
    for number, original in lines:
        line = _mask_code(original)
        definition = re.match(r"^ {0,3}\[([^]]+)\]:[ \t]*(.*)$", line)
        if definition:
            parsed = _destination(definition[2], 0, inline=False)
            if parsed is None:
                errors.append(f"{path}:{number}: malformed reference destination")
                continue
            key = _label(definition[1])
            if key not in definitions:
                definitions[key] = parsed[0]
            yield number, parsed[0]
        else:
            body.append((number, line))
    for number, line in body:
        index = 0
        while index < len(line):
            if line[index] != "[" or _escaped(line, index):
                index += 1
                continue
            begin = index
            depth = 1
            index += 1
            while index < len(line) and depth:
                if not _escaped(line, index):
                    if line[index] == "[":
                        depth += 1
                    elif line[index] == "]":
                        depth -= 1
                index += 1
            if depth or index == len(line):
                continue
            if line[index] == "(":
                parsed = _destination(line, index + 1, inline=True)
                if parsed is None:
                    errors.append(f"{path}:{number}: malformed inline destination")
                    continue
                value, index = parsed
                yield number, value
            elif line[index] == "[":
                closing = line.find("]", index + 1)
                if closing == -1:
                    continue
                key = _label(line[index + 1:closing] or line[begin + 1:index - 1])
                if key not in definitions:
                    errors.append(f"{path}:{number}: undefined reference [{key}]")
                index = closing + 1
                # Defined destinations were checked once above.


def check(root: Path) -> Report:
    """Return navigation diagnostics without fetching URLs or changing files."""
    root = root.resolve()
    files = sorted(set(root.glob("*.md")) | set((root / "docs").rglob("*.md"))
                   | set((root / "prompts").rglob("*.md")))
    report = Report(files=len(files))
    if not files:
        report.errors.append(f"{root}: no Markdown files found")
        return report
    lines = {}
    anchors = {}
    for path in files:
        try:
            lines[path] = _visible_lines(path, report.errors)
            anchors[path] = _anchors(lines[path])
        except (OSError, UnicodeError) as error:
            report.errors.append(f"{path}: cannot read Markdown: {error}")
    for path, content in lines.items():
        for number, destination in _links(content, path, report.errors):
            try:
                url = urlsplit(destination)
            except ValueError as error:
                report.errors.append(f"{path}:{number}: invalid destination: {error}")
                continue
            if url.scheme or url.netloc:
                continue
            report.links += 1
            target = (path.parent / unquote(url.path)).resolve() if url.path else path
            fragment = unquote(url.fragment)
            if not target.exists():
                report.errors.append(f"{path}:{number}: missing target {destination}")
            elif fragment and target.suffix.lower() == ".md":
                if target not in anchors:
                    try:
                        anchors[target] = _anchors(_visible_lines(target, report.errors))
                    except (OSError, UnicodeError) as error:
                        report.errors.append(f"{path}:{number}: cannot read {target}: {error}")
                        continue
                if fragment not in anchors[target]:
                    report.errors.append(f"{path}:{number}: missing anchor {destination}")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    report = check(parser.parse_args().root)
    print(f"Checked {report.links} local destinations across {report.files} Markdown files.")
    for error in report.errors:
        print(error)
    return int(bool(report.errors))


if __name__ == "__main__":
    raise SystemExit(main())
