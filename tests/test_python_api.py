"""Public descriptor, lifetime and native-buffer contracts."""
from concurrent.futures import ThreadPoolExecutor
import json
import math
from pathlib import Path
import unittest

from abstractavatar import Renderer, catalog, validate_descriptor


class DescriptorTests(unittest.TestCase):
    def setUp(self):
        self.good = {"schema_version": 1, "style": "halo", "state": "ready", "time": 1.2, "level": .5}

    def test_strict_json_rejects_unknown_duplicate_nonfinite_and_null_fields(self):
        self.assertEqual(validate_descriptor(json.dumps(self.good))["time"], 1.2)
        for field, value in [("script", "anything"), ("schema_version", True), ("level", math.nan), ("time", -1), ("time", 10**500), ("state", "future"), ("gaze", None), ("bands", None), ("bands", [0] * 7), ("gaze", [0, 2])]:
            with self.subTest(field=field, value=str(value)[:50]), self.assertRaises(ValueError):
                validate_descriptor({**self.good, field: value})
        with self.assertRaises(ValueError):
            validate_descriptor('{"schema_version":1,"schema_version":1}')
        with self.assertRaises(ValueError):
            validate_descriptor(" " * 4097)


class NativeTests(unittest.TestCase):
    def test_catalog_has_every_source_identity_and_stable_first_six(self):
        designs = catalog()
        self.assertEqual([d["id"] for d in designs[:6]], ["halo", "hover", "vortex", "eclipse", "eye", "rover"])
        self.assertEqual(len(designs), 44)
        self.assertEqual(len({d["id"] for d in designs}), 44)
        self.assertEqual(sum(d["id"].startswith("watch-") and not d["id"].startswith("watch-shape-") for d in designs), 34)
        self.assertTrue({"watch-shape-kitty", "watch-shape-face", "watch-shape-wave", "iphone-weave"} <= {d["id"] for d in designs})

    def test_gaze_moves_only_eyes_designs_and_catalog_json_agrees(self):
        designs = catalog()
        shipped = json.loads((Path(__file__).resolve().parents[1] / "catalog.json").read_text())["designs"]
        self.assertEqual({d["id"]: d["gaze"] for d in designs}, {d["id"]: d["gaze"] for d in shipped})
        self.assertTrue({"hover", "watch-hover", "rover", "watch-rover"} <= {d["id"] for d in designs if d["gaze"] == "eyes"})
        self.assertTrue({"halo", "watch-halo", "vortex", "eclipse"} <= {d["id"] for d in designs if d["gaze"] == "none"})
        with Renderer(64, 64) as renderer:
            for design in designs:
                rest = renderer.render(design["id"], "ready", .6, 1.1)
                looked = renderer.render(design["id"], "ready", .6, 1.1, gaze=(1, -1))
                (self.assertNotEqual if design["gaze"] == "eyes" else self.assertEqual)(rest, looked, design["id"])
                self.assertEqual(renderer.render(design["id"], "speaking", .6, 1.1), renderer.render(design["id"], "speaking", .6, 1.1, gaze=(1, -1)), design["id"])

    def test_every_design_renders_all_states_and_errors_are_static(self):
        with Renderer(48, 48) as renderer:
            for design in catalog():
                seen = []
                for state in ["ready", "thinking", "working", "listening", "speaking", "waiting", "error"]:
                    frame = renderer.render(design["id"], state, .6, 1.1)
                    self.assertEqual(len(frame), 48 * 48 * 4)
                    self.assertGreater(sum(frame[3::4]), 1000, design["id"] + state)
                    seen.append(frame)
                self.assertGreater(len(set(seen)), 1, design["id"])
                self.assertEqual(renderer.render(design["id"], "error", .6, 0), renderer.render(design["id"], "error", .6, 10))

    def test_owned_frames_survive_later_render_and_closed_handles_refuse(self):
        renderer = Renderer(32, 32)
        first = renderer.render("halo", "ready", 0, 1)
        before = bytes(first)
        renderer.render("hover", "working", 1, 20)
        self.assertEqual(first, before)
        renderer.close()
        renderer.close()
        with self.assertRaises(RuntimeError):
            renderer.render()

    def test_threads_render_without_shared_buffer_corruption(self):
        with Renderer(32, 32) as renderer:
            expected = {style: renderer.render(style, "speaking", .6, 1.1) for style in ["halo", "hover", "watch-nebula"]}
            jobs = ["halo", "hover", "watch-nebula"] * 8
            with ThreadPoolExecutor(max_workers=4) as pool:
                got = list(pool.map(lambda style: renderer.render(style, "speaking", .6, 1.1), jobs))
            self.assertEqual(got, [expected[style] for style in jobs])

    def test_bounds_palette_and_bands_fail_before_render(self):
        for width, height in [(0, 1), (1, 513), (True, 32)]:
            with self.assertRaises(ValueError):
                Renderer(width, height)
        with Renderer(32, 32) as renderer:
            with self.assertRaises(ValueError):
                renderer.set_palette(warm=(0, 999, 0))
            with self.assertRaises(ValueError):
                renderer.render(style="absent")
            with self.assertRaises(ValueError):
                renderer.render(transparent="false")
            base = renderer.render("watch-rings", "speaking", .6, 1, bands=[0] * 8)
            changed = renderer.render("watch-rings", "speaking", .6, 1, bands=[1] * 8)
            self.assertNotEqual(base, changed)

    def test_source_defined_voice_effects_follow_real_audio_only(self):
        affected = ["watch-lotus", "watch-black-hole", "watch-accretion", "watch-cruiser"]
        with Renderer(64, 64) as renderer:
            for style in affected:
                for state in ["speaking", "listening"]:
                    self.assertNotEqual(renderer.render(style, state, 0, 2.3), renderer.render(style, state, 1, 2.3), style + state)
                for state in ["ready", "thinking", "working", "waiting", "error"]:
                    self.assertEqual(renderer.render(style, state, 0, 2.3), renderer.render(style, state, 1, 2.3), style + state)

    def test_every_design_answers_speaking_level_including_time_zero(self):
        with Renderer(64,64) as renderer:
            for design in catalog():
                for time in [0,2.3]:
                    self.assertNotEqual(renderer.render(design["id"],"speaking",0,time),renderer.render(design["id"],"speaking",1,time),design["id"])

    def test_ported_activity_has_distinct_work_thought_wait_and_error_cues(self):
        with Renderer(64,64) as renderer:
            for design in catalog()[6:]:
                frames=[renderer.render(design["id"],state,.6,2.3) for state in ["ready","thinking","working","waiting","error"]]
                self.assertEqual(len(set(frames)),5,design["id"])


if __name__ == "__main__":
    unittest.main()
