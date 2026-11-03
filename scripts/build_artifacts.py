#!/usr/bin/env python3
"""Build native and/or Wasm artifacts and record exact generator provenance."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def source_hash():
    digest = hashlib.sha256()
    for source in sorted([ROOT / "Cargo.toml", *ROOT.joinpath("src").rglob("*.rs")]):
        digest.update(source.relative_to(ROOT).as_posix().encode())
        digest.update(b"\0")
        digest.update(source.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-only", action="store_true")
    parser.add_argument("--wasm-only", action="store_true")
    parser.add_argument("--uic-dir", type=Path, help="Explicit destination containing avatar_core.wasm and provenance.json")
    parser.add_argument("--check", action="store_true", help="Check artifact/source hashes without building")
    args = parser.parse_args()
    if args.native_only and args.wasm_only:
        parser.error("choose at most one of --native-only and --wasm-only")
    output = ROOT / "artifacts"
    if args.check:
        destination = args.uic_dir or output
        metadata = json.loads((destination / "provenance.json").read_text())
        if metadata.get("source") != "abstractavatar" or metadata.get("source_sha256") != source_hash():
            parser.exit(1, "AbstractAvatar artifact source is stale\n")
        actual = hashlib.sha256((destination / "avatar_core.wasm").read_bytes()).hexdigest()
        if metadata.get("wasm_sha256") != actual:
            parser.exit(1, "AbstractAvatar Wasm checksum differs\n")
        print("AbstractAvatar artifact and source hashes match")
        return
    output.mkdir(exist_ok=True)
    target_directory=ROOT / "target"
    if not args.wasm_only:
        host=next(line.split(": ",1)[1] for line in subprocess.check_output(["rustc","-vV"],text=True).splitlines() if line.startswith("host: "))
        subprocess.run(["cargo", "build", "--manifest-path", str(ROOT / "Cargo.toml"), "--release", "--features", "ffi", "--locked", "--target-dir", str(target_directory), "--target", host], check=True)
        name = "libabstractavatar.dylib" if sys.platform == "darwin" else "abstractavatar.dll" if sys.platform == "win32" else "libabstractavatar.so"
        source = target_directory / host / "release" / name
        shutil.copy2(source, output / name)
        package = ROOT / "python" / "abstractavatar" / "_native"
        package.mkdir(exist_ok=True)
        shutil.copy2(source, package / name)
        # Keep the conventional developer path stable for explicit native consumers.
        conventional=target_directory / "release" / name
        conventional.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(source,conventional)
        print(f"Native library: {source}")
    if not args.native_only:
        subprocess.run(["cargo", "build", "--manifest-path", str(ROOT / "Cargo.toml"), "--release", "--features", "ffi", "--target", "wasm32-unknown-unknown", "--target-dir",str(target_directory), "--locked"], check=True)
        binary = ROOT / "target" / "wasm32-unknown-unknown" / "release" / "abstractavatar.wasm"
        data = binary.read_bytes()
        (output / "avatar_core.wasm").write_bytes(data)
        metadata = {"abi": 1, "source": "abstractavatar", "crate_version": "0.1.1", "source_sha256": source_hash(), "wasm_sha256": hashlib.sha256(data).hexdigest(), "build": "python3 abstractavatar/scripts/build_artifacts.py", "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip()}
        (output / "provenance.json").write_text(json.dumps(metadata, indent=2) + "\n")
        if args.uic_dir:
            args.uic_dir.mkdir(parents=True, exist_ok=True)
            for name in ("avatar_core.wasm", "provenance.json"):
                shutil.copy2(output / name, args.uic_dir / name)
        print(f"Wasm artifact: {output / 'avatar_core.wasm'}")


if __name__ == "__main__":
    main()
