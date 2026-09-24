/* The map projects owned Rust observations. It never chooses a target, moves
   an animal, resolves a meal, or changes the ECS world outside exported ticks. */
(() => {
  "use strict";
  const root = document.querySelector("[data-mobile-preview], [data-resting-preview], [data-hunting-preview]");
  if (!root) return;
  const hunting = root.hasAttribute("data-hunting-preview");
  const resting = hunting || root.hasAttribute("data-resting-preview");
  const mode = hunting ? "hunting" : resting ? "resting" : "mobile";
  const prefix = `moss_${mode}_`;
  root.innerHTML = `<p class="eyebrow">The continuing meadow · compiled Rust</p>
    <h2>${hunting ? "One capture changes the rest of the tick" : resting ? "Recovered enough, but still committed to rest" : "Seeing, travelling and reaching are different events"}</h2>
    <p data-build>Loading the optional checked spatial build…</p>
    <fieldset disabled><legend>Prepare a run and inspect its inhabitants</legend>
      <label for="mobile-preset">Starting world</label><select id="mobile-preset"><option value="0">${hunting ? "Two hunters, one contested prey" : resting ? "An interrupted journey to finite food" : "A short journey to finite food"}</option><option value="1">A meadow with two renewing patches</option></select>
      ${resting ? `<label for="resting-minimum">Minimum committed rest</label><select id="resting-minimum"><option value="2">2 executed rest ticks</option><option value="3">3 executed rest ticks</option></select>${hunting ? `<label for="hunting-enabled">Hunters in the next world</label><select id="hunting-enabled"><option value="1">Two authored hunters</option><option value="0">No hunters</option></select>` : `<label for="resting-recovery">Recovery per rest action</label><select id="resting-recovery"><option value="2">2 fatigue points</option><option value="6">6 fatigue points (fast recovery)</option></select>`}` : ""}
      <div class="button-row"><button type="button" data-reset>Start selected world</button><button type="button" data-step="1">Step 1 tick</button><button type="button" data-step="3">Step 3 ticks</button><button type="button" data-step="40">Step 40 ticks</button><button type="button" data-save>Keep this ending</button></div>
      <label for="mobile-actor">Inspect a living ${hunting ? "animal" : "grazer"}</label><select id="mobile-actor"></select>
    </fieldset>
    <p class="quiet">The starting-world menu prepares the next reset. Choosing an animal only changes the inspector. Tick buttons run the checked Rust program. No tick runs automatically.</p>
    <p data-status role="status"></p>
    <dl class="preview-readings"><dt>Running inputs</dt><dd data-inputs>—</dd><dt>Run / completed tick</dt><dd data-clock>—</dd><dt>Population</dt><dd data-census>—</dd><dt>Latest tick</dt><dd data-flows>—</dd><dt>Whole-run accounting</dt><dd data-account>—</dd></dl>
    <figure><svg data-map viewBox="0 0 600 340" aria-hidden="true"></svg><figcaption data-map-caption></figcaption></figure>
    ${resting ? `<h3>Activity is separate from the food budget</h3><p data-rest-policy></p><p data-rest-totals></p><div class="table-wrap"><table><caption>Fatigue and commitments after the completed tick</caption><thead><tr><th>Grazer</th><th>Fatigue points</th><th>Activity</th><th>Rest actions still owed</th><th>Last actual rest tick</th><th>Last activity transition tick</th></tr></thead><tbody data-rest-actors></tbody></table></div><p class="quiet">Zero actions owed does not mean the actor has already woken. A later decision must also meet the exit threshold. Upkeep continues during rest; recovery reduces fatigue and never credits reserve.</p>` : ""}
    <h3>The selected animal’s recorded local view</h3><p data-local></p>
    <div class="table-wrap"><table><caption data-local-caption>Patches in its last owned observation, before travel and feeding</caption><thead data-local-head><tr><th>Patch</th><th>Observed cell</th><th>Observed biomass</th><th>Current biomass (inspector)</th></tr></thead><tbody data-observations></tbody></table></div>
    ${hunting ? `<h3>Hunters share the world and pay their own costs</h3><p data-hunt-totals></p><div class="table-wrap"><table><caption>Living hunters after the completed tick</caption><thead><tr><th>Hunter / cell</th><th>Reserve / capacity</th><th>Upkeep / attack cost / attack effort</th><th>Target / observed tick</th><th>First eligible / last capture ticks</th><th>Fatigue / activity / rest owed</th><th>Last rest / transition / travel opportunity ticks</th></tr></thead><tbody data-hunters></tbody></table></div><p class="quiet">A hunter's retained prey view can include an animal that has since moved or left the census. A successful capture rechecks current contact and eligibility, pays the attack cost, transfers the entire remaining prey reserve, and marks the prey terminal before later meals and births. Hunters do not reproduce in this model.</p>` : ""}
    <details open><summary>Current positions, reserves and opportunities</summary>
      <div class="table-wrap"><table><caption>Global census after the completed tick</caption><thead><tr><th>ID / stage</th><th>Current cell</th><th>Reserve / capacity</th><th>Upkeep rate / meal request</th><th>Target / observed tick</th><th>Current contact</th><th>First eligible / adult / next birth ticks</th></tr></thead><tbody data-grazers></tbody></table></div>
      <div class="table-wrap"><table><caption>Global patch state after the completed tick</caption><thead><tr><th>Patch</th><th>Cell</th><th>Biomass / capacity</th><th>Growth per lit tick</th></tr></thead><tbody data-patches></tbody></table></div>
      <p class="quiet">A retained target is a proposal. Feeding still checks current contact and available biomass. Reaching a birth deadline is only one condition; reserves, a partner and a valid shared patch also matter.</p>
    </details>
    <details><summary>Parentage and placement of living descendants</summary>
      <div class="table-wrap"><table><thead><tr><th>Child</th><th>Birth run / tick / ordinal</th><th>Parents</th><th>Donor / inherited trait</th><th>Proposed / applied change</th><th>Child trait</th></tr></thead><tbody data-origins></tbody></table></div>
      <p class="quiet">The current cell may differ from the birth cell. Retained placement events appear below. Parent IDs remain meaningful within the run even after a parent leaves the living roster.</p>
    </details>
    <details><summary>Exact starting inputs</summary><p data-policy></p><p data-space-policy></p>
      <div class="table-wrap"><table><caption>Founders</caption><thead><tr><th>ID</th><th>Cell</th><th>Reserve / capacity</th><th>Upkeep / meal</th><th>Sight radius / speed / travel cost</th></tr></thead><tbody data-founders></tbody></table></div>
      ${hunting ? `<p data-hunt-inputs></p><div class="table-wrap"><table><caption>Initial hunters</caption><thead><tr><th>ID / cell</th><th>Reserve / capacity</th><th>Upkeep / attack cost / effort</th><th>Sight / speed / travel cost</th></tr></thead><tbody data-initial-hunters></tbody></table></div>` : ""}
      <div class="table-wrap"><table><caption>Initial patches</caption><thead><tr><th>Patch</th><th>Cell</th><th>Biomass / capacity</th><th>Growth per lit tick</th></tr></thead><tbody data-initial-patches></tbody></table></div>
    </details>
    <details><summary>Per-tick observations recorded by this page</summary><p class="quiet">The newest 129 observations. This table is separate from the ${hunting ? "five" : resting ? "four" : "three"} bounded Rust journals. Whole-run accounting includes every tick this page executed, including observations no longer in this table.</p>
      <div class="table-wrap"><table><thead><tr><th>Tick / light</th><th>Living / juvenile</th><th>Reserve / biomass</th><th>Added / eaten / upkeep</th><th>Travel cells / cost</th><th>Births / grazer starvations</th><th>Birth transfer / cost</th>${resting ? "<th>Grazer fatigue / rest actions</th>" : ""}${hunting ? "<th>Hunters / reserve</th><th>Captures / hunter starvations</th><th>Hunter upkeep / travel / attack / transfer</th>" : ""}</tr></thead><tbody data-trajectory></tbody></table></div>
    </details>
    <details><summary>Events and the limits of the journals</summary><p data-coverage></p>
      ${hunting ? `<h3>Hunting, hunter costs and activity</h3><p data-hunt-coverage></p><ol data-hunt-events></ol>` : ""}
      ${resting ? `<h3>Grazer effort, rest and waking</h3><p data-rest-coverage></p><ol data-rest-events></ol>` : ""}
      <h3>Targets, travel and birth cells</h3><ol data-mobile-events></ol><h3>Births and maturation</h3><ol data-population-events></ol><h3>Growth, upkeep, meals and starvation</h3><ol data-resource-events></ol>
    </details>
    <details open><summary>Compare saved endings</summary><p class="quiet">Up to 12 endings remain until reload. Compare the same inputs and horizon to check repeatability; these deterministic reruns are not independent ecological samples. The two presets answer different questions and are not a controlled experiment.${hunting ? " Adding hunters also adds their initial reserves. Compare total flows, both removal causes and initial stores; a difference in living grazers alone does not isolate the effect of capture from those added stores." : ""}</p>
      <div class="table-wrap"><table><thead><tr><th>Run / inputs / tick</th><th>Living / juvenile</th><th>Births / grazer starvations</th><th>Added / upkeep / travel cost / birth cost</th><th>Current reserve / biomass</th><th>Travel cells / target changes</th>${resting ? "<th>Grazer rest actions / entries / wakes</th>" : ""}${hunting ? "<th>Hunters / reserve / initial stores</th><th>Captures / hunter starvations</th><th>Hunter upkeep / travel / attack / transfer</th>" : ""}</tr></thead><tbody data-comparisons></tbody></table></div>
    </details>`;
  const get = selector => root.querySelector(selector);
  const label = preset => [hunting ? "Contested prey" : resting ? "Interrupted journey" : "Short finite journey", "Two-patch meadow"][preset];
  const runningLabel = () => label(running) + (resting && current?.rest ? `; minimum ${current.rest.policy[4]}, recovery ${current.rest.policy[5]}` : "") + (hunting && current?.hunt ? `; ${current.hunt.summary[0]} initial hunters` : "");
  const phase = value => ["Not executed", "Dark", "Lit"][Number(value)];
  const signed = value => BigInt.asIntN(64, value);
  const cell = (x, y) => `(${x}, ${y})`;
  const quantity = (n, noun, plural = noun + "s") => `${n} ${n === 1n ? noun : plural}`;
  let api, current, running = 0, selected = "", trajectory = [], comparisons = [];
  let totals = {added: 0n, eaten: 0n, upkeep: 0n, transferred: 0n, dissipated: 0n};
  let initialStores = 0n;
  function read(table, row, column) {
    if (api[`${prefix}valid`](table, row, column) !== 1) throw new Error("The spatial adapter returned an invalid address.");
    return BigInt.asUintN(64, api[`${prefix}read`](table, row, column));
  }
  function rows(table, count, columns) {
    if (count > 10000n) throw new Error("This inspector supports at most 10,000 rows per table.");
    return Array.from({length: Number(count)}, (_, r) => Array.from({length: columns}, (_, c) => read(table, r, c)));
  }
  function snapshot() {
    const s = rows(0, 1n, 36)[0], c = rows(1, 1n, 10)[0], m = rows(9, 1n, 17)[0];
    const state = {s, c, m, grazers: rows(2, s[3], 24), patches: rows(3, s[4], 4), resources: rows(4, s[5], 6),
      populationEvents: rows(5, s[6], 17), mutation: rows(6, c[7], 1).map(r => signed(r[0])),
      founders: rows(7, s[34], 7), initialPatches: rows(8, s[35], 4),
      actors: rows(10, s[3], 19), patchCells: rows(11, s[4], 3), observations: rows(12, m[16], 5),
      mobileEvents: rows(13, m[2], 12), initialCells: rows(14, s[34], 6)};
    if (resting) {
      const policy = rows(15, 1n, 7)[0], counters = rows(16, 1n, 15)[0];
      state.rest = {policy, counters, actors: rows(17, s[3], 8), events: rows(18, counters[10], 8)};
      if (state.grazers.some((g, i) => g[0] !== state.rest.actors[i][0])) throw new Error("The activity report disagrees about stable IDs.");
    }
    if (hunting) {
      const summary = rows(19, 1n, 13)[0];
      state.hunt = {summary, ledger: rows(20, 1n, 13)[0], totals: rows(21, 1n, 13)[0],
        actors: rows(22, summary[1], 35), observations: rows(23, summary[2], 5),
        initial: rows(24, summary[0], 11), events: rows(25, summary[3], 12)};
    }
    for (const [a, b] of [[state.grazers, state.actors], [state.patches, state.patchCells], [state.founders, state.initialCells]]) {
      if (a.some((row, i) => row[0] !== b[i][0])) throw new Error("Owned reports disagree about stable IDs.");
    }
    return state;
  }
  function table(selector, data) {
    get(selector).replaceChildren(...data.map(values => {
      const tr = document.createElement("tr");
      values.forEach(value => {const td = document.createElement("td"); td.textContent = String(value); tr.append(td);});
      return tr;
    }));
  }
  function observe() {
    current = snapshot();
    const {s, m} = current;
    const reserve = current.grazers.reduce((sum, g) => sum + g[1], 0n), biomass = current.patches.reduce((sum, p) => sum + p[1], 0n);
    if (s[1] === 0n) initialStores = hunting ? current.hunt.summary[11] : reserve + biomass;
    trajectory.push({tick: s[1], light: s[2], living: s[7], juvenile: s[8], reserve, biomass,
      added: s[15], upkeep: s[16], eaten: s[17], deaths: s[18], births: s[19], transfer: s[20], cost: s[21], cells: m[7], travel: m[8], fatigue: current.rest?.actors.reduce((sum, a) => sum + a[1], 0n), rests: current.rest?.counters[2], hunters: current.hunt?.summary[1], hunterReserve: current.hunt?.summary[12], hunting: current.hunt?.ledger.slice()});
    trajectory = trajectory.slice(-129);
    totals.added += s[15]; totals.upkeep += s[16]; totals.eaten += s[17]; totals.transferred += s[20]; totals.dissipated += s[21];
  }
  const grazerKey = id => hunting ? `g${id}` : String(id);
  const hunterKey = id => `h${id}`;
  // Normalize only coordinates for drawing. Biology and observations are native.
  const hunterMapActor = a => [a[0], a[7], a[8], a[9], 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, a[19], a[20], a[21]];
  function map(actor) {
    const {m, actors, patchCells} = current, svg = get("[data-map]"), ns = "http://www.w3.org/2000/svg";
    const width = Number(m[0]), height = Number(m[1]);
    if (width > 100 || height > 100 || width * height > 2500) throw new Error("The map is too large for this inspector; use the native snapshot.");
    const unit = Math.min(520 / width, 250 / height), left = 40, top = 30;
    const el = (tag, attrs, text) => {
      const element = document.createElementNS(ns, tag);
      Object.entries(attrs).forEach(([key, value]) => element.setAttribute(key, String(value)));
      if (text !== undefined) element.textContent = text;
      return element;
    };
    const elements = [];
    for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
      // This only shades the recorded observation's geometric footprint. All
      // actual observations and targets come from Rust's owned snapshot.
      const inSight = actor && actor[16] === 1n && Math.abs(x - Number(actor[17])) + Math.abs(y - Number(actor[18])) <= Number(actor[3]);
      elements.push(el("rect", {x: left + x * unit, y: top + y * unit, width: unit, height: unit, fill: inSight ? "#e8e0b8" : "#f3f5ec", stroke: "#939d89", "stroke-width": 0.7}));
      const patch = patchCells.find(p => p[1] === BigInt(x) && p[2] === BigInt(y));
      const residents = actors.filter(g => g[1] === BigInt(x) && g[2] === BigInt(y));
      if (patch) elements.push(el("rect", {x: left + x * unit + 3, y: top + y * unit + 3, width: unit * 0.3, height: unit * 0.3, fill: "#366139"}));
      if (residents.length) {
        const active = residents.some(g => grazerKey(g[0]) === selected);
        elements.push(el("circle", {cx: left + (x + 0.64) * unit, cy: top + (y + 0.65) * unit, r: unit * 0.25, fill: "#604328", stroke: active ? "#126c94" : "#ffffff", "stroke-width": active ? 4 : 1}));
        elements.push(el("text", {x: left + (x + 0.64) * unit, y: top + (y + 0.65) * unit, "text-anchor": "middle", "dominant-baseline": "central", fill: "white", "font-size": Math.max(10, unit * 0.2)}, residents.length));
      }
      const hunters = current.hunt?.actors.filter(h => h[7] === BigInt(x) && h[8] === BigInt(y)) ?? [];
      if (hunters.length) {
        const cx = left + (x + 0.27) * unit, cy = top + (y + 0.68) * unit, radius = unit * 0.23;
        const active = hunters.some(h => hunterKey(h[0]) === selected);
        elements.push(el("polygon", {points: `${cx},${cy-radius} ${cx+radius},${cy} ${cx},${cy+radius} ${cx-radius},${cy}`, fill: "#a43830", stroke: active ? "#126c94" : "white", "stroke-width": active ? 4 : 1}));
        elements.push(el("text", {x:cx,y:cy,"text-anchor":"middle","dominant-baseline":"central",fill:"white","font-size":Math.max(10,unit*0.2)},hunters.length));
      }
      if (actor?.[16] === 1n && actor[17] === BigInt(x) && actor[18] === BigInt(y)) {
        elements.push(el("text", {x: left + (x + 0.65) * unit, y: top + (y + 0.28) * unit, "text-anchor": "middle", fill: "#126c94", "font-weight": "bold", "font-size": 16}, "×"));
      }
    }
    for (let x = 0; x < width; x++) elements.push(el("text", {x: left + (x + 0.5) * unit, y: top - 10, "text-anchor": "middle", fill: "currentColor", "font-size": 12}, x));
    for (let y = 0; y < height; y++) elements.push(el("text", {x: left - 12, y: top + (y + 0.6) * unit, "text-anchor": "end", fill: "currentColor", "font-size": 12}, y));
    svg.setAttribute("viewBox", `0 0 600 ${top + height * unit + 20}`);
    svg.replaceChildren(...elements);
    get("[data-map-caption]").textContent = `Global inspector: x increases right, y increases down; ${width} × ${height} cells. Green squares are patches, including empty ones. Brown circles count living grazers at each cell.${hunting ? " Red diamonds count living hunters." : ""} A blue outline marks the selected animal. The × and yellow cells show the origin and range of its last observation, which occurred before travel. All positions and observations are also in the tables.`;
  }
  function local() {
    const hunter = current.hunt?.actors.find(h => hunterKey(h[0]) === selected);
    if (hunter) {
      const h = hunter;
      get("[data-local-caption]").textContent = "Prey in the hunter's last owned observation, before movement and captures";
      get("[data-local-head]").innerHTML = "<tr><th>Prey</th><th>Observed cell</th><th>Observed reserve</th><th>Current cell / reserve (inspector)</th></tr>";
      get("[data-local]").textContent = `Hunter ${h[0]} is now at ${cell(h[7],h[8])}. ${h[17] === 1n && h[19] === 1n ? `It observed from ${cell(h[20],h[21])} on tick ${h[18]}, with Manhattan radius ${h[9]}.` : "It has not observed yet."} ${h[12] === 1n ? `Its retained target is prey ${h[13]} at ${cell(h[14],h[15])}, observed on tick ${h[16]}.` : "It has no retained target."} Maximum travel: ${quantity(h[10],"cell")} per tick, costing ${quantity(h[11],"reserve unit")} per cell. ${h[31] === 1n ? `Last travel opportunity evaluated on tick ${h[32]} (possibly zero cells).` : "No travel opportunity yet."} ${h[25] === 1n ? `Resting with ${h[24]} fatigue points and ${quantity(h[26],"committed action")} owed.` : `Foraging with ${h[24]} fatigue points.`} ${h[33] === 1n ? `Last successful capture: tick ${h[34]}.` : "No successful capture yet."}`;
      table("[data-observations]", current.hunt.observations.filter(o=>o[0]===h[0]).map(o=>{
        const i=current.grazers.findIndex(g=>g[0]===o[1]);
        return [o[1],cell(o[2],o[3]),o[4],i<0 ? "No longer in living census" : `${cell(current.actors[i][1],current.actors[i][2])} / ${current.grazers[i][1]}`];
      }));
      map(hunterMapActor(h)); return;
    }
    get("[data-local-caption]").textContent = "Patches in the grazer's last owned observation, before travel and feeding";
    get("[data-local-head]").innerHTML = "<tr><th>Patch</th><th>Observed cell</th><th>Observed biomass</th><th>Current biomass (inspector)</th></tr>";
    const a = current.actors.find(g => grazerKey(g[0]) === selected);
    if (!a) {get("[data-local]").textContent = hunting ? "No living animal to inspect." : "No living grazer to inspect."; table("[data-observations]", []); map(null); return;}
    get("[data-local]").textContent = `Grazer ${a[0]} is now at ${cell(a[1], a[2])}. ${a[11] === 1n && a[16] === 1n ? `It observed from ${cell(a[17], a[18])} on tick ${a[12]}, with Manhattan radius ${a[3]}.` : "It has not observed yet."} ${a[6] === 1n ? `Its retained target is patch ${a[7]} at ${cell(a[8], a[9])}, observed on tick ${a[10]}.` : "It has no retained target."} Maximum travel: ${quantity(a[4], "cell")} per tick, at ${quantity(a[5], "reserve unit")} per cell. ${a[14] === 1n ? `Last travel opportunity evaluated on tick ${a[15]} (possibly zero cells).` : "No travel opportunity evaluated yet."}`;
    table("[data-observations]", current.observations.filter(o => o[0] === a[0]).map(o => [o[1], cell(o[2], o[3]), o[4], current.patches.find(p => p[0] === o[1])?.[1] ?? "Missing"]));
    if (resting) {
      const activity = current.rest.actors.find(r => r[0] === a[0]);
      get("[data-local]").textContent += activity[2] === 1n
        ? ` It is resting with ${activity[1]} fatigue points and ${quantity(activity[3], "committed action")} still owed. Its old recorded view does not grant permission to act.`
        : ` It is foraging with ${activity[1]} fatigue points.`;
    }
    map(a);
  }
  function events(selector, entries, describe) {
    get(selector).replaceChildren(...entries.map(e => {const li = document.createElement("li"); li.textContent = `Run ${e[0]}, tick ${e[1]}: ${describe(e)}`; return li;}));
  }
  function render() {
    const {s, c, m} = current, latest = trajectory.at(-1);
    const choices = [...current.actors.map(a=>[grazerKey(a[0]),`Grazer ${a[0]}`]), ...(current.hunt?.actors.map(a=>[hunterKey(a[0]),`Hunter ${a[0]}`]) ?? [])];
    if (!choices.some(a=>a[0]===selected)) selected = choices[0]?.[0] ?? "";
    get("#mobile-actor").replaceChildren(...choices.map(([key,label])=>{const o=document.createElement("option");o.value=key;o.textContent=label;return o;}));
    get("#mobile-actor").value = selected; get("#mobile-actor").disabled = !choices.length;
    get("[data-inputs]").textContent = runningLabel();
    get("[data-clock]").textContent = `${s[0]} / ${s[1]} · ${phase(s[2])}`;
    get("[data-census]").textContent = `${s[7]} living (${s[8]} juvenile, ${s[9]} adult): ${s[10]} founders + ${s[11]} births − ${s[12]} starvations${hunting ? ` − ${current.hunt.totals[5]} captures. Hunters: ${current.hunt.summary[1]} living = ${current.hunt.summary[0]} initial − ${current.hunt.totals[6]} starvations` : ""}. Grazer cap ${c[6]}; ${s[14]} otherwise eligible pairs blocked on ${s[13]} ticks.`;
    get("[data-flows]").textContent = `${s[15]} added, ${s[17]} eaten, ${s[16]} upkeep spent; ${quantity(m[7], "cell")} travelled for ${quantity(m[8], "reserve unit")}; ${s[19]} births transferred ${s[20]} and dissipated ${s[21]}; ${s[18]} starvations; ${m[9]} target changes.`;
    const hc = current.hunt?.totals;
    const expected = initialStores + totals.added - totals.upkeep - m[11] - totals.dissipated - (hc ? hc[0]+hc[2]+hc[3] : 0n), actual = latest.reserve + latest.biomass + (current.hunt?.summary[12] ?? 0n);
    get("[data-account]").textContent = `${initialStores} starting stores + ${totals.added} actual growth − ${totals.upkeep} upkeep − ${m[11]} travel − ${totals.dissipated} birth cost${hunting ? ` − ${hc[0]} hunter upkeep − ${hc[2]} hunter travel − ${hc[3]} attack cost` : ""} = ${expected}. Current grazer reserves ${latest.reserve} + biomass ${latest.biomass}${hunting ? ` + hunter reserves ${current.hunt.summary[12]}` : ""} = ${actual}; difference ${actual - expected}. Meals (${totals.eaten}) and birth contributions (${totals.transferred}) transfer units within those stores.${hunting ? ` Captures also transferred ${hc[4]} units between animal roles; transferred reserve is not new energy. Starting grazer / patch / hunter stores were ${current.hunt.summary[8]} / ${current.hunt.summary[9]} / ${current.hunt.summary[10]}.` : ""} Lifetime grazer travel: ${m[10]} cells; target changes: ${m[12]}.`;
    table("[data-grazers]", current.grazers.map((g, i) => {const a = current.actors[i]; return [`${g[0]} / ${g[7] === 1n ? "adult" : "juvenile"}`, cell(a[1], a[2]), `${g[1]} / ${g[2]}`, `${g[3]} / ${g[4]}`, a[6] === 1n ? `${a[7]} / ${a[10]}` : "None", g[5] === 1n ? g[6] : "None", `${g[8]} / ${g[9] === 1n ? g[10] : "founder"} / ${g[11]}`];}));
    const patches = data => data.map((p, i) => [p[0], cell(current.patchCells[i][1], current.patchCells[i][2]), `${p[1]} / ${p[2]}`, p[3]]);
    table("[data-patches]", patches(current.patches)); table("[data-initial-patches]", patches(current.initialPatches));
    table("[data-origins]", current.grazers.filter(g => g[13] === 1n).map(g => [g[0], `${g[14]} / ${g[15]} / ${g[16]}`, `${g[17]}, ${g[18]}`, `${g[19]} / ${g[20]}`, `${signed(g[21])} / ${signed(g[22])}`, g[23]]));
    get("[data-policy]").textContent = `${s[33]} lit ticks per ${s[32]}-tick cycle. Each parent contributes ${c[2]} and dissipates ${c[3]} units, retaining at least one. Children have capacity ${c[4]} and meal request ${c[5]}, act from the next tick and mature ${c[0]} ticks after birth. Parent cooldown ${c[1]} ticks; cap ${c[6]}. Mutation repeats [${current.mutation.join(", ")}].`;
    get("[data-space-policy]").textContent = `Grid ${m[0]} × ${m[1]}. Newborn sight radius ${m[13]}, maximum travel ${m[14]} cells per tick, cost ${m[15]} per cell. Resource / population / spatial journal limits: ${c[8]} / ${c[9]} / ${m[3]}. These inputs come from the loaded Rust program.`;
    table("[data-founders]", current.founders.map((g, i) => {const a = current.initialCells[i]; return [g[0], cell(a[1], a[2]), `${g[1]} / ${g[2]}`, `${g[3]} / ${g[4]}`, `${a[3]} / ${a[4]} / ${a[5]}`];}));
    table("[data-trajectory]", trajectory.map(t => [`${t.tick} / ${phase(t.light)}`, `${t.living} / ${t.juvenile}`, `${t.reserve} / ${t.biomass}`, `${t.added} / ${t.eaten} / ${t.upkeep}`, `${t.cells} / ${t.travel}`, `${t.births} / ${t.deaths}`, `${t.transfer} / ${t.cost}`, ...(resting ? [`${t.fatigue} / ${t.rests}`] : []), ...(hunting ? [`${t.hunters} / ${t.hunterReserve}`, `${t.hunting[5]} / ${t.hunting[6]}`, `${t.hunting[0]} / ${t.hunting[2]} / ${t.hunting[3]} / ${t.hunting[4]}`] : [])]));
    const range = (after, through) => through > after ? `ticks ${after + 1n}–${through} inclusive` : "no complete interval";
    const afters = [s[23], s[26], m[4]], throughs = [s[24], s[27], m[5]];
    if (resting) {afters.push(current.rest.counters[11]); throughs.push(current.rest.counters[12]);}
    if (hunting) {afters.push(current.hunt.summary[5]); throughs.push(current.hunt.summary[6]);}
    const after = afters.reduce((a, b) => a > b ? a : b), through = throughs.reduce((a, b) => a < b ? a : b);
    get("[data-coverage]").textContent = `Resources: ${s[5]} retained, ${s[25]} evicted; complete ${range(s[23], s[24])}. Population: ${s[6]} retained, ${s[28]} evicted; complete ${range(s[26], s[27])}. Space: ${m[2]} retained, ${m[6]} evicted; complete ${range(m[4], m[5])}. All ${hunting ? "five" : resting ? "four" : "three"} together cover ${range(after, through)}. Outside a journal’s complete interval, absence is unknown. Lifetime travel ${m[10]} cells remains known even when its early events are lost.`;
    if (resting) {
      const {policy: p, counters: r, actors} = current.rest;
      get("[data-rest-policy]").textContent = `Fatigue capacity ${p[0]} points; each accepted travel cell adds ${p[1]}. Enter rest at ${p[2]} or more. Commit to at least ${p[4]} actual rest actions, recover up to ${p[5]} per action, and wake at a later decision only with no commitment left and fatigue at most ${p[3]}. These are the running policy values, not the menu prepared for a later reset.`;
      get("[data-rest-totals]").textContent = `Latest tick: ${r[0]} effort points added, ${r[1]} recovered, ${quantity(r[2], "rest action")}, ${quantity(r[3], "entry", "entries")} and ${quantity(r[4], "wake")}. Since reset: ${r[5]} effort points, ${r[6]} recovered, ${quantity(r[7], "rest action")}, ${quantity(r[8], "entry", "entries")} and ${quantity(r[9], "wake")}. Fatigue of current living grazers: ${actors.reduce((sum, a) => sum + a[1], 0n)} points. Dead grazers leave this roster, so living fatigue is not simply lifetime effort minus recovery.`;
      table("[data-rest-actors]", actors.map(a => [a[0], a[1], a[2] === 1n ? "Resting" : "Foraging", a[2] === 1n ? a[3] : "Not resting", a[4] === 1n ? a[5] : "None", a[6] === 1n ? a[7] : "None"]));
      get("[data-rest-coverage]").textContent = `Rest journal: ${r[10]} retained of limit ${r[14]}, ${r[13]} evicted; complete ${range(r[11], r[12])}. A zero recovery amount can still be an executed committed rest action.`;
      events("[data-rest-events]", current.rest.events, e => [
        `Grazer ${e[3]} entered rest at fatigue ${e[4]}, committing to ${e[5]} actions.`,
        `Grazer ${e[3]} woke at fatigue ${e[4]}.`,
        `Grazer ${e[3]} completed ${e[4]} travel cells, adding ${e[5]} fatigue points.`,
        `Grazer ${e[3]} rested, recovering ${e[4]} points; fatigue ${e[5]}, remaining commitment ${e[6]}.`,
      ][Number(e[2])]);
    }
    if (hunting) {
      const h = current.hunt, c = h.totals, l = h.ledger;
      get("[data-hunt-totals]").textContent = `Latest tick: ${quantity(l[5], "capture")} transferring ${l[4]}, ${l[0]} upkeep, ${l[2]} travel and ${l[3]} attack units spent; ${l[6]} hunter starvations. Since reset: ${quantity(c[5], "capture")} transferred ${c[4]}; ${c[0]} upkeep, ${c[2]} travel and ${c[3]} attack units spent. Hunter travel: ${c[1]} cells; ${c[7]} target changes. Hunter fatigue: ${c[8]} effort added (travel and captures), ${c[9]} recovered; ${c[10]} rest actions, ${c[11]} entries, ${c[12]} wakes. Dead hunters leave the current fatigue roster.`;
      const optional = (a,p,v) => a[p]===1n ? a[v] : "None";
      table("[data-hunters]",h.actors.map(a=>[`${a[0]} / ${cell(a[7],a[8])}`,`${a[1]} / ${a[2]}`,`${a[3]} / ${a[4]} / ${a[5]}`,a[12]===1n ? `${a[13]} / ${a[16]}` : "None",`${a[6]} / ${optional(a,33,34)}`,`${a[24]} / ${a[25]===1n ? "Resting" : "Foraging"} / ${a[25]===1n ? a[26] : "Not resting"}`,`${optional(a,27,28)} / ${optional(a,29,30)} / ${optional(a,31,32)}`]));
      table("[data-initial-hunters]",h.initial.map(a=>[`${a[0]} / ${cell(a[6],a[7])}`,`${a[1]} / ${a[2]}`,`${a[3]} / ${a[4]} / ${a[5]}`,`${a[8]} / ${a[9]} / ${a[10]}`]));
      get("[data-hunt-inputs]").textContent = `${h.summary[0]} initial hunters; hunter journal limit ${h.summary[4]}. Starting stores: ${h.summary[8]} grazer reserves + ${h.summary[9]} biomass + ${h.summary[10]} hunter reserves = ${h.summary[11]}. Hunters use the same running rest policy, with attack effort added only for a successful capture.`;
      get("[data-hunt-coverage]").textContent = `Hunting journal: ${h.summary[3]} retained of limit ${h.summary[4]}, ${h.summary[7]} evicted; complete ${range(h.summary[5],h.summary[6])}. Lifetime captures and costs remain known outside this retained event interval.`;
      events("[data-hunt-events]",h.events,e=>[
        `Hunter ${e[3]} spent ${e[4]} on upkeep.`,
        `Hunter ${e[3]} changed prey target from ${e[4]===1n ? e[5] : "none"} to ${e[6]===1n ? e[7] : "none"}.`,
        `Hunter ${e[3]} travelled ${cell(e[4],e[5])} → ${cell(e[6],e[7])}: ${e[8]} cells, ${e[9]} reserve units and ${e[10]} fatigue effort.`,
        `Hunter ${e[3]} captured prey ${e[4]} at ${cell(e[5],e[6])}, transferring ${e[7]} reserve units, spending ${e[8]} attack units and adding ${e[9]} fatigue points.`,
        `Hunter ${e[3]} starved.`,
        `Hunter ${e[3]} entered rest at fatigue ${e[4]}, committing to ${e[5]} actions.`,
        `Hunter ${e[3]} woke at fatigue ${e[4]}.`,
        `Hunter ${e[3]} rested, recovering ${e[4]}; fatigue ${e[5]}, remaining commitment ${e[6]}.`
      ][Number(e[2])]);
    }
    events("[data-mobile-events]", current.mobileEvents, e => e[2] === 0n ? `Grazer ${e[3]} changed target from ${e[4] === 1n ? e[5] : "none"} to ${e[6] === 1n ? e[7] : "none"}.` : e[2] === 1n ? `Grazer ${e[3]} travelled ${cell(e[4], e[5])} → ${cell(e[6], e[7])}: ${quantity(e[8], "cell")}, ${quantity(e[9], "unit")} spent.` : `Child ${e[3]} was placed at ${cell(e[4], e[5])}.`);
    events("[data-population-events]", current.populationEvents, e => e[2] === 1n ? `Individual ${e[3]} matured; upkeep ${e[4]}.` : `Child ${e[3]} born to ${e[6]} and ${e[7]}; donor ${e[8]}, inherited ${e[9]}, proposed / applied change ${signed(e[10])} / ${signed(e[11])}, child trait ${e[4]}. Each parent contributed ${e[12]} and dissipated ${e[13]}; child reserve ${e[14]}, first eligible tick ${e[15]}, maturation ${e[16]}.`);
    events("[data-resource-events]", current.resources, e => [`Patch ${e[3]} gained ${e[5]}.`, `Grazer ${e[3]} spent ${e[5]} on upkeep.`, `Grazer ${e[3]} ate ${e[5]} from patch ${e[4]}.`, `Grazer ${e[3]} starved.`][Number(e[2])]);
    local();
  }
  function fail(error) {get("fieldset").disabled = true; get("[data-build]").textContent = "No usable checked spatial preview is currently available."; get("[data-status]").textContent = `${error.message} The guide and native project remain usable.`;}
  get("#mobile-actor").addEventListener("change", () => {try {selected = get("#mobile-actor").value; local();} catch (error) {fail(error);}});
  root.querySelectorAll("[data-step]").forEach(button => button.addEventListener("click", () => {
    try {const count = Number(button.dataset.step); for (let i = 0; i < count; i++) {api[`${prefix}step`](); observe();} render(); get("[data-status]").textContent = `Executed ${count} tick${count === 1 ? "" : "s"}; run ${current.s[0]} is at tick ${current.s[1]}.`;} catch (error) {fail(error);}
  }));
  get("[data-reset]").addEventListener("click", () => {
    try {const next = Number(get("#mobile-preset").value); if (api[`${prefix}reset`](next, ...(resting ? [Number(get("#resting-minimum").value), Number(get(hunting ? "#hunting-enabled" : "#resting-recovery").value)] : [])) !== 1) throw new Error("Rust rejected those inputs."); running = next; trajectory = []; selected = ""; totals = {added: 0n, eaten: 0n, upkeep: 0n, transferred: 0n, dissipated: 0n}; observe(); render(); get("[data-status]").textContent = `Started run ${current.s[0]} at tick zero.`;} catch (error) {fail(error);}
  });
  get("[data-save]").addEventListener("click", () => {
    const {s, m} = current, t = trajectory.at(-1);
    comparisons.push([`${s[0]} / ${runningLabel()} / ${s[1]}`, `${s[7]} / ${s[8]}`, `${s[11]} / ${s[12]}`, `${totals.added} / ${totals.upkeep} / ${m[11]} / ${totals.dissipated}`, `${t.reserve} / ${t.biomass}`, `${m[10]} / ${m[12]}`, ...(resting ? [`${current.rest.counters[7]} / ${current.rest.counters[8]} / ${current.rest.counters[9]}`] : []), ...(hunting ? [`${current.hunt.summary[1]} / ${current.hunt.summary[12]} / ${initialStores}`, `${current.hunt.totals[5]} / ${current.hunt.totals[6]}`, `${current.hunt.totals[0]} / ${current.hunt.totals[2]} / ${current.hunt.totals[3]} / ${current.hunt.totals[4]}`] : [])]);
    comparisons = comparisons.slice(-12); table("[data-comparisons]", comparisons); get("[data-status]").textContent = `Kept run ${s[0]} at tick ${s[1]}.`;
  });
  (async () => {
    try {
      const response = await fetch(`previews/${mode}/build.json`, {cache: "no-store"});
      if (!response.ok) throw new Error(`Build fieldwork_preview.py --${mode} with --project or --reference, then run build.py.`);
      const metadata = await response.json();
      if (metadata.api !== 1 || metadata.mode !== mode || !["learner", "reference"].includes(metadata.sourceKind) || ![metadata.buildHash, metadata.sourceHash, metadata.wasmHash].every(v => /^[a-f0-9]{64}$/.test(v)) || metadata.wasm !== `${mode}-${metadata.buildHash}.wasm`) throw new Error("Unrecognized spatial build metadata.");
      const responseWasm = await fetch(`previews/${mode}/${metadata.wasm}`, {cache: "no-store"});
      if (!responseWasm.ok) throw new Error("The compiled spatial binary could not be loaded.");
      const bytes = await responseWasm.arrayBuffer(), digest = await crypto.subtle.digest("SHA-256", bytes);
      if ([...new Uint8Array(digest)].map(b => b.toString(16).padStart(2, "0")).join("") !== metadata.wasmHash) throw new Error("The spatial binary differs from its checked build record.");
      api = (await WebAssembly.instantiate(bytes, {})).instance.exports;
      if (!["reset", "step", "read", "valid"].every(n => typeof api[`${prefix}${n}`] === "function")) throw new Error("The compiled program has a different spatial interface.");
      get("[data-build]").textContent = `${metadata.sourceKind === "learner" ? "Learner project" : "Course reference"} · saved source ${metadata.sourceHash.slice(0, 12)} · build ${metadata.buildHash.slice(0, 12)} · Rust ${metadata.rust}. After an edit, check and rebuild to observe it here.`;
      observe(); render(); get("fieldset").disabled = false; get("[data-status]").textContent = "Ready at tick zero. Predict the next position, activity and actual meal.";
    } catch (error) {fail(error);}
  })();
})();
