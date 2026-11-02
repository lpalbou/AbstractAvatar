import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import assert from "node:assert/strict";

const [wasm, manifestPath] = process.argv.slice(2);
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const module = await WebAssembly.compile(readFileSync(wasm));
assert.equal(WebAssembly.Module.imports(module).length, 0, "Wasm must have no ambient imports");
const { exports: core } = await WebAssembly.instantiate(module, {});
assert.equal(core.avatar_abi_version(), 1);
let maximum = 0, differing = 0, channels = 0;
for (const size of new Set(manifest.cases.map(c => c.size))) {
  const handle = core.avatar_create(size, size);
  assert.ok(handle);
  try {
    manifest.palette.forEach((ramp, material) => ramp.forEach((rgba, shade) =>
      assert.equal(core.avatar_set_shade(handle, material, shade, rgba), 0)));
    assert.equal(core.avatar_set_bands(handle, .1,.2,.3,.4,.5,.6,.7,.8), 0);
    for (const item of manifest.cases.filter(c => c.size === size)) {
      assert.equal(core.avatar_set_gaze(handle, item.gaze[0], item.gaze[1]), 0);
      assert.equal(core.avatar_render(handle, item.style, item.pose, .6, 1.1, 1), 0);
      const length = core.avatar_pixels_len(handle);
      assert.equal(length, size * size * 4);
      const actual = new Uint8Array(core.memory.buffer, core.avatar_pixels(handle), length);
      const expected = readFileSync(join(dirname(manifestPath), item.file));
      for (let i = 0; i < length; i++) {
        const delta = Math.abs(actual[i] - expected[i]);
        if (delta) differing++;
        maximum = Math.max(maximum, delta);
        assert.ok(delta <= 2, `Native/Wasm drift ${delta} in ${item.file} channel ${i}`);
      }
      channels += length;
    }
  } finally {
    assert.equal(core.avatar_destroy(handle), 0);
  }
}
console.log(JSON.stringify({ frames: manifest.cases.length, channels, differing, maximum }, null, 2));
