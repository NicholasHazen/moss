/* The projection runs in the checked Rust/Wasm program, never in this adapter. */
(() => {
  "use strict";
  const root = document.querySelector("[data-evidence-preview]");
  if (!root) return;
  root.innerHTML = `<p class="eyebrow">Compiled Rust · isolated Bevy world</p>
    <h3>Follow the projected value into the page</h3>
    <p data-build>Loading the optional local build…</p>
    <form><div class="experiment-controls">
    <div><label for="evidence-used">Used units</label><input id="evidence-used" name="used" type="number" value="3" min="0" max="4294967295" step="1" required></div>
    <div><label for="evidence-capacity">Capacity</label><input id="evidence-capacity" name="capacity" type="number" value="8" min="0" max="4294967295" step="1" required></div>
    <div><label for="evidence-selected">Selected rows</label><select id="evidence-selected" name="selected"><option value="1">One</option><option value="0">None</option><option value="2">Two</option></select></div>
    <div><label for="evidence-ordering">Schedule</label><select id="evidence-ordering" name="ordering"><option value="0">Projection → observer</option><option value="1">Observer only</option><option value="2">Observer → projection</option></select></div>
    </div><button type="submit" disabled>Run this case</button> <button type="button" data-reset disabled>Reset preview</button></form>
    <p class="quiet">Each case starts with a stale display of 99 and an empty trace. The unselected sample remains 1 / 2.</p>
    <dl class="preview-readings"><dt>Current display</dt><dd data-display>—</dd><dt>Observer saw</dt><dd data-observed>—</dd></dl>
    <p data-status role="status"></p>`;
  const form = root.querySelector("form");
  const status = root.querySelector("[data-status]");
  const buttons = [...root.querySelectorAll("button")];
  let api;
  function render() {
    const display = api.fieldnotes_evidence_value(0);
    const observed = api.fieldnotes_evidence_value(1);
    const count = api.fieldnotes_evidence_value(2);
    root.querySelector("[data-display]").textContent = display < 0n ? "Unavailable" : `${display}%`;
    root.querySelector("[data-observed]").textContent = count === 0n ? "No observation yet" : observed < 0n ? "Unavailable" : `${observed}%`;
  }
  function fail(error) {
    buttons.forEach(button => { button.disabled = true; });
    root.querySelector("[data-build]").textContent = "No usable compiled preview is loaded.";
    status.textContent = `The Rust preview is unavailable. ${error.message} The lesson and native exercise remain available.`;
  }
  form.addEventListener("submit", event => {
    event.preventDefault();
    if (!api || !form.reportValidity()) return;
    const values = ["used", "capacity", "selected", "ordering"].map(name => Number(form.elements.namedItem(name).value));
    if (!values.every(value => Number.isInteger(value) && value >= 0 && value <= 4294967295)) return;
    try {
      api.fieldnotes_evidence_reset(...values);
      api.fieldnotes_evidence_step();
      render();
      status.textContent = `Rust case ran with ${values[0]} / ${values[1]} and ${values[2]} selected row${values[2] === 1 ? "" : "s"}. Read the current value and the observer's value separately.`;
    } catch (error) { fail(error); }
  });
  root.querySelector("[data-reset]").addEventListener("click", () => {
    try {
      form.reset();
      api.fieldnotes_evidence_reset(3, 8, 1, 0);
      render();
      status.textContent = "Reset to stale display 99; no schedule has run.";
    } catch (error) { fail(error); }
  });
  (async () => {
    try {
      const response = await fetch("previews/evidence/build.json", {cache: "no-store"});
      if (!response.ok) throw new Error("Build it with learning/scripts/preview.py evidence, then rebuild the course.");
      const metadata = await response.json();
      if (metadata.api !== 1 || !["reference", "learner"].includes(metadata.sourceKind) ||
          !/^[a-f0-9]{64}$/.test(metadata.sourceHash) || metadata.wasm !== `evidence-${metadata.sourceHash}.wasm`) {
        throw new Error("The local build metadata is not recognized.");
      }
      const bytes = await fetch(`previews/evidence/${metadata.wasm}`, {cache: "no-store"});
      if (!bytes.ok) throw new Error("The compiled program could not be loaded.");
      const {instance} = await WebAssembly.instantiate(await bytes.arrayBuffer(), {});
      api = instance.exports;
      if (!["fieldnotes_evidence_reset", "fieldnotes_evidence_step", "fieldnotes_evidence_value"].every(name => typeof api[name] === "function")) {
        throw new Error("The compiled program has a different browser interface.");
      }
      root.querySelector("[data-build]").textContent = `${metadata.sourceKind === "learner" ? "Learner file" : "Printed reference"} · build ${metadata.sourceHash.slice(0, 12)} · Rust ${metadata.rust} · Bevy ECS ${metadata.bevyEcs}. ${metadata.sourceKind === "learner" ? "Rebuild after editing the file; this is the compiled version." : "Build with --file to observe your own checked edit."}`;
      buttons.forEach(button => { button.disabled = false; });
      render();
      status.textContent = "Ready. Predict the display and trace before running a case.";
    } catch (error) { fail(error); }
  })();
})();
