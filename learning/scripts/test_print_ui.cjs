// Print-event logic from the real enhancement; this is not a browser/PDF test.
const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

function fixture() {
  const source = fs.readFileSync(path.join(__dirname, "../assets/fieldnotes.js"), "utf8");
  const start = source.indexOf("  let printedDetails =");
  const end = source.indexOf("  const listen =", start);
  assert.ok(start >= 0 && end > start, "The actual print handlers must be loaded");
  const closed = {open: false}, alreadyOpen = {open: true}, nested = {open: false};
  const nodes = [closed, alreadyOpen, nested];
  const events = new Map();
  let queries = 0;
  vm.runInNewContext(source.slice(start, end), {
    window: {
      addEventListener(name, listener) {
        if (!events.has(name)) events.set(name, []);
        events.get(name).push(listener);
      },
    },
    document: {
      querySelectorAll(selector) {
        assert.equal(selector, "article details:not([open])");
        queries++;
        return nodes.filter(node => !node.open);
      },
    },
  });
  return {closed, alreadyOpen, nested, state: () => nodes.map(node => node.open),
    queries: () => queries,
    dispatch(name) {
      assert.ok(events.has(name), `Missing ${name} handler`);
      for (const listener of events.get(name)) listener();
    }};
}

test("printing expands closed answers and restores them without closing pre-opened answers", () => {
  const f = fixture();
  f.dispatch("beforeprint");
  assert.deepEqual(f.state(), [true, true, true]);
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [false, true, false]);
});

test("repeated beforeprint retains the original disclosure snapshot", () => {
  const f = fixture();
  f.dispatch("beforeprint");
  f.dispatch("beforeprint");
  assert.deepEqual(f.state(), [true, true, true]);
  assert.equal(f.queries(), 1);
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [false, true, false]);
});

test("a later print session uses the reader's newly chosen disclosure state", () => {
  const f = fixture();
  f.dispatch("beforeprint");
  f.dispatch("afterprint");
  f.closed.open = true;
  f.alreadyOpen.open = false;
  f.dispatch("beforeprint");
  assert.deepEqual(f.state(), [true, true, true]);
  assert.equal(f.queries(), 2);
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [true, false, false]);
});

test("unmatched and duplicate afterprint cannot change the reader's answers", () => {
  const f = fixture();
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [false, true, false]);
  assert.equal(f.queries(), 0);
  f.dispatch("beforeprint");
  f.dispatch("afterprint");
  f.closed.open = true;
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [true, true, false]);
});

test("an all-open session still ends before a later session is captured", () => {
  const f = fixture();
  f.closed.open = true;
  f.nested.open = true;
  f.dispatch("beforeprint");
  f.dispatch("beforeprint");
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [true, true, true]);
  f.nested.open = false;
  f.dispatch("beforeprint");
  assert.equal(f.nested.open, true);
  f.dispatch("afterprint");
  assert.deepEqual(f.state(), [true, true, false]);
});
