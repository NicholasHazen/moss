#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
scripts/with-toolchain.sh cargo test --workspace --locked
scripts/with-toolchain.sh cargo clippy --workspace --all-targets --locked -- -D warnings
scripts/with-toolchain.sh cargo clippy -p moss-web --target wasm32-unknown-unknown --locked -- -D warnings
node --check crates/moss-web/shell.js
