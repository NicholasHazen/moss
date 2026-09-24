/* Deliberately small teaching models. These are not the Moss Rust runtime. */
(function (root) {
  "use strict";
  function tick(state) {
    return { ...state, tick: state.tick + 1, reserve: Math.max(0, state.reserve - state.cost) };
  }
  function selectedEntities(entities, requireCreature, optionalEnergy) {
    return entities.filter(entity => (!requireCreature || entity.creature) && (optionalEnergy || entity.energy !== null));
  }
  function ownershipTrace(mode) {
    const original = { reserve: 9 };
    const inspected = mode === "borrow" ? original : { ...original };
    inspected.reserve = 4;
    return { original: original.reserve, inspected: inspected.reserve };
  }
  function forecast(start, ticks, rate) {
    const max = 4294967295;
    if (![start, ticks, rate].every(value => Number.isInteger(value) && value >= 0 && value <= max)) return { error: "invalid" };
    const requested = BigInt(ticks) * BigInt(rate);
    if (requested > BigInt(max)) return { error: "overflow" };
    const paid = Math.min(start, Number(requested));
    return { requested: Number(requested), paid, remaining: start - paid };
  }
  function targetView(selected, rows) {
    if (selected === null) return { kind: "Unchosen" };
    const row = rows.find(item => item.id === selected);
    return row ? { kind: "Present", id: selected, biomass: row.biomass } : { kind: "Missing", id: selected };
  }
  function resolveMeals(biomass, consumers) {
    const valid = value => Number.isInteger(value) && value >= 0 && value <= 4294967295;
    if (!valid(biomass) || new Set(consumers.map(c => c.id)).size !== consumers.length ||
        consumers.some(c => ![c.id,c.reserve,c.capacity,c.request].every(valid) || c.reserve > c.capacity)) {
      throw new Error("Use unique IDs and valid whole-unit inputs with reserve at or below capacity.");
    }
    const rows = consumers.map(c => ({...c})).sort((a,b) => a.id-b.id);
    const outcomes = [];
    for (const row of rows) {
      const taken = Math.min(row.request, biomass, row.capacity-row.reserve);
      biomass -= taken; row.reserve += taken;
      outcomes.push({id:row.id,taken});
    }
    return {biomass,consumers:rows,outcomes};
  }
  function growthTick(state, order) {
    if (!["growth-first","meal-first"].includes(order)) throw new Error("Choose an explicit order");
    const next = {...state,tick:state.tick+1,added:0,eaten:0};
    next.lit = (next.tick-1)%4 < 2;
    const grow = () => {next.added = next.lit ? Math.min(3,4-next.biomass) : 0;next.biomass += next.added;};
    const eat = () => {next.eaten = Math.min(2,next.biomass,20-next.reserve);next.biomass-=next.eaten;next.reserve+=next.eaten;};
    if (order === "growth-first") {grow();eat();} else {eat();grow();}
    return next;
  }
  const models = { tick, selectedEntities, ownershipTrace, forecast, targetView, resolveMeals, growthTick };
  if (typeof module !== "undefined" && module.exports) module.exports = models;
  if (!root.document) return;
  root.MossModels = models;
  const html = (container, markup) => { container.innerHTML = markup; };
  for (const panel of root.document.querySelectorAll("[data-lab]")) {
    const kind = panel.dataset.lab;
    if (kind === "observe") {
      html(panel, `<p class="eyebrow">Experiment 01 · Render or execute?</p><h2>What spends a reserve?</h2><p>Predict the reserve after three Steps. Then change the camera scale. Which operation changes the world?</p><div class="tick-world" aria-hidden="true"><div class="world-actor">Fern<br><span data-energy>60</span></div><div class="world-patch">Meadow</div></div><label for="tick-cost">Maintenance per tick: <output id="cost-value">1</output></label><input id="tick-cost" type="range" min="0" max="10" value="1"><label for="camera-scale">Camera scale</label><input id="camera-scale" type="range" min="70" max="150" value="100"><div class="button-row"><button type="button" data-action="step">Execute one tick</button><button type="button" data-action="reset">Reset experiment</button></div><p class="readout" role="status" aria-live="polite"></p><p class="model-caption">JavaScript teaching model: a fixed creature, bounded reserve, and one upkeep rule. The Rust reference below checks the rule separately. Camera scale changes only the drawing.</p>`);
      let state = { tick: 0, reserve: 60, cost: 1 };
      const render = () => {
        panel.querySelector(".readout").textContent = `Executed ticks: ${state.tick}. Fern’s reserve: ${state.reserve}. Next tick costs ${state.cost}.`;
        panel.querySelector("[data-energy]").textContent = state.reserve;
      };
      panel.querySelector('[data-action="step"]').addEventListener("click", () => { state = tick(state); render(); });
      panel.querySelector('[data-action="reset"]').addEventListener("click", () => { state = { tick: 0, reserve: 60, cost: 1 }; panel.querySelector("#tick-cost").value = 1; panel.querySelector("#cost-value").textContent = 1; panel.querySelector("#camera-scale").value = 100; panel.querySelector(".world-actor").style.transform = ""; render(); });
      panel.querySelector("#tick-cost").addEventListener("input", event => { state.cost = Number(event.target.value); panel.querySelector("#cost-value").textContent = state.cost; render(); });
      panel.querySelector("#camera-scale").addEventListener("input", event => { panel.querySelector(".world-actor").style.transform = `scale(${Number(event.target.value) / 100})`; });
      render();
    } else if (kind === "ownership") {
      html(panel, `<p class="eyebrow">Experiment 02 · Follow the write</p><h2>Same number, different owner</h2><p>An energy record starts at 9. The inspector sets its working value to 4. Predict the original reserve for each access mode.</p><label for="access-mode">Inspector access</label><select id="access-mode"><option value="copy">Copy the value</option><option value="borrow">Borrow it mutably</option></select><p>What is the original reserve after the write?</p><div class="prediction"><button type="button" data-answer="9">9 units</button><button type="button" data-answer="4">4 units</button></div><p class="readout" role="status">Choose an answer to reveal the trace.</p><p class="model-caption">This diagram illustrates the two successful cases. Only Rust’s compiler can check whether a particular borrow or move is legal. The lesson below also follows a moved String.</p>`);
      const mode = panel.querySelector("#access-mode");
      const reset = () => { panel.querySelector(".readout").textContent = "Choose an answer to reveal the trace."; };
      mode.addEventListener("change", reset);
      panel.querySelectorAll("[data-answer]").forEach(button => button.addEventListener("click", () => {
        const result = ownershipTrace(mode.value);
        const correct = Number(button.dataset.answer) === result.original;
        panel.querySelector(".readout").textContent = `${correct ? "Yes." : "Follow the destination of the write."} Original: ${result.original}; inspector’s working value: ${result.inspected}. ${mode.value === "copy" ? "The new value has its own storage; writing it leaves the original at 9." : "The mutable reference reaches the original storage, so the original becomes 4."}`;
      }));
    } else if (kind === "units") {
      html(panel, `<p class="eyebrow">Experiment 04 · Keep the accounts separate</p><h2>Requested is not always paid</h2><p>Start with four reserve units, three ticks, and a rate of two. Then change the reserve to sixty. Which two fields become equal?</p><label for="forecast-reserve">Starting reserve units</label><input id="forecast-reserve" type="number" min="0" max="4294967295" step="1" value="4"><label for="forecast-ticks">Number of ticks</label><input id="forecast-ticks" type="number" min="0" max="4294967295" step="1" value="3"><label for="forecast-rate">Units per tick</label><input id="forecast-rate" type="number" min="0" max="4294967295" step="1" value="2"><p class="readout" role="status"></p><p class="model-caption">JavaScript arithmetic model of the same constant-rate forecast. Inputs are u32-sized nonnegative integers. No live reserve changes, and no Rust source is compiled here.</p>`);
      const render = () => {
        const result = forecast(...["forecast-reserve", "forecast-ticks", "forecast-rate"].map(id => panel.querySelector(`#${id}`).valueAsNumber));
        panel.querySelector(".readout").textContent = result.error === "invalid" ? "Use whole numbers from 0 through 4,294,967,295 for all three inputs." : result.error === "overflow" ? "No exact u32 forecast: ticks × rate exceeds 4,294,967,295. This is rejection, not a zero payment." : `Requested: ${result.requested} units. Actually paid: ${result.paid} units. Remaining: ${result.remaining} units.`;
      };
      panel.querySelectorAll("input").forEach(input => input.addEventListener("input", render)); render();
    } else if (kind === "options") {
      html(panel, `<p class="eyebrow">Experiment 05 · Preserve the evidence</p><h2>What can the inspector say?</h2><p>Patch 3 is present even when it has zero biomass. Select ID 99 to inspect an unresolved reference, then remove the selection entirely.</p><label for="selected-patch">Selected identity</label><select id="selected-patch"><option value="3">Patch 3</option><option value="99">Patch 99</option><option value="none">No selection</option></select><label for="patch-biomass">Biomass reported for patch 3</label><input id="patch-biomass" type="range" min="0" max="20" value="0"><p class="readout" role="status"></p><p class="model-caption">JavaScript view model for a fixed snapshot. Missing means absent from these readings; it does not prove a patch died or choose a replacement target.</p>`);
      const render = () => {
        const value = panel.querySelector("#selected-patch").value;
        const result = targetView(value === "none" ? null : Number(value), [{ id: 3, biomass: Number(panel.querySelector("#patch-biomass").value) }]);
        panel.querySelector(".readout").textContent = result.kind === "Unchosen" ? "Unchosen: no target identity was supplied." : result.kind === "Missing" ? `Missing: ID ${result.id} has no reading in this snapshot.` : `Present: ID ${result.id}, biomass ${result.biomass} units. A zero amount is still a present reading.`;
      };
      panel.querySelectorAll("input,select").forEach(input => input.addEventListener("input", render)); render();
    } else if (kind === "growth") {
      html(panel, `<p class="eyebrow">Experiment 18 · Same systems, different opportunity</p><h2>Which phase gets the space?</h2><p>Run both orders from an empty patch, then start from a full one. Compare actual supply as well as the meal. Ticks 1 and 2 are lit; 3 and 4 are dark; the cycle repeats.</p><label for="growth-initial">Starting biomass: <output id="growth-initial-value">0</output> of 4</label><input id="growth-initial" type="range" min="0" max="4" value="0"><div class="button-row"><button type="button" data-growth-step>Run next tick in both orders</button><button type="button" data-growth-reset>Reset both runs</button></div><p class="readout" role="status"></p><table><caption>Latest outcomes from both runs</caption><thead><tr><th scope="col">Order</th><th scope="col">Added</th><th scope="col">Eaten</th><th scope="col">Biomass</th><th scope="col">Reserve</th></tr></thead><tbody></tbody></table><p class="model-caption">JavaScript model of the proposed lab: patch capacity 4, growth request 3, meal request 2, reserve capacity 20, no maintenance or death. The paired Rust schedules below are checked separately. Changing starting biomass resets both runs; this model stops at tick 16.</p>`);
      let before,after;
      const render = () => {
        panel.querySelector(".readout").textContent = before.tick ? `Completed tick ${before.tick}, ${before.lit ? "lit" : "dark"}. Growth-first reserve ${before.reserve}; meal-first reserve ${after.reserve}.` : "No ticks executed yet. Both runs have the same starting values.";
        const body=panel.querySelector("tbody");body.replaceChildren();
        for (const [name,state] of [["Growth → meal",before],["Meal → growth",after]]) {
          const row=document.createElement("tr");
          [name,state.added,state.eaten,state.biomass,state.reserve].forEach((text,index)=>{const cell=document.createElement(index ? "td" : "th");if(!index)cell.scope="row";cell.textContent=text;row.append(cell);});body.append(row);
        }
        panel.querySelector("[data-growth-step]").disabled=before.tick>=16;
      };
      const reset = () => {const biomass=Number(panel.querySelector("#growth-initial").value);panel.querySelector("#growth-initial-value").textContent=biomass;before={tick:0,biomass,reserve:0,added:0,eaten:0};after={...before};render();};
      panel.querySelector("[data-growth-step]").addEventListener("click",()=>{before=growthTick(before,"growth-first");after=growthTick(after,"meal-first");render();});
      panel.querySelector("[data-growth-reset]").addEventListener("click",reset);
      panel.querySelector("#growth-initial").addEventListener("input",reset);reset();
    } else if (kind === "meals") {
      html(panel, `<p class="eyebrow">Experiment 15 · The second request sees the first payment</p><h2>Five units, two requests</h2><p>Both animals request three units. Predict the outcome when ID 2 resolves before ID 9. Then lower ID 2’s capacity to two and try again.</p><label for="meal-biomass">Starting biomass: <output id="meal-start-value">5</output></label><input id="meal-biomass" type="range" min="0" max="10" value="5"><label for="meal-capacity">ID 2 reserve capacity: <output id="meal-cap-value">10</output></label><input id="meal-capacity" type="range" min="0" max="10" value="10"><div class="button-row"><button type="button" data-meal-resolve>Resolve both requests</button><button type="button" data-meal-reset>Reset meal experiment</button></div><p class="readout" role="status"></p><p class="model-caption">JavaScript accounting model of the proposed isolated rule: one biomass unit yields one reserve unit; all consumers have valid contact; ascending unique IDs decide resolution order. Changing an input starts a fresh experiment. This is neither Rust execution nor a calibrated biological conversion.</p>`);
      let state;
      const render = () => {
        const rows = state.consumers.map(c => `ID ${c.id}: reserve ${c.reserve}/${c.capacity}`).join("; ");
        const outcomes = state.outcomes.length ? ` Last resolution: ${state.outcomes.map(o => `ID ${o.id} received ${o.taken}`).join("; ")}.` : " No requests resolved yet.";
        panel.querySelector(".readout").textContent = `Patch: ${state.biomass} biomass. ${rows}.${outcomes}`;
      };
      const reset = () => {
        const biomass = Number(panel.querySelector("#meal-biomass").value);
        const capacity = Number(panel.querySelector("#meal-capacity").value);
        panel.querySelector("#meal-start-value").textContent = biomass;
        panel.querySelector("#meal-cap-value").textContent = capacity;
        state = {biomass, consumers:[{id:9,reserve:0,capacity:10,request:3},{id:2,reserve:0,capacity,request:3}],outcomes:[]};
        render();
      };
      panel.querySelector("[data-meal-resolve]").addEventListener("click", () => {state = resolveMeals(state.biomass,state.consumers); render();});
      panel.querySelector("[data-meal-reset]").addEventListener("click",reset);
      panel.querySelectorAll("input").forEach(input=>input.addEventListener("input",reset));
      reset();
    } else if (kind === "queries") {
      const entities = [{name:"Fern",creature:true,energy:60},{name:"Meadow",creature:false,energy:null},{name:"Flint",creature:true,energy:45},{name:"Marker",creature:false,energy:0},{name:"Unfinished creature",creature:true,energy:null}];
      html(panel, `<p class="eyebrow">Experiment 03 · A signature is a selection</p><h2>Who can this system see?</h2><p>Fern and Flint have Creature and Energy. Meadow has neither. Marker has Energy(0), while an unfinished creature has Creature but no Energy.</p><label for="query-filter">Filter</label><select id="query-filter"><option value="creature">With&lt;Creature&gt;</option><option value="all">No marker filter</option></select><label for="query-access">Energy access</label><select id="query-access"><option value="required">&amp;Energy (required)</option><option value="optional">Option&lt;&amp;Energy&gt;</option></select><pre><code data-signature></code></pre><p class="readout" role="status"></p><p class="model-caption">A membership model, not an ECS implementation. It shows matching only; it does not simulate Bevy’s borrowing checks or scheduler.</p>`);
      const render = () => {
        const creature = panel.querySelector("#query-filter").value === "creature";
        const optional = panel.querySelector("#query-access").value === "optional";
        panel.querySelector("[data-signature]").textContent = `Query<${optional ? "Option<&Energy>" : "&Energy"}${creature ? ", With<Creature>" : ""}>`;
        const found = selectedEntities(entities, creature, optional);
        panel.querySelector(".readout").textContent = `Matches ${found.length} entities: ${found.map(e => `${e.name} (${e.energy === null ? "None" : e.energy})`).join(", ")}. Zero energy is a value; a missing component is absence.`;
      };
      panel.querySelectorAll("select").forEach(select => select.addEventListener("change", render));
      render();
    }
  }
})(typeof window !== "undefined" ? window : globalThis);
