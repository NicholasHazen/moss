#!/usr/bin/env python3
"""Test and build the existing moss-sim host; never edit or activate a live rule."""
import argparse
import hashlib
import json
import shutil
from pathlib import Path

from labs import ROOT, TOOLCHAIN, WORK, run

def runtime_fingerprint():
    repository = ROOT.parent
    inputs = [repository / "Cargo.toml", repository / "rust-toolchain.toml"]
    for directory in (ROOT / "runtime", repository / "crates/moss-sim"):
        inputs += [directory / "Cargo.toml"] + sorted(directory.glob("src/**/*.rs"))
    inputs += [ROOT / "runtime/Cargo.lock"]
    digest = hashlib.sha256()
    for path in sorted(inputs):
        digest.update(str(path.relative_to(repository)).encode() + b"\0" + path.read_bytes())
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["check", "build"])
    args = parser.parse_args()
    directory = ROOT / "runtime"
    if args.action == "check":
        run(["cargo", f"+{TOOLCHAIN}", "test", "--offline", "--locked"], directory)
    else:
        source_hash = runtime_fingerprint()
        run(["cargo", f"+{TOOLCHAIN}", "build", "--offline", "--locked", "--release", "--target", "wasm32-unknown-unknown"], directory)
        if runtime_fingerprint() != source_hash:
            raise SystemExit("Runtime inputs changed during compilation. Rebuild before publishing this binary.")
        output = WORK / "runtime"
        output.mkdir(parents=True, exist_ok=True)
        source = WORK / "target/wasm32-unknown-unknown/release/moss_fieldnotes_runtime.wasm"
        shutil.copyfile(source, output / "moss.wasm")
        (output / "build.json").write_text(json.dumps({"api": 1, "sourceHash": source_hash, "rust": TOOLCHAIN, "bevyEcs": "0.18.1"}) + "\n")
        print(f"Built {source.stat().st_size:,} bytes. Run build.py to include this runtime in the course.")


if __name__ == "__main__":
    main()
