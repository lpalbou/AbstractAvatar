#!/usr/bin/env python3
"""Compare bounded native/Wasm RGBA output with identical inputs and host ramps."""
from pathlib import Path
import argparse
import json
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))
from abstractavatar import Renderer, catalog  # noqa: E402


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--wasm", type=Path, default=ROOT / "artifacts/avatar_core.wasm")
    args = parser.parse_args()
    library = args.native.resolve()
    designs = catalog(library)
    # Distinct ramps exercise color mixing and partial material alpha.
    palette = [[int.from_bytes(bytes((shade, 20, 40, 255)), "big") for shade in range(256)],
               [int.from_bytes(bytes((20, shade, 80, 210)), "big") for shade in range(256)],
               [int.from_bytes(bytes((50, 40, shade, 180)), "big") for shade in range(256)]]
    cases = []
    with tempfile.TemporaryDirectory(prefix="abstractavatar-parity-") as folder:
        output = Path(folder)
        for size in [32, 64, 128]:
            with Renderer(size, size, library_path=library) as renderer:
                for material, shades in enumerate(palette):
                    for shade, rgba in enumerate(shades):
                        renderer._check(renderer._library.avatar_set_shade(renderer._handle, material, shade, rgba))
                for design in designs:
                    if size != 64 and design["id"] not in {"watch-halo", "watch-wave", "watch-hover", "watch-rings", "watch-eye"}:
                        continue
                    for pose, state in enumerate(["ready", "thinking", "working", "listening", "speaking", "waiting", "error"]):
                        raw = renderer.render(design["id"], state, .6, 1.1, bands=[.1,.2,.3,.4,.5,.6,.7,.8])
                        filename = f"{size}-{design['code']}-{pose}.rgba"
                        (output / filename).write_bytes(raw)
                        cases.append(dict(size=size, style=design["code"], pose=pose, file=filename, gaze=[0, 0]))
                    if size == 64 and design["gaze"] == "eyes":
                        # Eye-following path: Ready with an off-centre gaze.
                        raw = renderer.render(design["id"], "ready", .6, 1.1, gaze=(.75, -.5), bands=[.1,.2,.3,.4,.5,.6,.7,.8])
                        filename = f"{size}-{design['code']}-gaze.rgba"
                        (output / filename).write_bytes(raw)
                        cases.append(dict(size=size, style=design["code"], pose=0, file=filename, gaze=[.75, -.5]))
        manifest = output / "manifest.json"
        manifest.write_text(json.dumps(dict(palette=palette, cases=cases)))
        subprocess.run(["node", str(ROOT / "scripts/check_portability.mjs"), str(args.wasm.resolve()), str(manifest)], check=True)


if __name__ == "__main__":
    main()
