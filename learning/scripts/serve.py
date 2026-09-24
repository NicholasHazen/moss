#!/usr/bin/env python3
"""Serve the current static edition locally. Rebuild explicitly after edits."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


class CourseHandler(SimpleHTTPRequestHandler):
    # Some platform MIME registries interpret .rs as an XML service list, while
    # unknown source suffixes download. Worked module links should be readable.
    extensions_map = dict(SimpleHTTPRequestHandler.extensions_map, **{
        suffix: "text/plain; charset=utf-8"
        for suffix in (".rs", ".md", ".patch", ".toml", ".lock")
    })


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--port", type=int, default=8091)
args = parser.parse_args()
site = Path(__file__).resolve().parents[1] / "_site"
if not (site / "index.html").exists():
    parser.error("Build first: python3 learning/scripts/build.py")
with ThreadingHTTPServer(("127.0.0.1", args.port), partial(CourseHandler, directory=str(site))) as server:
    print(f"Moss Fieldnotes: http://127.0.0.1:{args.port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
