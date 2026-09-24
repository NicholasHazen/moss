/* Read the saved Rust project's owned observations; no ecological rules live here. */
(() => {
  "use strict";
  const root = document.querySelector("[data-ecosystem-preview]");
  if (!root) return;
  root.innerHTML = `<p class="eyebrow">One cumulative Bevy world · compiled Rust</p>
    <h2>Follow a shared meadow through time</h2>
    <p data-build>Loading the optional local course build…</p>
    <fieldset disabled><legend>Execute the selected scenario</legend>
      <label for="meadow-scenario">Scenario preset</label>
      <select id="meadow-scenario"><option value="0">Limited supply preset</option><option value="1">Generous supply preset</option></select>
      <button type="button" data-reset>Start selected scenario</button>
      <button type="button" data-step="1">Step 1 tick</button>
      <button type="button" data-step="4">Step 4 ticks</button>
      <button type="button" data-step="12">Step 12 ticks</button>
      <button type="button" data-step="64">Step 64 ticks</button>
    </fieldset>
    <p class="quiet">Choosing a menu item prepares the next reset. The world changes only when you start a scenario or execute ticks. Feeding sites are authored contacts; this arc does not simulate travel.</p>
    <p data-status role="status"></p>
    <dl class="preview-readings"><dt>Scenario running</dt><dd data-scenario>—</dd><dt>Run / completed tick</dt><dd data-clock>—</dd><dt>Executed light phase</dt><dd data-light>—</dd><dt>Living grazers</dt><dd data-alive>—</dd></dl>
    <div class="table-wrap"><table><caption>Individual state after the last executed tick</caption><thead><tr><th>ID</th><th>Life</th><th>Reserve / capacity</th><th>Upkeep / tick</th><th>Meal request / tick</th><th>Feeding site</th></tr></thead><tbody data-grazers></tbody></table></div>
    <div class="table-wrap"><table><caption>Shared patches</caption><thead><tr><th>ID</th><th>Biomass / capacity</th><th>Growth / lit tick</th></tr></thead><tbody data-patches></tbody></table></div>
    <p data-flows></p>
    <details open><summary>Trajectory observed in this page</summary><p class="quiet">One row per executed tick, including tick zero. The page keeps the newest 65 observations; this table is separate from the simulation’s bounded event history.</p>
      <div class="table-wrap"><table><thead><tr><th>Tick</th><th>Light</th><th>Alive</th><th>Living reserve total</th><th>Patch biomass total</th><th>Added / eaten / upkeep</th></tr></thead><tbody data-trajectory></tbody></table></div>
    </details>
    <details><summary>Retained simulation events and coverage</summary><p data-coverage></p><ol data-events></ol></details>
    <p data-previous class="quiet">Run the first scenario for a fixed tick count, then start the other to compare its ending at the same count.</p>`;
  const select = selector => root.querySelector(selector);
  let api, runningScenario = 0;
  let trajectory = [];
  const names = ["Limited supply", "Generous supply"];
  const phase = value => ["No phase yet", "Dark", "Lit"][Number(value)] ?? "Unknown";
  const read = (table, row, column) => {
    if (api.moss_course_valid(table, row, column) !== 1) throw new Error("The adapter returned an invalid observation address.");
    // WebAssembly i64 results arrive as signed BigInt even for Rust u64.
    return BigInt.asUintN(64, api.moss_course_read(table, row, column));
  };
  function tableRows(body, rows) {
    body.replaceChildren(...rows.map(values => {
      const row = document.createElement("tr");
      values.forEach(value => {
        const cell = document.createElement("td");
        cell.textContent = String(value);
        row.append(cell);
      });
      return row;
    }));
  }
  function snapshot() {
    const summary = Array.from({length: 16}, (_, col) => read(0, 0, col));
    // Bounds apply to display work, not to simulation state. An unsupported
    // learner scenario fails explicitly rather than silently truncating rows.
    if (summary[3] > 10000n || summary[4] > 10000n || summary[5] > 10000n) {
      throw new Error("This browser view supports at most 10,000 rows per table.");
    }
    const rows = (table, count, columns) => Array.from({length: Number(count)}, (_, row) =>
      Array.from({length: columns}, (_, col) => read(table, row, col)));
    return {summary, grazers: rows(1, summary[3], 8), patches: rows(2, summary[4], 4), events: rows(3, summary[5], 6)};
  }
  function observation(state) {
    const alive = state.grazers.filter(grazer => grazer[5] === 1n);
    return {tick: state.summary[1], lit: state.summary[2], alive: alive.length,
      reserve: alive.reduce((sum, grazer) => sum + grazer[1], 0n),
      biomass: state.patches.reduce((sum, patch) => sum + patch[1], 0n),
      added: state.summary[6], eaten: state.summary[8], upkeep: state.summary[7]};
  }
  function render(state) {
    const s = state.summary;
    select("[data-scenario]").textContent = names[runningScenario];
    select("[data-clock]").textContent = `${s[0]} / ${s[1]}`;
    select("[data-light]").textContent = `${phase(s[2])}; ${s[15]} lit ticks per ${s[14]}-tick cycle`;
    select("[data-alive]").textContent = `${state.grazers.filter(g => g[5] === 1n).length} / ${state.grazers.length} retained individuals`;
    tableRows(select("[data-grazers]"), state.grazers.map(g => [g[0], g[5] === 1n ? "Alive" : "Dead", `${g[1]} / ${g[2]}`, g[3], g[4], g[6] === 1n ? g[7] : "None"]));
    tableRows(select("[data-patches]"), state.patches.map(p => [p[0], `${p[1]} / ${p[2]}`, p[3]]));
    select("[data-flows]").textContent = `Latest tick: ${s[6]} units actually added to patches, ${s[8]} eaten, ${s[7]} spent on upkeep, ${s[9]} starvations.`;
    tableRows(select("[data-trajectory]"), trajectory.map(t => [t.tick, phase(t.lit), t.alive, t.reserve, t.biomass, `${t.added} / ${t.eaten} / ${t.upkeep}`]));
    const covered = s[11] > s[10];
    select("[data-coverage]").textContent = `${s[5]} retained events; limit ${s[13]}; ${s[12]} events evicted. ${covered ? `Complete coverage: ticks ${s[10] + 1n} through ${s[11]}, inclusive.` : "No complete tick interval is currently covered."} Events from a partially retained tick may appear below; absence there does not establish zero events.`;
    select("[data-events]").replaceChildren(...state.events.map(e => {
      const item = document.createElement("li");
      const outcome = [
        `Patch ${e[3]} gained ${e[5]} units`, `Grazer ${e[3]} spent ${e[5]} units on upkeep`,
        `Grazer ${e[3]} ate ${e[5]} units from patch ${e[4]}`, `Grazer ${e[3]} starved`,
      ][Number(e[2])] ?? "Unknown event kind";
      item.textContent = `Run ${e[0]}, tick ${e[1]}: ${outcome}.`;
      return item;
    }));
  }
  function record() {
    const state = snapshot();
    trajectory.push(observation(state));
    trajectory = trajectory.slice(-65);
    return state;
  }
  function fail(error) {
    select("fieldset").disabled = true;
    select("[data-build]").textContent = "No usable checked preview is currently available.";
    select("[data-status]").textContent = `The Rust view is unavailable: ${error.message} The explanation and native project remain usable.`;
  }
  root.querySelectorAll("[data-step]").forEach(button => button.addEventListener("click", () => {
    try {
      const ticks = Number(button.dataset.step);
      let state;
      for (let i = 0; i < ticks; i++) {
        api.moss_course_step();
        state = record();
      }
      render(state);
      select("[data-status]").textContent = `Executed ${ticks} tick${ticks === 1 ? "" : "s"}; ${names[runningScenario]} is now at tick ${state.summary[1]}.`;
    } catch (error) { fail(error); }
  }));
  select("[data-reset]").addEventListener("click", () => {
    try {
      const previous = trajectory.at(-1);
      const next = Number(select("#meadow-scenario").value);
      if (api.moss_course_reset(next) !== 1) throw new Error("Rust rejected the selected scenario.");
      if (previous) select("[data-previous]").textContent = `Previous observed ending — ${names[runningScenario]}, tick ${previous.tick}: ${previous.alive} alive, ${previous.reserve} living reserve units, ${previous.biomass} patch biomass units. Compare at that same tick; equal potential supply is not necessarily equal accepted growth.`;
      runningScenario = next;
      trajectory = [];
      const state = record();
      render(state);
      select("[data-status]").textContent = `Started ${names[runningScenario]}, run ${state.summary[0]}, tick zero. No tick has executed.`;
    } catch (error) { fail(error); }
  });
  (async () => {
    try {
      const response = await fetch("previews/ecosystem/build.json", {cache: "no-store"});
      if (!response.ok) throw new Error("Build it with fieldwork_preview.py --reference or --project, then run build.py.");
      const metadata = await response.json();
      if (metadata.api !== 1 || !["learner", "reference"].includes(metadata.sourceKind) ||
          !/^[a-f0-9]{64}$/.test(metadata.buildHash) || !/^[a-f0-9]{64}$/.test(metadata.sourceHash) ||
          !/^[a-f0-9]{64}$/.test(metadata.wasmHash) || metadata.wasm !== `ecosystem-${metadata.buildHash}.wasm`) {
        throw new Error("Unrecognized build metadata.");
      }
      const binaryResponse = await fetch(`previews/ecosystem/${metadata.wasm}`, {cache: "no-store"});
      if (!binaryResponse.ok) throw new Error("The compiled binary could not be loaded.");
      const bytes = await binaryResponse.arrayBuffer();
      const digest = await crypto.subtle.digest("SHA-256", bytes);
      const hash = [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("");
      if (hash !== metadata.wasmHash) throw new Error("The binary does not match its checked build record.");
      const {instance} = await WebAssembly.instantiate(bytes, {});
      api = instance.exports;
      if (!["moss_course_reset", "moss_course_step", "moss_course_read", "moss_course_valid"].every(name => typeof api[name] === "function")) {
        throw new Error("The compiled program has a different browser interface.");
      }
      select("[data-build]").textContent = `${metadata.sourceKind === "learner" ? "Learner project" : "Course reference"} · saved source ${metadata.sourceHash.slice(0, 12)} · build ${metadata.buildHash.slice(0, 12)} · Rust ${metadata.rust}. This runs compiled source; after an edit, check and rebuild to observe it here.`;
      render(record());
      select("fieldset").disabled = false;
      select("[data-status]").textContent = "Ready at tick zero. Predict which grazer first misses a meal.";
    } catch (error) { fail(error); }
  })();
})();
