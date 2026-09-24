// Exercise the real Listen handler with DOM/speech boundaries represented below.
// This checks passage selection and order, not browser layout or voice playback.
const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

class Element {
  constructor(tag, text = "", parent = null, classes = []) {
    this.tagName = tag.toUpperCase(); this.textContent = text;
    this.parentElement = parent; this.children = []; this.events = new Map();
    this.classes = new Set(classes); this.open = false;
    this.classList = {add: value => this.classes.add(value),
      remove: value => this.classes.delete(value), contains: value => this.classes.has(value)};
    if (parent) parent.children.push(this);
  }
  matches(selector) {
    if (selector.startsWith(".")) return this.classes.has(selector.slice(1));
    if (selector === "details:not([open])") return this.tagName === "DETAILS" && !this.open;
    return this.tagName.toLowerCase() === selector;
  }
  closest(selector) {
    for (let node = this; node; node = node.parentElement) {
      if (selector.split(",").some(part => node.matches(part.trim()))) return node;
    }
    return null;
  }
  querySelector(selector) {
    for (const child of this.children) {
      if (child.matches(selector)) return child;
      const descendant = child.querySelector(selector);
      if (descendant) return descendant;
    }
    return null;
  }
  getClientRects() { return [{}]; }
  scrollIntoView() {}
  addEventListener(type, listener) { this.events.set(type, listener); }
  click() { this.events.get("click")(); }
}

test("Listen skips expanded chapter navigation while keeping overview, lesson lists and summary in order", () => {
  const article = new Element("article");
  const candidates = [];
  function passage(tag, text, parent = article) {
    const node = new Element(tag, text, parent); candidates.push(node); return node;
  }
  passage("h1", "Keep the original owner");
  const overview = new Element("section", "", article, ["lesson-overview"]);
  passage("h2", "What you will investigate", overview);
  passage("p", "Follow a reserve through one borrowed call.", overview);
  passage("li", "Predict which owner remains available.", new Element("ul", "", overview));
  const contents = new Element("details", "", article, ["chapter-contents"]);
  contents.open = true; // These links are visible, so closed-details filtering cannot hide them.
  const links = new Element("ol", "", new Element("nav", "", contents));
  const firstLink = passage("li", "Read the borrow", links);
  const secondLink = passage("li", "Carry the lesson forward", links);
  passage("h2", "Read the borrow");
  passage("li", "Borrow the reserve to inspect it.", new Element("ul", "", article));
  passage("li", "Use the same owner after the call.", new Element("ol", "", article));
  const summary = new Element("section", "", article, ["lesson-summary"]);
  passage("h2", "Carry the lesson forward", summary);
  passage("p", "A borrowed view does not consume its owner.", summary);

  const controls = Object.fromEntries(["listen", "pause-reading", "stop-reading", "speech-status"]
    .map(id => [id, new Element("button")]));
  const pending = [], spoken = [];
  const source = fs.readFileSync(path.join(__dirname, "../assets/fieldnotes.js"), "utf8");
  const start = source.indexOf("  const listen =");
  const end = source.indexOf("  sync();", start);
  assert.ok(start >= 0 && end > start, "Load the actual Listen handler");
  vm.runInNewContext(source.slice(start, end), {
    document: {
      getElementById: id => controls[id],
      querySelectorAll(selector) {
        // Apply the handler's selector to the candidate article descendants.
        const tags = selector.split(",").map(part => part.trim().replace(/^article /, ""));
        return candidates.filter(node => tags.some(tag => node.matches(tag)));
      },
    },
    window: {addEventListener() {}, speechSynthesis: {
      cancel() { pending.length = 0; },
      speak(utterance) { spoken.push(utterance.text); pending.push(utterance); },
    }},
    SpeechSynthesisUtterance: class { constructor(text) { this.text = text; } },
  });
  controls.listen.click();
  let completed = 0;
  while (pending.length) {
    assert.ok(completed++ < 20, "Speech must advance to completion");
    const utterance = pending.shift(); utterance.onstart(); utterance.onend();
  }
  assert.deepEqual(spoken, [
    "Keep the original owner", "What you will investigate",
    "Follow a reserve through one borrowed call.", "Predict which owner remains available.",
    "Read the borrow", "Borrow the reserve to inspect it.", "Use the same owner after the call.",
    "Carry the lesson forward", "A borrowed view does not consume its owner.",
  ]);
  assert.equal(contents.open, true, "Listening must not collapse the reader's navigation");
  assert.equal(firstLink.classes.has("reading-active"), false);
  assert.equal(secondLink.classes.has("reading-active"), false);
  assert.equal(controls["speech-status"].textContent, "Finished reading this lesson.");
  assert.equal(controls.listen.disabled, false);
});
