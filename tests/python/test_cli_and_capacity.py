"""Consumer-facing packaging, output and resource-boundary checks."""
import json
from pathlib import Path
import struct
import subprocess
import sys
import zlib

import pytest

from abstractavatar import Renderer, catalog


def test_handle_capacity_refuses_then_recovers_without_leaking():
    renderers = []
    try:
        for _ in range(16):
            renderers.append(Renderer(width=8, height=8))
        with pytest.raises(RuntimeError, match="capacity"):
            Renderer(width=8, height=8)
        renderers.pop().close()
        with Renderer(width=8, height=8) as replacement:
            assert len(replacement.render()) == 8 * 8 * 4
    finally:
        for renderer in renderers:
            renderer.close()
    with Renderer(width=8, height=8) as recovered:
        assert len(recovered.render()) == 256


def test_cli_enumerates_native_catalog_and_writes_valid_png(tmp_path):
    result = subprocess.run([sys.executable, "-m", "abstractavatar", "catalog"], capture_output=True, text=True, check=True)
    assert [row["id"] for row in json.loads(result.stdout)] == [row["id"] for row in catalog()]
    image = tmp_path / "avatar.png"
    subprocess.run([
        sys.executable, "-m", "abstractavatar", "render", "--style", "halo", "--state", "speaking",
        "--level", "0.6", "--time", "1.2", "--size", "32", "--output", str(image),
    ], capture_output=True, text=True, check=True)
    data = image.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    offset, compressed = 8, bytearray()
    while offset < len(data):
        length = struct.unpack(">I", data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + length]
        crc = struct.unpack(">I", data[offset + 8 + length:offset + 12 + length])[0]
        assert zlib.crc32(kind + payload) == crc
        if kind == b"IHDR":
            assert struct.unpack(">IIBBBBB", payload) == (32, 32, 8, 6, 0, 0, 0)
        if kind == b"IDAT":
            compressed.extend(payload)
        offset += 12 + length
    rows = zlib.decompress(compressed)
    assert len(rows) == 32 * (1 + 32 * 4)
    assert any(rows)


def test_cli_invalid_request_writes_no_partial_image(tmp_path):
    output = tmp_path / "refused.png"
    result = subprocess.run([
        sys.executable, "-m", "abstractavatar", "render", "--size", "513", "--output", str(output),
    ], capture_output=True, text=True)
    assert result.returncode == 2
    assert not output.exists()
