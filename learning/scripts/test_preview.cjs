const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const directory = path.join(__dirname, "../work/previews/evidence");
const available = fs.existsSync(path.join(directory, "build.json"));

async function host() {
  const metadata = JSON.parse(fs.readFileSync(path.join(directory, "build.json"), "utf8"));
  const module = await WebAssembly.compile(fs.readFileSync(path.join(directory, metadata.wasm)));
  assert.deepEqual(WebAssembly.Module.imports(module), [], "JavaScript supplies no projection rule");
  return (await WebAssembly.instantiate(module, {})).exports;
}
const optional = {skip: !available && "Build preview.py evidence first"};
test("compiled exercise distinguishes current projection from the observer's trace", optional, async () => {
  const api = await host();
  for (const [ordering, display, observed] of [[0, 37n, 37n], [1, 99n, 99n], [2, 37n, 99n]]) {
    api.fieldnotes_evidence_reset(3, 8, 1, ordering);
    assert.equal(api.fieldnotes_evidence_value(0), 99n);
    assert.equal(api.fieldnotes_evidence_value(2), 0n);
    api.fieldnotes_evidence_step();
    assert.equal(api.fieldnotes_evidence_value(0), display);
    assert.equal(api.fieldnotes_evidence_value(1), observed);
    assert.equal(api.fieldnotes_evidence_value(2), 1n);
  }
});
test("compiled selection and arithmetic stay in Rust across browser inputs", optional, async () => {
  const api = await host();
  for (const [used, capacity, selected, expected] of [[3,8,0,-1n],[3,8,2,-1n],[3,0,1,-1n],[0,8,1,0n],[9,8,1,100n],[4294967295,4294967295,1,100n],[7,11,1,63n]]) {
    api.fieldnotes_evidence_reset(used, capacity, selected, 0);
    api.fieldnotes_evidence_step();
    assert.equal(api.fieldnotes_evidence_value(0), expected);
    assert.equal(api.fieldnotes_evidence_value(1), expected);
  }
});
