const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const file = path.join(__dirname, "../work/runtime/moss.wasm");
const available = fs.existsSync(file);

async function runtime() {
  const module = await WebAssembly.compile(fs.readFileSync(file));
  assert.deepEqual(WebAssembly.Module.imports(module), [], "host must supply no replacement rule");
  return (await WebAssembly.instantiate(module, {})).exports;
}

test("actual WASM schedule spends both reserves and reset advances run", { skip: !available && "Build runtime.py build first" }, async () => {
  const api = await runtime();
  assert.equal(api.moss_fieldnotes_run(), 1n);
  for (let i = 0; i < 3; i++) api.moss_fieldnotes_step();
  assert.equal(api.moss_fieldnotes_tick(), 3n);
  assert.equal(api.moss_fieldnotes_value(1, 2), 57);
  assert.equal(api.moss_fieldnotes_value(2, 2), 57);
  assert.equal(api.moss_fieldnotes_value(3, 3), 80);
  api.moss_fieldnotes_reset();
  assert.equal(api.moss_fieldnotes_run(), 2n);
  assert.equal(api.moss_fieldnotes_tick(), 0n);
  assert.equal(api.moss_fieldnotes_value(1, 2), 60);
});

test("WASM reads preserve time and distinguish absent from depleted", { skip: !available && "Build runtime.py build first" }, async () => {
  const api = await runtime();
  for (let i = 0; i < 70; i++) api.moss_fieldnotes_step();
  for (let i = 0; i < 20; i++) {
    assert.equal(api.moss_fieldnotes_value(1, 2), 0);
    assert.equal(api.moss_fieldnotes_value(3, 2), -1);
    assert.equal(api.moss_fieldnotes_value(99, 2), -1);
    assert.equal(api.moss_fieldnotes_value(1, 99), -1);
    assert.equal(api.moss_fieldnotes_tick(), 70n);
  }
  assert.equal(api.moss_fieldnotes_value(1, 0), 10);
  assert.equal(api.moss_fieldnotes_value(3, 3), 80);
});
