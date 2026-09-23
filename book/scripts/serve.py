#!/usr/bin/env python3
"""Preview the fixed book export at its production URL prefix, on loopback only."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import tomllib
from urllib.parse import urlsplit

BOOK = Path(__file__).resolve().parents[1]
SITE = BOOK / "_site"
PREFIX = tomllib.loads((BOOK / "book.toml").read_text())["output"]["html"]["site-url"]


class BookHandler(SimpleHTTPRequestHandler):
    def do_GET(self) -> None:
        if self.redirect_root():
            return
        super().do_GET()

    def do_HEAD(self) -> None:
        if self.redirect_root():
            return
        super().do_HEAD()

    def redirect_root(self) -> bool:
        if urlsplit(self.path).path in {"/", PREFIX.rstrip("/")}:
            self.send_response(302)
            self.send_header("Location", PREFIX)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return True
        return False

    def send_head(self):
        if not urlsplit(self.path).path.startswith(PREFIX):
            self.send_error(404)
            return None
        return super().send_head()

    def translate_path(self, path: str) -> str:
        return super().translate_path("/" + path[len(PREFIX):])

    def send_error(self, code: int, message=None, explain=None) -> None:
        page = SITE / "404.html"
        if code != 404 or not page.is_file():
            super().send_error(code, message, explain)
            return
        body = page.read_bytes()
        self.send_response(404)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)


if __name__ == "__main__":
    print(f"Moss book: http://127.0.0.1:8090{PREFIX} (fixed build)", flush=True)
    server = ThreadingHTTPServer(("127.0.0.1", 8090), partial(BookHandler, directory=str(SITE)))
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
