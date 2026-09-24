// Direct compiled-WASM checks, not browser, accessibility, or native-source QA.
// Run: node --test learning/scripts/test_mobile_wasm.cjs
// The digest binds executed bytes to their record. Source/lock/host freshness is
// checked separately by fieldwork_preview.current(metadata, "mobile").
const {test} = require("node:test");
const assert = require("node:assert/strict");
const {createHash} = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const directory = path.join(__dirname, "../work/previews/mobile");
const metadataPath = path.join(directory, "build.json");
const optional = {
  skip: !fs.existsSync(metadataPath) && "Build fieldwork_preview.py --mobile first",
};

async function artifact() {
  const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf8"));
  assert.equal(metadata.api, 1);
  assert.equal(metadata.mode, "mobile");
  assert.ok(["learner", "reference"].includes(metadata.sourceKind));
  for (const field of ["sourceHash", "buildHash", "wasmHash"]) {
    assert.match(metadata[field], /^[a-f0-9]{64}$/, field);
  }
  assert.equal(metadata.wasm, `mobile-${metadata.buildHash}.wasm`);
  const bytes = fs.readFileSync(path.join(directory, metadata.wasm));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), metadata.wasmHash,
    "execute only the binary named and hashed by this build record");
  const module = await WebAssembly.compile(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), [],
    "the compiled world imports no JavaScript rules, time, or random source");
  return {metadata, module};
}

async function host() {
  const {module} = await artifact();
  const api = (await WebAssembly.instantiate(module, {})).exports;
  for (const name of ["reset", "step", "read", "valid"]) {
    assert.equal(typeof api[`moss_mobile_${name}`], "function");
  }
  return api;
}

function read(api, table, row, column) {
  assert.equal(api.moss_mobile_valid(table, row, column), 1,
    `available scalar ${table}/${row}/${column}`);
  return BigInt.asUintN(64, api.moss_mobile_read(table, row, column));
}
function row(api, table, index, columns) {
  return Array.from({length: columns}, (_, column) => read(api, table, index, column));
}
function rows(api, table, count, columns) {
  assert.ok(count <= 10000n, "bounded adapter fixture");
  return Array.from({length: Number(count)}, (_, index) => row(api, table, index, columns));
}
function snapshot(api) {
  const summary = row(api, 0, 0, 36);
  const config = row(api, 1, 0, 10);
  const space = row(api, 9, 0, 17);
  return {
    summary, config, space,
    individuals: rows(api, 2, summary[3], 24),
    patches: rows(api, 3, summary[4], 4),
    resources: rows(api, 4, summary[5], 6),
    population: rows(api, 5, summary[6], 17),
    mutation: rows(api, 6, config[7], 1).map(value => BigInt.asIntN(64, value[0])),
    founders: rows(api, 7, summary[34], 7),
    initialPatches: rows(api, 8, summary[35], 4),
    actors: rows(api, 10, summary[3], 19),
    patchCells: rows(api, 11, summary[4], 3),
    observations: rows(api, 12, space[16], 5),
    journeys: rows(api, 13, space[2], 12),
    initialActors: rows(api, 14, summary[34], 6),
  };
}
function sum(values, column) {
  return values.reduce((total, value) => total + value[column], 0n);
}
function stores(state) {
  return sum(state.individuals, 1) + sum(state.patches, 1);
}
function distance(from, to) {
  const abs = value => value < 0n ? -value : value;
  return abs(from[0] - to[0]) + abs(from[1] - to[1]);
}

test("mobile artifact has a verified digest, expected exports, and no JavaScript imports", optional, async () => {
  const {metadata, module} = await artifact();
  assert.equal(metadata.rust, "1.93.1");
  const exported = WebAssembly.Module.exports(module).map(value => value.name);
  for (const name of ["reset", "step", "read", "valid"]) {
    assert.ok(exported.includes(`moss_mobile_${name}`));
  }
});

test("all scalar tables distinguish a valid zero from an unavailable address", optional, async () => {
  const api = await host();
  assert.equal(read(api, 0, 0, 1), 0n);
  assert.equal(read(api, 10, 0, 1), 0n);
  assert.equal(read(api, 10, 0, 6), 0n, "no target is an available optional-value flag");
  assert.equal(read(api, 10, 0, 11), 0n, "no observation is an available optional-value flag");
  assert.deepEqual(snapshot(api).mutation, [0n],
    "these fixtures use zero mutation; negative mutation has a separate population ABI test");
  const shapes = [[1, 36], [1, 10], [1, 24], [1, 4], [0, 6], [0, 17], [1, 1],
    [1, 7], [1, 4], [1, 17], [1, 19], [1, 3], [0, 5], [0, 12], [1, 6]];
  for (const [table, [count, columns]] of shapes.entries()) {
    for (const address of [[table, count, 0], [table, 0, columns]]) {
      assert.equal(api.moss_mobile_valid(...address), 0, `invalid ${address}`);
      assert.equal(api.moss_mobile_read(...address), -1n,
        "the sentinel alone does not describe availability or signedness");
    }
  }
  for (const address of [[15, 0, 0], [99, 0, 0], [10, -1, 0], [9, 0, -1]]) {
    assert.equal(api.moss_mobile_valid(...address), 0);
    assert.equal(api.moss_mobile_read(...address), -1n);
  }
  assert.equal(read(api, 0, 0, 1), 0n, "neither valid nor invalid reads execute a tick");
});

test("complete repeated observations and invalid resets preserve current and future state", optional, async () => {
  const api = await host(), comparison = await host();
  const initial = snapshot(api);
  assert.deepEqual(initial.summary.slice(0, 3), [1n, 0n, 0n]);
  assert.deepEqual(initial.actors[0].slice(0, 6), [1n, 0n, 0n, 3n, 1n, 1n]);
  for (let repeat = 0; repeat < 4; repeat++) assert.deepEqual(snapshot(api), initial);
  for (let tick = 1; tick <= 3; tick++) {
    api.moss_mobile_step(); comparison.moss_mobile_step();
    const before = snapshot(api);
    for (const selector of [2, 99, -1]) {
      assert.equal(api.moss_mobile_reset(selector), 0);
      assert.deepEqual(snapshot(api), before);
    }
    assert.deepEqual(snapshot(api), snapshot(comparison), "rejected reset also preserves the next transition");
  }
  const completed = snapshot(api);
  for (let repeat = 0; repeat < 4; repeat++) assert.deepEqual(snapshot(api), completed);
});

test("reset installs declared initial positions and clears every old journal and counter", optional, async () => {
  const api = await host();
  for (let tick = 0; tick < 3; tick++) api.moss_mobile_step();
  const previousRun = read(api, 0, 0, 0);
  assert.equal(api.moss_mobile_reset(1), 1);
  const state = snapshot(api);
  assert.deepEqual(state.summary.slice(0, 3), [previousRun + 1n, 0n, 0n]);
  assert.deepEqual(state.summary.slice(11, 29), Array(18).fill(0n));
  assert.deepEqual(state.space.slice(4, 13), Array(9).fill(0n));
  assert.deepEqual(state.space.slice(0, 2), [11n, 5n]);
  assert.deepEqual(state.summary.slice(32, 36), [8n, 4n, 4n, 2n]);
  assert.deepEqual(state.initialActors, [
    [1n, 2n, 2n, 5n, 1n, 1n], [2n, 3n, 1n, 5n, 1n, 1n],
    [3n, 2n, 3n, 5n, 1n, 1n], [4n, 3n, 2n, 5n, 1n, 1n],
  ]);
  assert.deepEqual(state.actors.map(value => value.slice(0, 6)), state.initialActors);
  assert.deepEqual(state.patchCells, [[1000n, 3n, 2n], [1001n, 7n, 2n]]);
  assert.deepEqual(state.initialPatches, [[1000n, 6n, 18n, 8n], [1001n, 18n, 18n, 4n]]);
  assert.deepEqual(state.patches, state.initialPatches);
  assert.ok(state.actors.every(value => value.slice(6).every(field => field === 0n)));
  assert.equal(stores(state), 96n);
  assert.deepEqual(snapshot(api), state, "reset performs no upkeep, growth, observation, or travel");
});

test("three compiled ticks match the native short journey and retain pre-travel observations", optional, async () => {
  const api = await host();
  // Literal oracle from the native example, not inferred by running this WASM.
  const expected = [
    {cell: [1n, 0n], origin: [0n, 0n], reserve: 3n, biomass: 3n, seen: 3n, travel: 1n, meal: 0n},
    {cell: [2n, 0n], origin: [1n, 0n], reserve: 3n, biomass: 1n, seen: 3n, travel: 1n, meal: 2n},
    {cell: [2n, 0n], origin: [2n, 0n], reserve: 3n, biomass: 0n, seen: 1n, travel: 0n, meal: 1n},
  ];
  for (const [index, wanted] of expected.entries()) {
    api.moss_mobile_step();
    const state = snapshot(api), tick = BigInt(index + 1), actor = state.actors[0];
    assert.equal(state.summary[1], tick);
    assert.deepEqual(actor.slice(1, 3), wanted.cell);
    assert.deepEqual(actor.slice(16, 19), [1n, ...wanted.origin]);
    assert.deepEqual(actor.slice(6, 13), [1n, 100n, 2n, 0n, tick, 1n, tick]);
    assert.equal(state.individuals[0][1], wanted.reserve);
    assert.equal(state.patches[0][1], wanted.biomass);
    assert.deepEqual(state.observations, [[1n, 100n, 2n, 0n, wanted.seen]]);
    assert.deepEqual(state.space.slice(7, 9), [wanted.travel, wanted.travel]);
    assert.equal(state.summary[17], wanted.meal);
    assert.equal(state.individuals[0][5], tick === 1n ? 0n : 1n,
      "seeing and selecting food does not confer contact before arrival");
  }
  const final = snapshot(api);
  assert.deepEqual(final.space.slice(10, 13), [2n, 2n, 1n]);
  assert.deepEqual(final.journeys.filter(event => event[2] === 1n).map(event => event.slice(1, 10)), [
    [1n, 1n, 1n, 0n, 0n, 1n, 0n, 1n, 1n],
    [2n, 1n, 1n, 1n, 0n, 2n, 0n, 1n, 1n],
  ]);
});

test("source IDs join all projections and local observations precede travel and competing meals", optional, async () => {
  const api = await host();
  api.moss_mobile_reset(1);
  let previous = snapshot(api);
  let movedWithObservation = 0, observedFoodThenConsumed = 0, absentDistantPatch = 0;
  for (let count = 0; count < 120; count++) {
    api.moss_mobile_step();
    const state = snapshot(api), tick = state.summary[1];
    assert.deepEqual(state.actors.map(value => value[0]), state.individuals.map(value => value[0]));
    assert.deepEqual(state.patchCells.map(value => value[0]), state.patches.map(value => value[0]));
    assert.equal(sum(state.actors, 13), state.space[16]);
    assert.ok(state.observations.every(value => state.actors.some(actor => actor[0] === value[0])));
    for (const actor of state.actors) {
      assert.ok(actor[1] < state.space[0] && actor[2] < state.space[1]);
      const before = previous.actors.find(value => value[0] === actor[0]);
      const sightings = state.observations.filter(value => value[0] === actor[0]);
      if (!before) continue; // Newborn assertions have their own check below.
      assert.deepEqual(actor.slice(11, 13), [1n, tick]);
      assert.deepEqual(actor.slice(16, 19), [1n, before[1], before[2]],
        "observation origin is the pre-travel position, not the current map marker");
      if (actor[1] !== before[1] || actor[2] !== before[2]) movedWithObservation++;
      const localPatches = state.patchCells.filter(patch => distance(before.slice(1, 3), patch.slice(1, 3)) <= actor[3]);
      assert.deepEqual(sightings.map(value => value[1]), localPatches.map(value => value[0]),
        "local perception lists every in-range patch, including currently empty ones");
      absentDistantPatch += state.patchCells.length - localPatches.length;
      for (const sighting of sightings) {
        const position = state.patchCells.find(value => value[0] === sighting[1]);
        assert.deepEqual(sighting.slice(2, 4), position.slice(1, 3));
        const initialBiomass = previous.patches.find(value => value[0] === sighting[1])[1];
        const growth = sum(state.resources.filter(event => event[1] === tick && event[2] === 0n && event[3] === sighting[1]), 5);
        assert.equal(sighting[4], initialBiomass + growth,
          "all observers read after actual accepted growth and before any competing meal");
        if (sighting[4] > state.patches.find(value => value[0] === sighting[1])[1]) observedFoodThenConsumed++;
      }
      if (actor[6] === 1n) {
        const target = sightings.find(value => value[1] === actor[7]);
        assert.ok(target && target[4] > 0n, "target refers to a positive owned local observation");
        assert.deepEqual(actor.slice(8, 11), [target[2], target[3], tick]);
      }
      const individual = state.individuals.find(value => value[0] === actor[0]);
      if (individual[5] === 1n) {
        const contact = state.patchCells.find(value => value[0] === individual[6]);
        assert.deepEqual(actor.slice(1, 3), contact.slice(1, 3));
        assert.equal(actor[7], individual[6]);
      }
    }
    previous = state;
  }
  assert.ok(movedWithObservation > 0 && observedFoodThenConsumed > 0 && absentDistantPatch > 0,
    "fixture actually exercises movement, stale food readings, and a limited local view");
});

test("spatial births publish placement and parentage without granting a premature observation or action", optional, async () => {
  const api = await host();
  api.moss_mobile_reset(1);
  let birthsChecked = 0;
  for (let count = 0; count < 120; count++) {
    api.moss_mobile_step();
    const state = snapshot(api), tick = state.summary[1];
    const born = state.population.filter(event => event[1] === tick && event[2] === 0n);
    assert.equal(BigInt(born.length), state.summary[19]);
    for (const birth of born) {
      birthsChecked++;
      const actor = state.actors.find(value => value[0] === birth[3]);
      const child = state.individuals.find(value => value[0] === birth[3]);
      const placement = state.journeys.find(event => event[1] === tick && event[2] === 2n && event[3] === birth[3]);
      assert.ok(actor && child && placement);
      assert.deepEqual(actor.slice(1, 3), placement.slice(4, 6));
      assert.deepEqual(actor.slice(3, 6), state.space.slice(13, 16));
      assert.ok(actor.slice(6).every(field => field === 0n),
        "newborn has no target, observation, visible readings, travel tick, or observed origin");
      assert.equal(child[1], birth[14]);
      assert.deepEqual(child.slice(5, 9), [0n, 0n, 0n, tick + 1n]);
      assert.deepEqual(child.slice(14, 18), [state.summary[0], tick, birth[5], birth[6]]);
      assert.equal(child[18], birth[7]);
      for (const parentId of birth.slice(6, 8)) {
        const parent = state.actors.find(value => value[0] === parentId);
        assert.ok(parent);
        assert.deepEqual(parent.slice(1, 3), actor.slice(1, 3), "both parents meet at the birth cell");
      }
      assert.equal(state.observations.filter(value => value[0] === birth[3]).length, 0);
      assert.equal(state.resources.filter(event => event[1] === tick && event[3] === birth[3]).length, 0);
      assert.equal(state.journeys.filter(event => event[1] === tick && event[3] === birth[3] && event[2] !== 2n).length, 0);
    }
  }
  assert.equal(birthsChecked, 32);
});

test("120 compiled ticks preserve literal native checkpoints and every actual energy and census balance", optional, async () => {
  const api = await host();
  api.moss_mobile_reset(1);
  let previous = snapshot(api);
  const run = previous.summary[0];
  const totals = {growth: 0n, upkeep: 0n, transfer: 0n, cost: 0n, travel: 0n, cells: 0n, targets: 0n};
  // [living, juvenile, births, starvations, travel cells, reserve sum, biomass]
  // Native examples/mobile.rs, recorded in the checkpoint README.
  const expected = new Map([
    [1n, [5n, 1n, 1n, 0n, 3n, 75n, 18n]],
    [2n, [6n, 2n, 2n, 0n, 4n, 73n, 18n]],
    [4n, [6n, 1n, 3n, 1n, 4n, 69n, 18n]],
    [8n, [4n, 1n, 4n, 4n, 21n, 39n, 3n]],
    [12n, [3n, 1n, 5n, 6n, 21n, 38n, 18n]],
    [40n, [3n, 1n, 12n, 13n, 63n, 37n, 6n]],
    [80n, [3n, 1n, 22n, 23n, 116n, 34n, 8n]],
    [120n, [4n, 1n, 32n, 32n, 173n, 42n, 4n]],
  ]);
  for (let tick = 1n; tick <= 120n; tick++) {
    api.moss_mobile_step();
    const state = snapshot(api), s = state.summary, m = state.space;
    assert.deepEqual(s.slice(0, 2), [run, tick]);
    assert.equal(stores(state) + s[16] + m[8] + s[21], stores(previous) + s[15],
      `actual stores include paid travel and birth dissipation at tick ${tick}`);
    assert.equal(s[7] + s[12], s[10] + s[11]);
    assert.equal(s[7], s[8] + s[9]);
    assert.ok(s[7] <= state.config[6]);
    totals.growth += s[15]; totals.upkeep += s[16]; totals.transfer += s[20]; totals.cost += s[21];
    totals.cells += m[7]; totals.travel += m[8]; totals.targets += m[9];
    assert.deepEqual(m.slice(10, 13), [totals.cells, totals.travel, totals.targets],
      "lifetime counters equal actual ledgers, even after early journey events are evicted");
    if (expected.has(tick)) {
      assert.deepEqual([s[7], s[8], s[11], s[12], m[10], sum(state.individuals, 1), sum(state.patches, 1)], expected.get(tick));
    }
    previous = state;
  }
  assert.deepEqual(totals, {growth: 606n, upkeep: 419n, transfer: 128n, cost: 64n, travel: 173n, cells: 173n, targets: 89n});
  assert.deepEqual(previous.summary.slice(13, 15), [0n, 0n]);
  assert.deepEqual(previous.actors.map(value => value.slice(0, 3)), [
    [1n, 7n, 2n], [2n, 7n, 2n], [1031n, 7n, 2n], [1033n, 7n, 2n],
  ]);
  assert.equal(96n + totals.growth, stores(previous) + totals.upkeep + totals.travel + totals.cost);
});

test("bounded journals disclose unknown full-range travel while cumulative travel remains known", optional, async () => {
  const api = await host();
  api.moss_mobile_reset(1);
  for (let tick = 0; tick < 120; tick++) api.moss_mobile_step();
  const state = snapshot(api), s = state.summary, m = state.space;
  assert.deepEqual(s.slice(23, 26), [99n, 120n, 592n]);
  assert.deepEqual(s.slice(26, 29), [0n, 120n, 0n]);
  assert.deepEqual(m.slice(4, 7), [69n, 120n, 166n]);
  assert.equal(state.resources.length, 128);
  assert.equal(state.journeys.length, 128);
  for (const events of [state.resources, state.population, state.journeys]) {
    assert.ok(events.every(event => event[0] === s[0] && event[1] >= 1n && event[1] <= 120n));
    assert.ok(events.every((event, index) => !index || events[index - 1][1] <= event[1]));
  }
  assert.equal(BigInt(state.population.filter(event => event[2] === 0n).length), s[11],
    "population journal still covers the full run and contains every accepted birth");
  assert.ok(1n <= m[4], "requesting ticks 1–120 would cross evicted spatial history");
  const retainedTravel = sum(state.journeys.filter(event => event[2] === 1n), 8);
  assert.ok(retainedTravel < m[10], "summing retained records cannot replace the lifetime counter");
  assert.equal(m[10], 173n);
  assert.equal([s[23], s[26], m[4]].reduce((a, b) => a > b ? a : b), 99n,
    "only ticks 100–120 are fully covered by all three journals together");
});
