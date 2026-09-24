// Run the complete enhancement against the remaining reading controls. These
// fixtures check storage/DOM wiring; real browser layout is checked separately.
const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const records = require("../assets/records.js");
const KEY = "moss-fieldnotes-v1";

class Element {
  constructor() {
    this.events = new Map(); this.children = []; this.value = "";
    this.textContent = ""; this.hidden = true; this.open = false;
    this.dataset = {}; this.classes = new Set();
    this.classList = {add: name => this.classes.add(name), remove: name => this.classes.delete(name),
      toggle: name => this.classes.has(name) ? (this.classes.delete(name), false) : (this.classes.add(name), true)};
  }
  addEventListener(type, listener) { this.events.set(type, listener); }
  dispatch(type) { return this.events.get(type)({target: this}); }
  append(...nodes) { this.children.push(...nodes); }
  replaceChildren(...nodes) { this.children = nodes; }
  setAttribute() {}
  setSelectionRange() {}
  focus() {}
}

function fixture({notes = true, saved = null, locks = true} = {}) {
  let bytes = saved;
  const writes = [], events = new Map();
  const controls = Object.fromEntries(["text-size", "focus-mode", "print-lesson", "listen",
    "pause-reading", "stop-reading", "speech-status", "reading-status"]
    .map(id => [id, new Element()]));
  const noteIds = ["lesson-state", "lesson-note", "clippings", "save-status", "save-selection"];
  for (const id of noteIds) controls[id] = notes ? new Element() : null;
  const toolbar = new Element(), notebook = new Element(), practice = new Element();
  const groups = {".reading-toolbar": [toolbar], ".reading-notes": notes ? [notebook] : [],
    ".practice-state": notes ? [practice] : []};
  const storage = {
    getItem(key) { assert.equal(key, KEY); return bytes; },
    setItem(key, value) { assert.equal(key, KEY); bytes = value; writes.push(value); },
  };
  const document = {
    body: Object.assign(new Element(), {dataset: {page: notes ? "01-observe" : "about"}}),
    documentElement: {style: {setProperty() {}}},
    getElementById(id) {
      if (id === "") return null; // The initial empty URL fragment.
      assert.ok(Object.hasOwn(controls, id), `Unexpected or removed control: ${id}`);
      return controls[id];
    },
    querySelector() { return null; },
    querySelectorAll(selector) { return selector.split(",").flatMap(part => groups[part] || []); },
    createElement() { return new Element(); },
  };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../assets/fieldnotes.js"), "utf8"), {
    document, localStorage: storage, location: {hash: ""},
    navigator: locks ? {locks: {request: async (key, action) => { assert.equal(key, KEY); return action(); }}} : {},
    window: {MossRecords: records, addEventListener: (type, listener) => events.set(type, listener)},
  });
  return {controls, toolbar, notebook, practice, writes, bytes: () => bytes,
    replaceBytes(value) { bytes = value; },
    storageEvent() { events.get("storage")({key: KEY, storageArea: storage}); },
    unloadBlocked() {
      let prevented = false;
      events.get("beforeunload")({preventDefault() { prevented = true; }});
      return prevented;
    }};
}
const settled = () => new Promise(resolve => setImmediate(resolve));

test("notes and progress still load/save without search or utility-panel controls", async () => {
  const saved = records.empty();
  saved.settings.textSize = "large";
  saved.lessons["01-observe"] = {state: "reading", note: "An earlier prediction.", clips: ["A kept passage."]};
  saved.lessons["02-ownership"] = {state: "reviewed", note: "Keep this other lesson.", clips: []};
  const f = fixture({saved: JSON.stringify(saved)});
  assert.equal(f.toolbar.hidden, false);
  assert.equal(f.notebook.hidden, false);
  assert.equal(f.notebook.open, false, "Reading notes starts collapsed");
  assert.equal(f.practice.hidden, false);
  assert.equal(f.controls["lesson-note"].value, "An earlier prediction.");
  assert.equal(f.controls.clippings.children.length, 1);
  assert.equal(f.controls["text-size"].value, "large");
  assert.equal(f.writes.length, 0, "Loading must not migrate or rewrite saved data");
  f.controls["lesson-note"].value = "A revised prediction.";
  f.controls["lesson-note"].dispatch("input");
  f.controls["lesson-state"].value = "practicing";
  f.controls["lesson-state"].dispatch("change");
  await settled();
  const expected = records.validate(saved);
  expected.lessons["01-observe"].note = "A revised prediction.";
  expected.lessons["01-observe"].state = "practicing";
  assert.deepEqual(JSON.parse(f.bytes()), expected);
  assert.equal(f.controls["save-status"].textContent, "Saved in this browser.");
  assert.equal(f.controls["reading-status"].textContent, "");
  assert.equal(f.unloadBlocked(), false);
});

test("malformed saved bytes stay untouched with or without note controls", async () => {
  for (const notes of [true, false]) {
    const original = '{"version":1,"lessons":broken';
    const f = fixture({notes, saved: original});
    assert.match(f.controls["reading-status"].textContent, /left unchanged/);
    if (notes) {
      f.controls["lesson-note"].value = "Do not lose this new thought.";
      f.controls["lesson-note"].dispatch("input");
    } else {
      f.controls["text-size"].value = "larger";
      f.controls["text-size"].dispatch("change");
    }
    await settled();
    assert.equal(f.bytes(), original);
    assert.equal(f.writes.length, 0);
    assert.match(f.controls["reading-status"].textContent, /Copy any new notes/);
    assert.doesNotMatch(f.controls["reading-status"].textContent, /export|download/i);
    if (notes) {
      assert.equal(f.controls["lesson-note"].value, "Do not lose this new thought.");
      assert.equal(f.controls["save-status"].textContent, f.controls["reading-status"].textContent);
    }
    assert.equal(f.unloadBlocked(), true);
  }
});

test("unreadable changes from another tab are reported without overwriting either view", async () => {
  const f = fixture();
  f.controls["lesson-note"].value = "This tab's note.";
  f.controls["lesson-note"].dispatch("input");
  await settled();
  const priorWrites = f.writes.length;
  f.replaceBytes("another tab's malformed record"); f.storageEvent();
  assert.match(f.controls["reading-status"].textContent, /could not be merged/);
  assert.equal(f.controls["lesson-note"].value, "This tab's note.");
  f.controls["lesson-note"].value = "Still keep this thought.";
  f.controls["lesson-note"].dispatch("input");
  await settled();
  assert.equal(f.bytes(), "another tab's malformed record");
  assert.equal(f.writes.length, priorWrites);
  assert.equal(f.controls["lesson-note"].value, "Still keep this thought.");
  assert.equal(f.unloadBlocked(), true);
});

test("pages without notes use the general status for uncoordinated saves", async () => {
  const f = fixture({notes: false, locks: false});
  assert.match(f.controls["reading-status"].textContent, /Keep one Fieldnotes tab/);
  f.controls["text-size"].value = "larger";
  f.controls["text-size"].dispatch("change");
  await settled();
  assert.equal(JSON.parse(f.bytes()).settings.textSize, "larger");
  assert.match(f.controls["reading-status"].textContent, /Keep one Fieldnotes tab/);
  assert.equal(f.unloadBlocked(), false);
});
