#!/bin/sh
# Keep macOS builds independent of the system-wide Xcode selection.
# Respect an explicitly selected developer directory; Linux needs no override.
set -eu
# Trunk 0.21 parses NO_COLOR as a boolean, while many terminals set it to 1.
if [ "${NO_COLOR:-}" = 1 ]; then
    export NO_COLOR=true
fi
if [ "$(uname -s)" = Darwin ] && [ -z "${DEVELOPER_DIR:-}" ] && [ -d /Library/Developer/CommandLineTools ]; then
    export DEVELOPER_DIR=/Library/Developer/CommandLineTools
fi
exec "$@"
