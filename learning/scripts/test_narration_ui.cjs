// Event-logic checks for the real enhancement; this is not a browser/audio test.
const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

class Element {
  constructor() {
    this.events = new Map(); this.children = []; this.classes = new Set();
    this.classList = {add: name => this.classes.add(name), remove: name => this.classes.delete(name)};
    this.hidden = true; this.textContent = ""; this.dataset = {};
  }
  addEventListener(type, listener) {
    if (!this.events.has(type)) this.events.set(type, []);
    this.events.get(type).push(listener);
  }
  dispatch(type) { for (const listener of this.events.get(type) || []) listener(); }
  append(...children) { this.children.push(...children); }
  scrollIntoView() {}
  click() { this.dispatch("click"); }
}

async function fixture() {
  const player = new Element(), audio = new Element(), status = new Element();
  const toolbar = new Element(), listen = new Element(), stop = new Element();
  const title = new Element(), paragraph = new Element();
  player.dataset.narrationSource = "fixture-cues.json";
  player.querySelector = selector => selector === "audio" ? audio : status;
  audio.paused = true; audio.currentTime = 2;
  audio.play = () => { audio.paused = false; audio.dispatch("play"); return Promise.resolve(); };
  audio.pause = () => { if (!audio.paused) { audio.paused = true; audio.dispatch("pause"); } };
  // Represent the other reader only at its existing Listen/Stop event boundary.
  // The acceptance logic under test comes entirely from narration.js below.
  listen.addEventListener("click", () => { title.classList.add("reading-active"); stop.hidden = false; });
  stop.addEventListener("click", () => { title.classList.remove("reading-active"); stop.hidden = true; });
  const document = {
    querySelector(selector) {
      return new Map([["[data-narration-source]", player], [".reading-toolbar", toolbar],
        ['[data-narration="0"]', title], ['[data-narration="1"]', paragraph]]).get(selector) || null;
    },
    createElement: () => new Element(),
    getElementById: id => id === "listen" ? listen : id === "stop-reading" ? stop : null,
  };
  const cues = {duration: 10, cues: [
    {passage: 0, start: 0, end: 1, text: "Title"},
    {passage: 1, start: 1, end: 10, text: "Paragraph"},
  ]};
  await vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../assets/narration.js"), "utf8"), {
    document, fetch: async () => ({ok: true, json: async () => cues}),
  });
  return {audio, title, paragraph, listen, stop,
    audioHighlights: () => [title, paragraph].filter(node => node.classes.has("narration-active"))};
}

test("Listen clears generated highlighting and delayed paused events cannot restore it", async () => {
  const f = await fixture();
  await f.audio.play();
  assert.deepEqual(f.audioHighlights(), [f.paragraph]);
  f.listen.click();
  assert.equal(f.audio.paused, true);
  assert.equal(f.title.classes.has("reading-active"), true);
  assert.deepEqual(f.audioHighlights(), []);
  f.audio.dispatch("timeupdate");
  f.audio.currentTime = 0.5;
  f.audio.dispatch("seeked");
  assert.deepEqual(f.audioHighlights(), []);
});

test("resuming generated narration stops browser reading and reclaims exactly one passage", async () => {
  const f = await fixture();
  await f.audio.play(); f.listen.click();
  f.audio.currentTime = 0.5;
  await f.audio.play();
  assert.equal(f.title.classes.has("reading-active"), false);
  assert.equal(f.stop.hidden, true);
  assert.deepEqual(f.audioHighlights(), [f.title]);
  f.audio.currentTime = 2; f.audio.dispatch("timeupdate");
  assert.deepEqual(f.audioHighlights(), [f.paragraph]);
});

test("ordinary audio pause and paused seek retain their own passage ownership", async () => {
  const f = await fixture();
  await f.audio.play(); f.audio.pause();
  assert.deepEqual(f.audioHighlights(), [f.paragraph]);
  f.audio.currentTime = 0.5; f.audio.dispatch("seeked");
  assert.deepEqual(f.audioHighlights(), [f.title]);
  assert.equal(f.title.classes.has("reading-active"), false);
  await f.audio.play();
  assert.deepEqual(f.audioHighlights(), [f.title]);
});
