#!/bin/sh
# A pinned, local publishing tool; independent of the Moss Cargo workspace.
set -eu
book_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
book_tool="$book_root/../.tools/mdbook-0.5.3/bin/mdbook"
if [ ! -x "$book_tool" ]; then
    echo "Install mdBook 0.5.3 using the command in book/README.md." >&2
    exit 1
fi
if [ "$("$book_tool" --version)" != "mdbook v0.5.3" ]; then
    echo "The textbook requires mdBook 0.5.3; check the local tool installation." >&2
    exit 1
fi
build_book() {
    "$book_tool" build "$book_root"
    python3 "$book_root/scripts/finish_site.py"
    python3 "$book_root/scripts/package_starter.py" --copy-to-site
    python3 "$book_root/scripts/package_edition.py" --copy-to-site
}
case "${1:-build}" in
    build)
        build_book
        ;;
    serve)
        build_book
        exec python3 "$book_root/scripts/serve.py"
        ;;
    *) echo "Usage: sh book/scripts/book.sh [build|serve]" >&2; exit 2 ;;
esac
