"""AbstractAvatar: deterministic procedural frames through the bounded native ABI."""
from __future__ import annotations

import ctypes as C
import json
import math
from pathlib import Path
import sys
import threading
from typing import Any

__version__ = "0.2.0"
__all__ = ["Renderer", "catalog", "retired", "resolve_style", "validate_descriptor", "__version__"]
_STATES = ("ready", "thinking", "working", "listening", "speaking", "waiting", "error")
_LIBRARIES: dict[str, Any] = {}
_LIBRARY_LOCK = threading.RLock()
_NAMES = {"darwin": "libabstractavatar.dylib", "win32": "abstractavatar.dll"}


def _library(library_path: str | Path | None = None):
    path = Path(library_path).expanduser().resolve() if library_path is not None else Path(__file__).parent / "_native" / _NAMES.get(sys.platform, "libabstractavatar.so")
    if not path.is_file():
        raise RuntimeError("AbstractAvatar native library is unavailable. Install a built AbstractAvatar wheel or supply library_path explicitly.")
    key = str(path)
    with _LIBRARY_LOCK:
        if key in _LIBRARIES:
            return _LIBRARIES[key]
        lib = C.CDLL(key)
        signatures = {
            "avatar_abi_version": ([], C.c_uint32),
            "avatar_style_count": ([], C.c_uint32),
            "avatar_style_field_ptr": ([C.c_uint32] * 2, C.c_void_p),
            "avatar_style_field_len": ([C.c_uint32] * 2, C.c_size_t),
            "avatar_default_style": ([], C.c_uint32),
            "avatar_retired_count": ([], C.c_uint32),
            "avatar_retired_field_ptr": ([C.c_uint32] * 2, C.c_void_p),
            "avatar_retired_field_len": ([C.c_uint32] * 2, C.c_size_t),
            "avatar_create": ([C.c_uint32] * 2, C.c_uint32),
            "avatar_set_shade": ([C.c_uint32] * 4, C.c_int32),
            "avatar_set_ground": ([C.c_uint32] * 2, C.c_int32),
            "avatar_set_gaze": ([C.c_uint32, C.c_float, C.c_float], C.c_int32),
            "avatar_set_bands": ([C.c_uint32] + [C.c_float] * 8, C.c_int32),
            "avatar_clear_bands": ([C.c_uint32], C.c_int32),
            "avatar_render_copy": ([C.c_uint32] * 3 + [C.c_float] * 2 + [C.c_uint32, C.c_void_p, C.c_size_t], C.c_int32),
            "avatar_destroy": ([C.c_uint32], C.c_int32),
        }
        try:
            for name, (args, result) in signatures.items():
                fn = getattr(lib, name)
                fn.argtypes, fn.restype = args, result
        except AttributeError as exc:
            raise RuntimeError("AbstractAvatar native library is missing a required ABI function") from exc
        if lib.avatar_abi_version() != 1:
            raise RuntimeError("Unsupported AbstractAvatar native ABI")
        _LIBRARIES[key] = lib
        return lib


def catalog(library_path: str | Path | None = None) -> list[dict[str, Any]]:
    """Return runtime catalog records. Persist ``id`` rather than numeric ``code``."""
    lib = _library(library_path)
    count = int(lib.avatar_style_count())
    if not 1 <= count <= 256:
        raise RuntimeError("AbstractAvatar catalog exceeds the supported bounds")
    out: list[dict[str, Any]] = []
    seen: set[str] = set()
    for code in range(count):
        record: dict[str, Any] = {"code": code}
        for field, name in enumerate(("id", "name", "description", "watch_code", "family", "gaze")):
            record[name] = _text(lib, "avatar_style_field", code, field)
        if not record["id"] or record["id"] in seen:
            raise RuntimeError("Invalid or duplicated AbstractAvatar style identifier")
        if record["gaze"] not in ("eyes", "none"):
            raise RuntimeError("Invalid AbstractAvatar gaze component")
        record["reference"] = record["watch_code"]
        seen.add(record["id"])
        out.append(record)
    return out


def _text(lib, prefix: str, index: int, field: int) -> str:
    length = int(getattr(lib, prefix + "_len")(index, field))
    pointer = getattr(lib, prefix + "_ptr")(index, field)
    if length > 512 or (length and not pointer):
        raise RuntimeError("Invalid AbstractAvatar catalog field")
    return C.string_at(pointer, length).decode("utf-8") if length else ""


def retired(library_path: str | Path | None = None) -> list[dict[str, Any]]:
    """Return the designs removed from the catalog. Their ids stay reserved and resolve to
    ``replacement``, or to the default design when ``replacement`` is ``None``."""
    lib = _library(library_path)
    count = int(lib.avatar_retired_count())
    if count > 256:
        raise RuntimeError("AbstractAvatar retired list exceeds the supported bounds")
    out = []
    for index in range(count):
        record = {name: _text(lib, "avatar_retired_field", index, field) for field, name in enumerate(("id", "replacement", "retired_in", "date", "reason"))}
        record["replacement"] = record["replacement"] or None
        out.append(record)
    return out


def resolve_style(style: str, library_path: str | Path | None = None) -> str:
    """Return the active design id to draw for a stored id: the id itself when it is active,
    a retired id's replacement, otherwise the default design (``halo``). Never fails for a
    string, so a stored identity that names a removed avatar still renders."""
    if not isinstance(style, str):
        raise ValueError("Avatar style must be a string")
    lib = _library(library_path)
    active = [record["id"] for record in catalog(library_path)]
    if style in active:
        return style
    replacement = next((r["replacement"] for r in retired(library_path) if r["id"] == style), None)
    return replacement if replacement in active else active[int(lib.avatar_default_style())]


def _number(value: Any, lo: float, hi: float, name: str) -> float:
    try:
        valid = not isinstance(value, bool) and isinstance(value, (int, float)) and math.isfinite(value) and lo <= value <= hi
    except (OverflowError, TypeError, ValueError):
        valid = False
    if not valid:
        raise ValueError(f"{name} must be finite and in {lo:g}..{hi:g}")
    return float(value)


def _vector(value: Any, count: int, lo: float, hi: float, name: str) -> tuple[float, ...]:
    if not isinstance(value, (list, tuple)) or len(value) != count:
        raise ValueError(f"{name} must contain exactly {count} values")
    return tuple(_number(v, lo, hi, name) for v in value)


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicated avatar field: {key}")
        result[key] = value
    return result


def validate_descriptor(value: dict[str, Any] | str, *, styles: set[str] | None = None) -> dict[str, Any]:
    """Validate an inert schema-version-1 descriptor; no file or URL is evaluated."""
    if isinstance(value, str):
        if len(value.encode("utf-8")) > 4096:
            raise ValueError("Avatar descriptor exceeds 4096 bytes")
        try:
            value = json.loads(value, object_pairs_hook=_unique_object)
        except (ValueError, RecursionError) as exc:
            raise ValueError(f"Invalid avatar descriptor: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError("Avatar descriptor must be an object")
    required = {"schema_version", "style", "state", "time", "level"}
    if set(value) - required - {"gaze", "bands"} or required - set(value):
        raise ValueError("Avatar descriptor has unknown or missing fields")
    if isinstance(value["schema_version"], bool) or not isinstance(value["schema_version"], (int, float)) or value["schema_version"] != 1:
        raise ValueError("Unsupported avatar schema_version")
    if not isinstance(value["style"], str) or not value["style"] or len(value["style"]) > 64:
        raise ValueError("Avatar style must be a nonempty identifier of at most 64 characters")
    if styles is not None and value["style"] not in styles:
        raise ValueError("Unknown avatar style")
    if value["state"] not in _STATES:
        raise ValueError("Unknown avatar state")
    out = dict(value)
    out["time"] = _number(value["time"], 0, 86400, "time")
    out["level"] = _number(value["level"], 0, 1, "level")
    if "gaze" in out:
        out["gaze"] = _vector(out["gaze"], 2, -1, 1, "gaze")
    if "bands" in out:
        out["bands"] = _vector(out["bands"], 8, 0, 1, "bands")
    return out


def _color(value: Any, count: int, name: str) -> tuple[int, ...]:
    if not isinstance(value, (list, tuple)) or len(value) != count or any(type(v) is not int or not 0 <= v <= 255 for v in value):
        raise ValueError(f"{name} must contain {count} integer channels in 0..255")
    return tuple(value)


class Renderer:
    """Own one reusable bounded native RGBA8 buffer; close it when the host closes."""
    def __init__(self, width: int = 128, height: int = 128, library_path: str | Path | None = None):
        if type(width) is not int or type(height) is not int or not 1 <= width <= 512 or not 1 <= height <= 512:
            raise ValueError("Avatar dimensions must be integers in 1..512")
        self.width, self.height = width, height
        self._lock = threading.RLock()
        self._library = _library(library_path)
        self._library_path = library_path
        self._styles = {r["id"]: r["code"] for r in catalog(library_path)}
        self._handle = int(self._library.avatar_create(width, height))
        if not self._handle:
            raise RuntimeError("AbstractAvatar renderer capacity exceeded (at most 16 live renderers)")
        self._buffer = C.create_string_buffer(width * height * 4)
        try:
            self.set_palette()
        except BaseException:
            self.close()
            raise

    def _check(self, status: int):
        if status:
            raise RuntimeError(f"AbstractAvatar operation refused (code {status})")

    def _open(self):
        if not self._handle:
            raise RuntimeError("AbstractAvatar renderer is closed")

    def set_palette(self, *, warm=(211, 158, 57), eye=(225, 70, 95), dim=(80, 90, 105), ground=(0, 0, 0, 0)):
        """Set three host theme color anchors and an RGBA background."""
        anchors = [_color(v, 3, n) for v, n in [(warm, "warm"), (eye, "eye"), (dim, "dim")]]
        ground = _color(ground, 4, "ground")
        with self._lock:
            self._open()
            self._check(self._library.avatar_set_ground(self._handle, int.from_bytes(bytes(ground), "big")))
            for material, mid in enumerate(anchors):
                for shade in range(256):
                    t = shade / 255
                    left, right, amount = (ground[:3], mid, t / .65) if t < .65 else (mid, (255, 255, 255), (t - .65) / .35)
                    rgb = tuple(int(a + (b - a) * amount + .5) for a, b in zip(left, right))
                    self._check(self._library.avatar_set_shade(self._handle, material, shade, int.from_bytes(bytes((*rgb, 255)), "big")))

    def render(self, style="halo", state="ready", level=0.0, time=0.0, gaze=(0, 0), transparent=True, bands=None) -> bytes:
        """Return owned, tightly packed, straight-alpha RGBA8 bytes.

        A retired or unknown ``style`` renders its replacement or the default design
        (see :func:`resolve_style`)."""
        if isinstance(style, str) and style not in self._styles:
            style = resolve_style(style, self._library_path)
        desc = {"schema_version": 1, "style": style, "state": state, "time": time, "level": level, "gaze": gaze}
        if bands is not None:
            desc["bands"] = bands
        frame = validate_descriptor(desc, styles=set(self._styles))
        if type(transparent) is not bool:
            raise ValueError("transparent must be a boolean")
        with self._lock:
            self._open()
            self._check(self._library.avatar_set_gaze(self._handle, *frame["gaze"]))
            if "bands" in frame:
                self._check(self._library.avatar_set_bands(self._handle, *frame["bands"]))
            else:
                self._check(self._library.avatar_clear_bands(self._handle))
            self._check(self._library.avatar_render_copy(self._handle, self._styles[style], _STATES.index(state), frame["level"], frame["time"], int(transparent), self._buffer, len(self._buffer)))
            return self._buffer.raw

    def render_descriptor(self, descriptor: dict[str, Any] | str, *, transparent: bool = True) -> bytes:
        frame = validate_descriptor(descriptor)
        return self.render(style=frame["style"], state=frame["state"], time=frame["time"], level=frame["level"], gaze=frame.get("gaze", (0, 0)), bands=frame.get("bands"), transparent=transparent)

    def close(self):
        with self._lock:
            if self._handle:
                self._check(self._library.avatar_destroy(self._handle))
                self._handle = 0

    def __enter__(self):
        self._open()
        return self

    def __exit__(self, *_):
        self.close()

    def __del__(self):
        if getattr(self, "_handle", 0):
            try:
                self.close()
            except Exception:
                pass
