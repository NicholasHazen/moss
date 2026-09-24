/* One checked Rust world. JavaScript formats owned observations and records runs;
   it does not calculate meals, eligibility, births, traits or death. */
(() => {
  "use strict";
  const root = document.querySelector("[data-population-preview]");
  if (!root) return;
  root.innerHTML = `<p class="eyebrow">The continuing meadow · compiled Rust</p>
    <h2>Follow the descendants, then account for them</h2>
    <p data-build>Loading the optional local population build…</p>
    <fieldset disabled><legend>Prepare the next run</legend>
      <label for="population-environment">Food supply</label>
      <select id="population-environment"><option value="0">Steady: 6 units each tick</option><option value="1">Pulsed: 12 units on 2 of every 4 ticks</option></select>
      <label for="population-cohort">Founder upkeep per tick</label>
      <select id="population-cohort"><option value="0">1 and 1</option><option value="1">1 and 2</option><option value="2">2 and 2</option><option value="3">1 and 3</option></select>
      <label for="population-mutation">Authored mutation cycle</label>
      <select id="population-mutation"><option value="0">No change: [0]</option><option value="1">Repeat [-1, 0, +1, 0]</option></select>
      <div class="button-row"><button type="button" data-reset>Start selected population</button>
        <button type="button" data-step="1">Step 1 tick</button><button type="button" data-step="4">Step 4 ticks</button>
        <button type="button" data-step="40">Step 40 ticks</button><button type="button" data-save>Keep this ending</button></div>
    </fieldset>
    <p class="quiet">Menus prepare a reset. They do not alter the running world. A reset begins a fresh run; tick buttons execute Rust. Contacts are authored shared feeding sites. This view is a global inspector, not an animal’s perception.</p>
    <p data-status role="status"></p>
    <dl class="preview-readings"><dt>Running inputs</dt><dd data-inputs>—</dd><dt>Run / completed tick</dt><dd data-clock>—</dd>
      <dt>Living population</dt><dd data-census>—</dd><dt>Since this run began</dt><dd data-totals>—</dd><dt>Latest tick</dt><dd data-flows>—</dd></dl>
    <figure><svg data-chart viewBox="0 0 560 180" aria-hidden="true"></svg>
      <figcaption data-chart-caption>Living population by completed tick. The table below gives the same observations.</figcaption></figure>
    <details open><summary>Living individuals and their next opportunities</summary>
      <div class="table-wrap"><table><caption>Owned census after the last completed tick; dead individuals have left this roster</caption><thead><tr><th>ID</th><th>Stage</th><th>Reserve / capacity</th><th>Upkeep trait / rate per eligible tick</th><th>Meal request</th><th>Site</th><th>First upkeep / meal tick</th><th>Maturation tick</th><th>Next possible birth tick</th></tr></thead><tbody data-individuals></tbody></table></div>
      <p class="quiet">Reaching the next birth tick only satisfies a timing condition. Adult stage, a valid shared patch, available reserve, a partner and capacity are still required.</p>
      <div class="table-wrap"><table><caption>Current shared patches</caption><thead><tr><th>ID</th><th>Biomass / capacity</th><th>Growth per lit tick</th></tr></thead><tbody data-patches></tbody></table></div>
    </details>
    <details><summary>Where each living descendant’s trait came from</summary>
      <div class="table-wrap"><table><thead><tr><th>Child</th><th>Birth run / tick / ordinal</th><th>Parents</th><th>Donor</th><th>Inherited</th><th>Proposed / applied change</th><th>Child trait</th></tr></thead><tbody data-origins></tbody></table></div>
      <p class="quiet">Founders have no parent record. Parent IDs refer to this run even after a parent dies. Proposed changes can be clipped by the trait bounds. Lower upkeep has no modeled disadvantage here.</p>
    </details>
    <details><summary>Exact starting inputs and active birth policy</summary><p data-policy></p><p data-initial></p>
      <div class="table-wrap"><table><caption>Founder inputs</caption><thead><tr><th>ID</th><th>Reserve / capacity</th><th>Upkeep</th><th>Meal request</th><th>Site</th></tr></thead><tbody data-founders></tbody></table></div>
      <div class="table-wrap"><table><caption>Starting patch inputs</caption><thead><tr><th>ID</th><th>Biomass / capacity</th><th>Growth per lit tick</th></tr></thead><tbody data-initial-patches></tbody></table></div>
    </details>
    <details><summary>Per-tick observations recorded by this page</summary>
      <p class="quiet">The newest 129 observations, including tick zero when retained. These observations are separate from the two bounded simulation journals.</p>
      <div class="table-wrap"><table><thead><tr><th>Tick</th><th>Light</th><th>Living / juvenile / adult</th><th>Reserve / biomass</th><th>Births / deaths</th><th>Added / eaten / upkeep</th><th>Birth transfer / cost</th></tr></thead><tbody data-trajectory></tbody></table></div>
    </details>
    <details><summary>Events and the limits of what they establish</summary><p data-coverage></p>
      <h3>Births and maturation</h3><ol data-population-events></ol><h3>Growth, meals, upkeep and starvation</h3><ol data-resource-events></ol>
    </details>
    <details open><summary>Compare saved endings</summary>
      <p class="quiet">Keep endings at the same tick. This page holds at most 12 until you reload. Totals are measured from every tick executed since each reset; they are not reconstructed from evicted events. Repeating identical deterministic inputs checks repeatability, not independent ecological samples.</p>
      <div class="table-wrap"><table><thead><tr><th>Run</th><th>Inputs</th><th>Tick</th><th>Living / juvenile / adult</th><th>Births / deaths</th><th>Added / eaten / upkeep</th><th>Birth transfer / cost</th><th>Cap-blocked pairs / ticks</th></tr></thead><tbody data-comparisons></tbody></table></div>
    </details>`;
  const select = selector => root.querySelector(selector);
  const phase = value => ["Not executed", "Dark", "Lit"][Number(value)] ?? "Unknown";
  const signed = value => BigInt.asIntN(64, value);
  let api, current, running = [0, 0, 0], trajectory = [], comparisons = [];
  let totals = {added: 0n, eaten: 0n, upkeep: 0n, transferred: 0n, dissipated: 0n};
  function read(table, row, column) {
    if (api.moss_population_valid(table, row, column) !== 1) throw new Error("The population adapter returned an invalid address.");
    return BigInt.asUintN(64, api.moss_population_read(table, row, column));
  }
  function rows(table, count, columns) {
    if (count > 10000n) throw new Error("This browser view supports at most 10,000 rows per table.");
    return Array.from({length: Number(count)}, (_, row) => Array.from({length: columns}, (_, col) => read(table, row, col)));
  }
  function snapshot() {
    const summary = rows(0, 1n, 36)[0], config = rows(1, 1n, 10)[0];
    return {summary, config, individuals: rows(2, summary[3], 24), patches: rows(3, summary[4], 4),
      resources: rows(4, summary[5], 6), events: rows(5, summary[6], 17),
      mutation: rows(6, config[7], 1).map(row => signed(row[0])),
      founders: rows(7, summary[34], 7), initialPatches: rows(8, summary[35], 4)};
  }
  function tableRows(body, data) {
    body.replaceChildren(...data.map(values => {
      const row = document.createElement("tr");
      values.forEach(value => { const cell = document.createElement("td"); cell.textContent = String(value); row.append(cell); });
      return row;
    }));
  }
  function description(inputs) {
    return `${inputs[0] === 0 ? "Steady" : "Pulsed"}; founders ${["1 / 1", "1 / 2", "2 / 2", "1 / 3"][inputs[1]]}; mutation ${inputs[2] === 0 ? "[0]" : "[-1, 0, +1, 0]"}`;
  }
  function observe() {
    current = snapshot();
    const s = current.summary;
    trajectory.push({tick: s[1], phase: s[2], living: s[7], juvenile: s[8], adult: s[9],
      reserve: current.individuals.reduce((sum, g) => sum + g[1], 0n),
      biomass: current.patches.reduce((sum, p) => sum + p[1], 0n),
      added: s[15], upkeep: s[16], eaten: s[17], deaths: s[18], births: s[19], transferred: s[20], dissipated: s[21]});
    trajectory = trajectory.slice(-129);
    totals.added += s[15]; totals.upkeep += s[16]; totals.eaten += s[17];
    totals.transferred += s[20]; totals.dissipated += s[21];
  }
  function chart() {
    const svg = select("[data-chart]"), ns = "http://www.w3.org/2000/svg";
    const element = (tag, attrs, text) => {
      const el = document.createElementNS(ns, tag);
      Object.entries(attrs).forEach(([name, value]) => el.setAttribute(name, String(value)));
      if (text !== undefined) el.textContent = text;
      return el;
    };
    const first = trajectory[0].tick, last = trajectory.at(-1).tick;
    const span = Number(last - first) || 1;
    const maximum = Math.max(1, ...trajectory.map(t => Number(t.living)));
    const points = key => trajectory.map(t => `${40 + Number(t.tick - first) / span * 500},${140 - Number(t[key]) / maximum * 115}`).join(" ");
    svg.replaceChildren(element("path", {d: "M40 20 V140 H540", fill: "none", stroke: "currentColor"}),
      element("text", {x: 30, y: 30, "text-anchor": "end", "font-size": 12}, maximum),
      element("text", {x: 30, y: 143, "text-anchor": "end", "font-size": 12}, "0"),
      element("text", {x: 40, y: 164, "font-size": 12}, `Tick ${first}`),
      element("text", {x: 540, y: 164, "text-anchor": "end", "font-size": 12}, `Tick ${last}`),
      element("polyline", {points: points("living"), fill: "none", stroke: "#244629", "stroke-width": 3}),
      element("polyline", {points: points("juvenile"), fill: "none", stroke: "#725013", "stroke-width": 2, "stroke-dasharray": "5 4"}));
    select("[data-chart-caption]").textContent = `Living individuals (solid green) and juveniles (dashed brown), ticks ${first}–${last}. The vertical scale is 0–${maximum} and adapts to this run; compare table values across runs. ${trajectory.at(-1).living} living and ${trajectory.at(-1).juvenile} juvenile at the latest observation.`;
  }
  function eventList(selector, data, describe) {
    select(selector).replaceChildren(...data.map(event => {
      const item = document.createElement("li");
      item.textContent = `Run ${event[0]}, tick ${event[1]}: ${describe(event)}`;
      return item;
    }));
  }
  function render() {
    const s = current.summary, c = current.config;
    select("[data-inputs]").textContent = description(running);
    select("[data-clock]").textContent = `${s[0]} / ${s[1]} · ${phase(s[2])}`;
    select("[data-census]").textContent = `${s[7]} living: ${s[8]} juvenile, ${s[9]} adult. Authored cap: ${c[6]}.`;
    select("[data-totals]").textContent = `${s[10]} founders + ${s[11]} births − ${s[12]} starvations = ${s[7]} living. The cap blocked ${s[14]} otherwise eligible pairs on ${s[13]} ticks.`;
    select("[data-flows]").textContent = `${s[15]} units added, ${s[17]} eaten, ${s[16]} upkeep spent; ${s[19]} births transferred ${s[20]} units and dissipated ${s[21]}; ${s[18]} starvations.`;
    tableRows(select("[data-individuals]"), current.individuals.map(g => [g[0], g[7] === 1n ? "Adult" : "Juvenile", `${g[1]} / ${g[2]}`, `${g[12]} / ${g[3]}`, g[4], g[5] === 1n ? g[6] : "None", g[8], g[9] === 1n ? g[10] : "Founder adult", g[11]]));
    const patches = data => data.map(p => [p[0], `${p[1]} / ${p[2]}`, p[3]]);
    tableRows(select("[data-patches]"), patches(current.patches));
    tableRows(select("[data-origins]"), current.individuals.filter(g => g[13] === 1n).map(g => [g[0], `${g[14]} / ${g[15]} / ${g[16]}`, `${g[17]}, ${g[18]}`, g[19], g[20], `${signed(g[21])} / ${signed(g[22])}`, g[23]]));
    select("[data-policy]").textContent = `Each parent contributes ${c[2]} reserve units and dissipates ${c[3]}, while retaining at least one. A child has capacity ${c[4]}, requests ${c[5]} per meal, first eats and pays upkeep on the tick after birth, and matures ${c[0]} ticks after birth. Parent cooldown: ${c[1]} ticks. Maximum living: ${c[6]}. Mutation repeats [${current.mutation.join(", ")}]. Pairing uses adjacent eligible IDs within each valid shared patch; accepted births alternate the lower and higher parent as trait donor. IDs are never recycled within a run.`;
    select("[data-initial]").textContent = `${s[33]} lit ticks in each ${s[32]}-tick cycle. Resource journal limit ${c[8]}, population journal limit ${c[9]}. These are the actual starting inputs and policy reported by the compiled program.`;
    tableRows(select("[data-founders]"), current.founders.map(g => [g[0], `${g[1]} / ${g[2]}`, g[3], g[4], g[5] === 1n ? g[6] : "None"]));
    tableRows(select("[data-initial-patches]"), patches(current.initialPatches));
    tableRows(select("[data-trajectory]"), trajectory.map(t => [t.tick, phase(t.phase), `${t.living} / ${t.juvenile} / ${t.adult}`, `${t.reserve} / ${t.biomass}`, `${t.births} / ${t.deaths}`, `${t.added} / ${t.eaten} / ${t.upkeep}`, `${t.transferred} / ${t.dissipated}`]));
    const coverage = (after, through) => through > after ? `ticks ${after + 1n}–${through}, inclusive` : "no complete tick interval";
    const commonAfter = s[23] > s[26] ? s[23] : s[26], commonThrough = s[24] < s[27] ? s[24] : s[27];
    select("[data-coverage]").textContent = `Resource journal: ${s[5]} retained, ${s[25]} evicted; complete coverage ${coverage(s[23], s[24])}. Population journal: ${s[6]} retained, ${s[28]} evicted; complete coverage ${coverage(s[26], s[27])}. Both together cover ${coverage(commonAfter, commonThrough)}. A missing event outside the relevant complete interval is unknown, not evidence of zero. Run totals above remain cumulative even after eviction.`;
    eventList("[data-population-events]", current.events, e => e[2] === 1n
      ? `Individual ${e[3]} matured; upkeep trait ${e[4]}.`
      : `Child ${e[3]} born to ${e[6]} and ${e[7]} (birth ordinal ${e[5]}). Donor ${e[8]} supplied trait ${e[9]}; proposed change ${signed(e[10])}, applied ${signed(e[11])}, child trait ${e[4]}. Each parent contributed ${e[12]} and dissipated ${e[13]}; child reserve ${e[14]}, first upkeep/meal tick ${e[15]}, maturation tick ${e[16]}.`);
    eventList("[data-resource-events]", current.resources, e => [
      `Patch ${e[3]} gained ${e[5]} units.`, `Individual ${e[3]} spent ${e[5]} units on upkeep.`,
      `Individual ${e[3]} ate ${e[5]} units from patch ${e[4]}.`, `Individual ${e[3]} starved.`,
    ][Number(e[2])] ?? "Unrecognized event kind.");
    chart();
  }
  function fail(error) {
    select("fieldset").disabled = true;
    select("[data-build]").textContent = "No usable checked population preview is currently available.";
    select("[data-status]").textContent = `${error.message} The explanation and native project remain usable.`;
  }
  root.querySelectorAll("[data-step]").forEach(button => button.addEventListener("click", () => {
    try {
      const count = Number(button.dataset.step);
      for (let i = 0; i < count; i++) { api.moss_population_step(); observe(); }
      render();
      select("[data-status]").textContent = `Executed ${count} tick${count === 1 ? "" : "s"}; run ${current.summary[0]} is at tick ${current.summary[1]}.`;
    } catch (error) { fail(error); }
  }));
  select("[data-reset]").addEventListener("click", () => {
    try {
      const next = ["environment", "cohort", "mutation"].map(key => Number(select(`#population-${key}`).value));
      if (api.moss_population_reset(...next) !== 1) throw new Error("Rust rejected the selected population inputs.");
      running = next; trajectory = [];
      totals = {added: 0n, eaten: 0n, upkeep: 0n, transferred: 0n, dissipated: 0n};
      observe(); render();
      select("[data-status]").textContent = `Started run ${current.summary[0]} at tick zero. No tick has executed.`;
    } catch (error) { fail(error); }
  });
  select("[data-save]").addEventListener("click", () => {
    const s = current.summary;
    comparisons.push([s[0], description(running), s[1], `${s[7]} / ${s[8]} / ${s[9]}`, `${s[11]} / ${s[12]}`,
      `${totals.added} / ${totals.eaten} / ${totals.upkeep}`, `${totals.transferred} / ${totals.dissipated}`, `${s[14]} / ${s[13]}`]);
    comparisons = comparisons.slice(-12);
    tableRows(select("[data-comparisons]"), comparisons);
    select("[data-status]").textContent = `Kept run ${s[0]} at tick ${s[1]}; ${comparisons.length} ending${comparisons.length === 1 ? "" : "s"} retained on this page.`;
  });
  (async () => {
    try {
      const response = await fetch("previews/population/build.json", {cache: "no-store"});
      if (!response.ok) throw new Error("Build with fieldwork_preview.py --population and --project or --reference, then run build.py.");
      const metadata = await response.json();
      if (metadata.api !== 1 || metadata.mode !== "population" || !["learner", "reference"].includes(metadata.sourceKind) ||
          ![metadata.buildHash, metadata.sourceHash, metadata.wasmHash].every(value => /^[a-f0-9]{64}$/.test(value)) ||
          metadata.wasm !== `population-${metadata.buildHash}.wasm`) throw new Error("Unrecognized population build metadata.");
      const binaryResponse = await fetch(`previews/population/${metadata.wasm}`, {cache: "no-store"});
      if (!binaryResponse.ok) throw new Error("The compiled population binary could not be loaded.");
      const bytes = await binaryResponse.arrayBuffer();
      const digest = await crypto.subtle.digest("SHA-256", bytes);
      const hash = [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("");
      if (hash !== metadata.wasmHash) throw new Error("The population binary does not match its checked build record.");
      const {instance} = await WebAssembly.instantiate(bytes, {});
      api = instance.exports;
      if (!["moss_population_reset", "moss_population_step", "moss_population_read", "moss_population_valid"].every(name => typeof api[name] === "function")) throw new Error("The compiled program has a different population interface.");
      select("[data-build]").textContent = `${metadata.sourceKind === "learner" ? "Learner project" : "Course reference"} · saved source ${metadata.sourceHash.slice(0, 12)} · build ${metadata.buildHash.slice(0, 12)} · Rust ${metadata.rust}. After an edit, check and rebuild to observe it here. All saved endings below come from this loaded binary.`;
      observe(); render(); select("fieldset").disabled = false;
      select("[data-status]").textContent = "Ready at tick zero. Predict the reserves after the first accepted birth.";
    } catch (error) { fail(error); }
  })();
})();
