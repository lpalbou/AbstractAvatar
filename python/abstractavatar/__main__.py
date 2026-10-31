"""Catalog inspection and deterministic PNG rendering, without a GUI dependency."""
import argparse
import json
from pathlib import Path
import struct
import zlib
from . import Renderer, catalog


def png(width, height, pixels):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    stride = width * 4
    rows = b"".join(b"\0" + pixels[y * stride:(y + 1) * stride] for y in range(height))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")


def main():
    parser = argparse.ArgumentParser(description="Inspect and render AbstractAvatar procedural designs")
    parser.add_argument("--library", help="Explicit trusted native library path")
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("catalog", help="Print the native style catalog as JSON")
    render = commands.add_parser("render", help="Render an RGBA PNG frame")
    render.add_argument("--style", default="halo")
    render.add_argument("--state", default="ready")
    render.add_argument("--time", type=float, default=0.0)
    render.add_argument("--level", type=float, default=0.0)
    render.add_argument("--size", type=int, default=128)
    render.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "catalog":
            print(json.dumps(catalog(args.library), indent=2))
        else:
            with Renderer(args.size, args.size, args.library) as renderer:
                data = renderer.render(style=args.style, state=args.state, time=args.time, level=args.level)
            args.output.write_bytes(png(args.size, args.size, data))
    except (ValueError, RuntimeError, OSError) as exc:
        parser.exit(2, f"abstractavatar: {exc}\n")


if __name__ == "__main__":
    main()
