// Direct compiled-WASM checks, not browser, accessibility, or native-source QA.
// Run: node --test learning/scripts/test_hunting_wasm.cjs
// The digest binds executed bytes to their record. Source/lock/host freshness is
// checked separately by fieldwork_preview.current(metadata, "hunting").
const {test} = require("node:test");
const assert = require("node:assert/strict");
const {createHash} = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const directory = path.join(__dirname, "../work/previews/hunting");
const metadataPath = path.join(directory, "build.json");
const optional = {
  skip: !fs.existsSync(metadataPath) && "Build fieldwork_preview.py --hunting first",
};

async function artifact() {
  const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf8"));
  assert.equal(metadata.api, 1);
  assert.equal(metadata.mode, "hunting");
  assert.ok(["learner", "reference"].includes(metadata.sourceKind));
  for (const field of ["sourceHash", "buildHash", "wasmHash"]) {
    assert.match(metadata[field], /^[a-f0-9]{64}$/, field);
  }
  assert.equal(metadata.wasm, `hunting-${metadata.buildHash}.wasm`);
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
    assert.equal(typeof api[`moss_hunting_${name}`], "function");
  }
  return api;
}

function read(api, table, row, column) {
  assert.equal(api.moss_hunting_valid(table, row, column), 1,
    `available scalar ${table}/${row}/${column}`);
  return BigInt.asUintN(64, api.moss_hunting_read(table, row, column));
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
  const policy = row(api, 15, 0, 7);
  const rest = row(api, 16, 0, 15);
  const hunt = row(api, 19, 0, 13);
  return {
    summary, config, space, policy, rest, hunt,
    huntLedger: row(api, 20, 0, 13), huntTotals: row(api, 21, 0, 13),
    hunters: rows(api, 22, hunt[1], 35), prey: rows(api, 23, hunt[2], 5),
    initialHunters: rows(api, 24, hunt[0], 11), hunts: rows(api, 25, hunt[3], 12),
    resting: rows(api, 17, summary[3], 8),
    restEvents: rows(api, 18, rest[10], 8),
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
  return sum(state.individuals, 1) + sum(state.patches, 1) + sum(state.hunters, 1);
}

test("hunting artifact executes its named and hashed binary without JavaScript rule imports", optional, async () => {
  const {metadata, module} = await artifact();
  assert.equal(metadata.rust, "1.93.1");
  const exported = WebAssembly.Module.exports(module).map(value => value.name);
  for (const name of ["reset", "step", "read", "valid"]) assert.ok(exported.includes(`moss_hunting_${name}`));
});

test("initial hunter inputs, all-role stores and absence flags survive the scalar boundary", optional, async () => {
  const api = await host(), state = snapshot(api);
  assert.deepEqual(state.summary.slice(0, 3), [1n, 0n, 0n]);
  assert.deepEqual(state.hunt, [2n, 2n, 0n, 0n, 128n, 0n, 0n, 0n, 10n, 10n, 8n, 28n, 8n]);
  assert.deepEqual(state.initialHunters, [100n, 101n].map(id => [id, 4n, 8n, 1n, 1n, 2n, 0n, 0n, 1n, 0n, 1n]));
  assert.deepEqual(state.huntLedger, Array(13).fill(0n));
  assert.deepEqual(state.huntTotals, Array(13).fill(0n));
  assert.deepEqual(state.policy, [10n, 2n, 6n, 2n, 2n, 2n, 128n]);
  assert.equal(stores(state), state.hunt[11]);
  for (const hunter of state.hunters) {
    assert.deepEqual(hunter.slice(1, 12), [4n, 8n, 1n, 1n, 2n, 1n, 0n, 0n, 1n, 0n, 1n]);
    assert.ok(hunter.slice(12, 23).every(value => value === 0n));
    assert.equal(hunter[23], 1n, "fatigue exists at zero; this is not absent fatigue");
    assert.ok(hunter.slice(24).every(value => value === 0n));
  }
  for (const address of [[19, 1, 0], [19, 0, 13], [20, 0, 13], [21, 1, 0], [22, 2, 0], [22, 0, 35], [23, 0, 0], [24, 2, 0], [25, 0, 0], [26, 0, 0]]) {
    assert.equal(api.moss_hunting_valid(...address), 0, `invalid ${address}`);
    assert.equal(api.moss_hunting_read(...address), -1n);
  }
});

test("reads and invalid resets preserve all reports and future transitions; valid resets clear old causes", optional, async () => {
  const api = await host(), comparison = await host();
  for (let tick = 0; tick < 6; tick++) {
    api.moss_hunting_step(); comparison.moss_hunting_step();
    const before = snapshot(api);
    for (const args of [[2, 2, 1], [0, 1, 1], [0, 4, 1], [0, 2, 2], [0, 2, -1]]) {
      assert.equal(api.moss_hunting_reset(...args), 0);
      assert.deepEqual(snapshot(api), before);
    }
    assert.deepEqual(snapshot(api), before);
    assert.deepEqual(snapshot(api), snapshot(comparison));
  }
  const run = read(api, 0, 0, 0);
  assert.equal(api.moss_hunting_reset(1, 3, 0), 1);
  const reset = snapshot(api);
  assert.deepEqual(reset.summary.slice(0, 3), [run + 1n, 0n, 0n]);
  assert.deepEqual(reset.hunt, [0n, 0n, 0n, 0n, 128n, 0n, 0n, 0n, 72n, 24n, 0n, 96n, 0n]);
  assert.deepEqual(reset.huntLedger, Array(13).fill(0n));
  assert.deepEqual(reset.huntTotals, Array(13).fill(0n));
  assert.equal(reset.policy[4], 3n);
  assert.equal(reset.summary[11], 0n);
  assert.equal(reset.summary[12], 0n);
  assert.deepEqual(reset.hunters, []);
  assert.deepEqual(reset.hunts, []);
});

test("literal contention transfers prey once and leaves the losing hunter's reading visibly stale", optional, async () => {
  const api = await host();
  api.moss_hunting_step();
  const state = snapshot(api), run = state.summary[0];
  assert.deepEqual(state.individuals.map(g => [g[0], g[1]]), [[2n, 6n]]);
  assert.equal(state.patches[0][1], 8n);
  assert.deepEqual(state.hunters.map(h => [h[0], h[1], h[24], h[33], h[34]]), [
    [100n, 6n, 2n, 1n, 1n], [101n, 3n, 0n, 0n, 0n],
  ]);
  assert.deepEqual(state.prey, [[100n, 1n, 0n, 0n, 4n], [100n, 2n, 0n, 0n, 4n],
    [101n, 1n, 0n, 0n, 4n], [101n, 2n, 0n, 0n, 4n]]);
  assert.ok(state.hunters.every(h => h[12] === 1n && h[13] === 1n && h[16] === 1n));
  assert.deepEqual(state.huntLedger.slice(0, 8), [2n, 0n, 0n, 1n, 4n, 1n, 0n, 2n]);
  assert.deepEqual(state.hunts.filter(event => event[2] === 3n), [[run, 1n, 3n, 100n, 1n, 0n, 0n, 4n, 1n, 2n, 0n, 0n]]);
  assert.equal(state.resources.filter(event => event[3] === 1n && [2n, 3n].includes(event[2])).length, 0,
    "captured prey cannot eat and is not also labeled starved");
  assert.equal(state.summary[11], 0n, "capture happens before the formerly eligible parental transaction");
  assert.equal(state.summary[12], 0n);
  assert.equal(stores(state) + state.summary[16] + state.huntLedger[0] + state.huntLedger[3], 28n);
  assert.equal(state.hunters[1][1], 3n, "the losing contender pays upkeep, but no failed attack cost");

  api.moss_hunting_reset(0, 2, 0);
  api.moss_hunting_step();
  const control = snapshot(api);
  assert.equal(control.summary[11], 1n, "same grazer/patch inputs can fund a birth without hunters");
  assert.equal(control.huntTotals[5], 0n);
});

// Literal native pilot endpoints. The hunter toggle adds 32 initial reserve
// units as well as hunters, so the total initial store is explicitly compared.
const comparisons = [
  {minimum: 2, enabled: 0, living: 0n, births: 7n, starved: 11n, travel: 36n, reserve: 0n, biomass: 36n,
    flows: [98n, 108n, 14n, 36n, 0n, 0n, 0n], hunt: Array(13).fill(0n)},
  {minimum: 3, enabled: 0, living: 1n, births: 10n, starved: 13n, travel: 71n, reserve: 24n, biomass: 31n,
    flows: [308n, 258n, 20n, 71n, 0n, 0n, 0n], hunt: Array(13).fill(0n)},
  {minimum: 2, enabled: 1, living: 0n, births: 3n, starved: 4n, travel: 15n, reserve: 0n, biomass: 36n,
    flows: [60n, 53n, 6n, 15n, 62n, 13n, 3n], hunt: [62n, 13n, 13n, 3n, 46n, 3n, 2n, 14n, 32n, 26n, 13n, 6n, 6n]},
  {minimum: 3, enabled: 1, living: 0n, births: 3n, starved: 3n, travel: 14n, reserve: 0n, biomass: 36n,
    flows: [74n, 59n, 6n, 14n, 72n, 11n, 4n], hunt: [72n, 11n, 11n, 4n, 55n, 4n, 2n, 14n, 30n, 30n, 15n, 5n, 5n]},
];
for (const expected of comparisons) {
  test(`native120 pilot: minimum ${expected.minimum}, ${expected.enabled ? "two hunters" : "no hunters"}, actual energy and cause-specific census`, optional, async () => {
    const api = await host();
    api.moss_hunting_reset(1, expected.minimum, expected.enabled);
    let previous = snapshot(api), lostHunterFatigue = 0n;
    const initial = previous.hunt[11], totals = Array(13).fill(0n), flows = Array(7).fill(0n);
    assert.equal(initial, expected.enabled ? 128n : 96n);
    assert.equal(stores(previous), initial);
    for (let tick = 1n; tick <= 120n; tick++) {
      api.moss_hunting_step();
      const state = snapshot(api), s = state.summary, m = state.space, h = state.huntLedger;
      const flow = [s[15], s[16], s[21], m[8], h[0], h[2], h[3]];
      flow.forEach((value, c) => {flows[c] += value;});
      assert.equal(stores(state) + flow.slice(1).reduce((a, b) => a + b, 0n), stores(previous) + s[15],
        `all stores include hunter energy and all six sinks at tick ${tick}`);
      assert.equal(s[7] + s[12] + state.huntTotals[5], s[10] + s[11], "capture and starvation are separate grazer removals");
      assert.equal(state.hunt[1] + state.huntTotals[6], state.hunt[0], "hunters neither reproduce nor remain immortal");
      assert.equal(s[7], s[8] + s[9]);
      assert.ok(s[7] <= state.config[6]);
      h.forEach((value, c) => {totals[c] += value;});
      assert.deepEqual(state.huntTotals, totals);
      assert.equal(sum(state.hunters, 1), state.hunt[12]);
      const events = state.hunts.filter(event => event[1] === tick);
      assert.equal(sum(events.filter(event => event[2] === 0n), 4), h[0]);
      assert.equal(sum(events.filter(event => event[2] === 2n), 8), h[1]);
      assert.equal(sum(events.filter(event => event[2] === 2n), 9), h[2]);
      assert.equal(sum(events.filter(event => event[2] === 3n), 8), h[3]);
      assert.equal(sum(events.filter(event => event[2] === 3n), 7), h[4]);
      assert.equal(BigInt(events.filter(event => event[2] === 3n).length), h[5]);
      assert.equal(sum(events.filter(event => event[2] === 2n), 10) + sum(events.filter(event => event[2] === 3n), 9), h[8]);
      assert.equal(sum(events.filter(event => event[2] === 7n), 4), h[9]);
      for (const before of previous.hunters) {
        if (state.hunters.some(value => value[0] === before[0])) continue;
        const own = events.filter(event => event[3] === before[0]);
        lostHunterFatigue += before[24] + sum(own.filter(event => event[2] === 2n), 10)
          + sum(own.filter(event => event[2] === 3n), 9) - sum(own.filter(event => event[2] === 7n), 4);
      }
      assert.equal(sum(state.hunters, 24) + lostHunterFatigue, totals[8] - totals[9],
        "hunter fatigue is separate from energy and departs with terminal bodies");
      previous = state;
    }
    assert.deepEqual(flows, expected.flows);
    assert.deepEqual(totals, expected.hunt);
    assert.deepEqual([previous.summary[7], previous.summary[11], previous.summary[12], previous.space[10], sum(previous.individuals, 1), sum(previous.patches, 1)],
      [expected.living, expected.births, expected.starved, expected.travel, expected.reserve, expected.biomass]);
    assert.equal(initial + flows[0], stores(previous) + flows.slice(1).reduce((a, b) => a + b, 0n));
    assert.equal(previous.hunt[1], 0n);
  });
}

test("hunter observations retain pre-travel origin and owned eligible local prey, including stale captured IDs", optional, async () => {
  const api = await host();
  api.moss_hunting_reset(1, 3, 1);
  let previous = snapshot(api), moved = 0, stale = 0, observed = 0;
  const abs = n => n < 0n ? -n : n;
  const distance = (a, b) => abs(a[0] - b[0]) + abs(a[1] - b[1]);
  for (let tick = 1n; tick <= 120n; tick++) {
    api.moss_hunting_step();
    const state = snapshot(api);
    assert.equal(sum(state.hunters, 22), state.hunt[2]);
    assert.ok(state.hunters.every(h => !state.individuals.some(g => g[0] === h[0])));
    for (const hunter of state.hunters) {
      const before = previous.hunters.find(h => h[0] === hunter[0]);
      assert.ok(before);
      const sightings = state.prey.filter(p => p[0] === hunter[0]);
      if (hunter[25] === 1n) {
        assert.equal(hunter[12], 0n);
        assert.ok(hunter[18] < tick);
        assert.deepEqual(sightings, previous.prey.filter(p => p[0] === hunter[0]));
        continue;
      }
      observed++;
      assert.deepEqual(hunter.slice(17, 22), [1n, tick, 1n, before[7], before[8]]);
      if (hunter[7] !== before[7] || hunter[8] !== before[8]) moved++;
      const eligible = previous.individuals.flatMap(g => {
        const cell = previous.actors.find(a => a[0] === g[0]).slice(1, 3);
        const upkeep = sum(state.resources.filter(e => e[1] === tick && e[2] === 1n && e[3] === g[0]), 5);
        const reserve = g[1] - upkeep;
        return reserve > 0n && g[8] <= tick && distance(before.slice(7, 9), cell) <= hunter[9]
          ? [[hunter[0], g[0], ...cell, reserve]] : [];
      });
      assert.deepEqual(sightings, eligible, "perception is local, after upkeep and before either role moves");
      for (const sight of sightings) if (!state.individuals.some(g => g[0] === sight[1])) stale++;
      if (hunter[12] === 1n) {
        const target = sightings.find(p => p[1] === hunter[13]);
        assert.ok(target);
        assert.deepEqual(hunter.slice(14, 17), [target[2], target[3], tick]);
      }
    }
    previous = state;
  }
  assert.ok(observed > 0 && moved > 0 && stale > 0, "the declared pilot exercises all three observation distinctions");
});

test("captures and rest events preserve action eligibility and distinct terminal causes", optional, async () => {
  const api = await host();
  api.moss_hunting_reset(1, 3, 1);
  let captures = 0, rests = 0;
  const capturedIds = new Set();
  for (let tick = 1n; tick <= 120n; tick++) {
    api.moss_hunting_step();
    const state = snapshot(api), events = state.hunts.filter(e => e[1] === tick);
    for (const capture of events.filter(e => e[2] === 3n)) {
      captures++;
      const hunterId = capture[3], preyId = capture[4];
      assert.ok(!capturedIds.has(preyId)); capturedIds.add(preyId);
      assert.ok(!state.individuals.some(g => g[0] === preyId));
      assert.equal(state.resources.filter(e => e[1] === tick && e[3] === preyId && [2n, 3n].includes(e[2])).length, 0);
      assert.ok(state.population.filter(e => e[1] === tick && e[2] === 0n).every(e => e[6] !== preyId && e[7] !== preyId));
      const hunter = state.hunters.find(h => h[0] === hunterId);
      assert.ok(hunter);
      assert.deepEqual(hunter.slice(7, 9), capture.slice(5, 7), "successful capture uses current contact");
      assert.deepEqual(hunter.slice(33, 35), [1n, tick]);
      assert.equal(capture[8], hunter[4]);
      assert.equal(capture[9], hunter[5]);
      assert.equal(events.filter(e => e[2] === 3n && e[3] === hunterId).length, 1);
    }
    for (const rest of events.filter(e => e[2] === 7n)) {
      rests++;
      assert.equal(events.filter(e => [2n, 3n].includes(e[2]) && e[3] === rest[3]).length, 0);
      const hunter = state.hunters.find(h => h[0] === rest[3]);
      if (hunter) {
        assert.deepEqual(hunter.slice(24, 29), [rest[5], 1n, rest[6], 1n, tick]);
        assert.equal(hunter[12], 0n);
      }
    }
    assert.ok(state.resources.filter(e => e[2] === 3n).every(e => !capturedIds.has(e[3])));
  }
  assert.deepEqual([captures, rests], [4, 15]);
});

test("hunter history is complete for the declared pilots without implying completeness of other journals", optional, async () => {
  const api = await host();
  for (const minimum of [2, 3]) {
    api.moss_hunting_reset(1, minimum, 1);
    for (let tick = 0; tick < 120; tick++) api.moss_hunting_step();
    const state = snapshot(api), h = state.hunt;
    assert.deepEqual(h.slice(5, 8), [0n, 120n, 0n], "these finite pilots do not force hunting-history eviction");
    assert.ok(h[3] <= h[4]);
    assert.equal(BigInt(state.hunts.filter(e => e[2] === 3n).length), state.huntTotals[5]);
    assert.equal(BigInt(state.hunts.filter(e => e[2] === 4n).length), state.huntTotals[6]);
    assert.ok(state.hunts.every(e => e[0] === state.summary[0] && e[1] >= 1n && e[1] <= 120n));
    assert.equal(state.hunts.filter(e => e[1] > 80n).length, 0, "the fully covered final interval is known quiet");
    const lowers = [state.summary[23], state.summary[26], state.space[4], state.rest[11], h[5]];
    const allAfter = lowers.reduce((a, b) => a > b ? a : b);
    assert.ok(allAfter >= h[5], "shared coverage intersects all five journals instead of copying the hunting boundary");
  }
  api.moss_hunting_reset(1, 3, 0);
  for (let tick = 0; tick < 120; tick++) api.moss_hunting_step();
  const control = snapshot(api);
  assert.deepEqual(control.hunt.slice(5, 8), [0n, 120n, 0n]);
  assert.equal(control.hunt[3], 0n);
  assert.equal(control.huntTotals[5], 0n);
  assert.ok(control.summary[23] > 0n && control.space[4] > 0n && control.rest[11] > 0n,
    "known zero captures do not restore other journals' evicted early events");
});
