// Direct compiled-WASM runtime checks, not a browser or accessibility test.
// Run: node --test learning/scripts/test_population_wasm.cjs
// The SHA check binds these bytes to their metadata. Source/lock/host freshness
// remains fieldwork_preview.current(metadata, "population")'s responsibility.
const {test} = require("node:test");
const assert = require("node:assert/strict");
const {createHash} = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const directory = path.join(__dirname, "../work/previews/population");
const metadataPath = path.join(directory, "build.json");
const optional = {
  skip: !fs.existsSync(metadataPath) && "Build fieldwork_preview.py --population first",
};

async function artifact() {
  const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf8"));
  assert.equal(metadata.api, 1);
  assert.equal(metadata.mode, "population");
  assert.ok(["learner", "reference"].includes(metadata.sourceKind));
  for (const field of ["sourceHash", "buildHash", "wasmHash"]) {
    assert.match(metadata[field], /^[a-f0-9]{64}$/, field);
  }
  assert.equal(metadata.wasm, `population-${metadata.buildHash}.wasm`);
  const bytes = fs.readFileSync(path.join(directory, metadata.wasm));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), metadata.wasmHash,
    "execute only the binary named and hashed by this build record");
  const module = await WebAssembly.compile(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), [],
    "the compiled world receives no JavaScript rule or time implementation");
  return {metadata, module};
}

async function host() {
  const {module} = await artifact();
  // Every test receives its own authoritative world and thread-local host state.
  const api = (await WebAssembly.instantiate(module, {})).exports;
  for (const name of ["reset", "step", "read", "valid"]) {
    assert.equal(typeof api[`moss_population_${name}`], "function");
  }
  return api;
}

function read(api, table, row, column) {
  assert.equal(api.moss_population_valid(table, row, column), 1,
    `available scalar ${table}/${row}/${column}`);
  return BigInt.asUintN(64, api.moss_population_read(table, row, column));
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
  return {
    summary, config,
    individuals: rows(api, 2, summary[3], 24),
    patches: rows(api, 3, summary[4], 4),
    resources: rows(api, 4, summary[5], 6),
    events: rows(api, 5, summary[6], 17),
    mutation: rows(api, 6, config[7], 1).map(value => BigInt.asIntN(64, value[0])),
    founders: rows(api, 7, summary[34], 7),
    initialPatches: rows(api, 8, summary[35], 4),
  };
}
function stores(state) {
  return state.individuals.reduce((sum, value) => sum + value[1], 0n)
    + state.patches.reduce((sum, value) => sum + value[1], 0n);
}

test("population artifact has a verified digest and no JavaScript imports", optional, async () => {
  const {metadata, module} = await artifact();
  assert.equal(metadata.rust, "1.93.1");
  const exported = WebAssembly.Module.exports(module).map(value => value.name);
  for (const name of ["reset", "step", "read", "valid"]) {
    assert.ok(exported.includes(`moss_population_${name}`));
  }
});

test("scalar availability distinguishes a signed minus one from an invalid address", optional, async () => {
  const api = await host();
  assert.equal(api.moss_population_reset(0, 0, 1), 1);
  assert.deepEqual(snapshot(api).mutation, [-1n, 0n, 1n, 0n]);
  assert.equal(api.moss_population_valid(6, 0, 0), 1);
  assert.equal(api.moss_population_read(6, 0, 0), -1n);
  for (const address of [[0, 1, 0], [1, 1, 0], [2, 99, 0], [6, 0, 1], [99, 0, 0]]) {
    assert.equal(api.moss_population_valid(...address), 0, `invalid ${address}`);
    assert.equal(api.moss_population_read(...address), -1n,
      "the raw sentinel shares minus one's bits; availability carries meaning");
  }
  api.moss_population_step();
  const birth = snapshot(api).events.find(event => event[2] === 0n);
  assert.ok(birth);
  assert.equal(BigInt.asIntN(64, birth[10]), -1n, "proposed change is signed");
  assert.equal(BigInt.asIntN(64, birth[11]), 0n, "trait one clips the applied change");
  assert.equal(birth[4], 1n);
  assert.equal(api.moss_population_reset(0, 0, 0), 1);
  assert.deepEqual(snapshot(api).mutation, [0n], "the other preset restores its explicit cycle");
});

test("complete repeated observations execute no tick and preserve every projected table", optional, async () => {
  const api = await host();
  const initial = snapshot(api);
  assert.deepEqual(initial.summary.slice(0, 3), [1n, 0n, 0n]);
  assert.deepEqual(initial.individuals.map(value => [value[0], value[1]]), [[1n, 5n], [2n, 5n]]);
  for (let repeat = 0; repeat < 5; repeat++) assert.deepEqual(snapshot(api), initial);
  api.moss_population_step();
  const completed = snapshot(api);
  assert.equal(completed.summary[1], 1n);
  for (let repeat = 0; repeat < 5; repeat++) assert.deepEqual(snapshot(api), completed);
});

test("invalid reset preserves the current run while a selected reset starts at tick zero", optional, async () => {
  const api = await host();
  api.moss_population_step();
  api.moss_population_step();
  const before = snapshot(api);
  for (const inputs of [[2, 0, 0], [0, 4, 0], [0, 0, 2]]) {
    assert.equal(api.moss_population_reset(...inputs), 0);
    assert.deepEqual(snapshot(api), before, `rejected selector ${inputs} changes nothing`);
  }
  assert.equal(api.moss_population_reset(1, 3, 1), 1);
  const reset = snapshot(api);
  assert.deepEqual(reset.summary.slice(0, 3), [before.summary[0] + 1n, 0n, 0n]);
  assert.deepEqual(reset.summary.slice(11, 29), Array(18).fill(0n),
    "totals, latest flows and both journal coverage records begin empty");
  assert.deepEqual(reset.founders.map(value => value[3]), [1n, 3n]);
  assert.deepEqual(reset.summary.slice(32, 34), [4n, 2n]);
  assert.equal(reset.initialPatches[0][3], 12n);
  assert.deepEqual(reset.mutation, [-1n, 0n, 1n, 0n]);
  assert.deepEqual(snapshot(api), reset, "reset does not execute an environmental phase");
});

test("four compiled ticks preserve the literal birth, first action and maturity trace", optional, async () => {
  const api = await host();
  const expected = [
    [[1n, 3n], [2n, 3n], [101n, 4n]],
    [[1n, 4n], [2n, 4n], [101n, 5n]],
    [[1n, 5n], [2n, 5n], [101n, 6n]],
    [[1n, 3n], [2n, 3n], [101n, 7n], [102n, 4n]],
  ];
  for (let index = 0; index < expected.length; index++) {
    api.moss_population_step();
    const state = snapshot(api), tick = BigInt(index + 1);
    assert.equal(state.summary[1], tick);
    assert.deepEqual(state.individuals.map(value => [value[0], value[1]]), expected[index]);
    assert.equal(state.patches[0][1], 2n);
    const child = state.individuals.find(value => value[0] === 101n);
    assert.equal(child[7], tick < 3n ? 0n : 1n, "juvenile until tick three");
    assert.equal(child[8], 2n, "first upkeep/meal eligibility");
    assert.deepEqual(child.slice(9, 13), [1n, 3n, 3n, 1n],
      "present maturity deadline, maturity tick, reproductive deadline, trait");
    assert.deepEqual(child.slice(14, 21), [1n, 1n, 0n, 1n, 2n, 1n, 1n],
      "run, birth tick, ordinal, parents, donor and inherited trait");
    const childActions = state.resources.filter(event => event[1] === tick && event[3] === 101n);
    assert.deepEqual(childActions.map(event => [event[2], event[5]]),
      tick === 1n ? [] : [[1n, 1n], [2n, 2n]],
      "birth visibility does not grant same-tick actions");
    assert.equal(state.events.filter(event => event[2] === 1n && event[3] === 101n).length,
      tick < 3n ? 0 : 1, "maturation records exactly once");
  }
  const final = snapshot(api), second = final.individuals.find(value => value[0] === 102n);
  assert.deepEqual(second.slice(7, 13), [0n, 5n, 1n, 6n, 6n, 1n]);
  assert.deepEqual(second.slice(15, 20), [4n, 1n, 1n, 2n, 2n],
    "second birth at four, ordinal one, same parents, upper-ID donor");
  assert.equal(final.summary[11], 2n);
  assert.deepEqual(final.summary.slice(19, 22), [1n, 4n, 2n]);
});

// Literal native-example oracles, not values inferred from the WASM under test.
// Ordering: census [living, juvenile, adult]; coverage [exclusive lower, upper, lost].
const comparisons = [
  {name: "1/1 steady", environment: 0, cohort: 0, census: [6n, 1n, 5n],
    births: 15n, deaths: 11n, upkeep: 200n, transfer: 60n, cost: 30n,
    blockedPairs: 0n, blockedTicks: 0n, reserve: 20n, biomass: 0n,
    coverage: [28n, 40n, 275n], traits: [6n, 0n, 0n]},
  {name: "1/1 pulsed", environment: 1, cohort: 0, census: [5n, 1n, 4n],
    births: 15n, deaths: 12n, upkeep: 208n, transfer: 60n, cost: 30n,
    blockedPairs: 1n, blockedTicks: 1n, reserve: 12n, biomass: 0n,
    coverage: [26n, 40n, 233n], traits: [5n, 0n, 0n]},
  {name: "2/2 steady", environment: 0, cohort: 2, census: [3n, 0n, 3n],
    births: 1n, deaths: 0n, upkeep: 238n, transfer: 4n, cost: 2n,
    blockedPairs: 0n, blockedTicks: 0n, reserve: 8n, biomass: 2n,
    coverage: [22n, 40n, 150n], traits: [0n, 3n, 0n]},
  {name: "2/2 pulsed", environment: 1, cohort: 2, census: [3n, 0n, 3n],
    births: 1n, deaths: 0n, upkeep: 238n, transfer: 4n, cost: 2n,
    blockedPairs: 0n, blockedTicks: 0n, reserve: 8n, biomass: 2n,
    coverage: [21n, 40n, 130n], traits: [0n, 3n, 0n]},
];
for (const expected of comparisons) {
  test(`forty compiled ticks match native ${expected.name} flows, turnover and coverage`, optional, async () => {
    const api = await host();
    assert.equal(api.moss_population_reset(expected.environment, expected.cohort, 0), 1);
    let previous = snapshot(api);
    const run = previous.summary[0];
    const totals = {growth: 0n, upkeep: 0n, transfer: 0n, cost: 0n, births: 0n, deaths: 0n};
    for (let tick = 1n; tick <= 40n; tick++) {
      api.moss_population_step();
      const state = snapshot(api), s = state.summary;
      assert.deepEqual(s.slice(0, 2), [run, tick]);
      assert.equal(stores(state) + s[16] + s[21], stores(previous) + s[15],
        `actual stores balance at tick ${tick}`);
      assert.equal(s[7] + s[12], s[10] + s[11], `census balance at tick ${tick}`);
      assert.equal(s[7], s[8] + s[9]);
      assert.ok(s[7] <= state.config[6]);
      totals.growth += s[15]; totals.upkeep += s[16];
      totals.transfer += s[20]; totals.cost += s[21];
      totals.births += s[19]; totals.deaths += s[18];
      previous = state;
    }
    const state = previous, s = state.summary;
    assert.deepEqual(s.slice(7, 10), expected.census);
    assert.deepEqual(s.slice(10, 15), [2n, expected.births, expected.deaths,
      expected.blockedTicks, expected.blockedPairs]);
    assert.deepEqual(totals, {growth: 240n, upkeep: expected.upkeep,
      transfer: expected.transfer, cost: expected.cost, births: expected.births, deaths: expected.deaths});
    assert.equal(state.individuals.reduce((sum, value) => sum + value[1], 0n), expected.reserve);
    assert.equal(state.patches[0][1], expected.biomass);
    const traits = [1n, 2n, 3n].map(trait => BigInt(state.individuals.filter(value => value[12] === trait).length));
    assert.deepEqual(traits, expected.traits);
    assert.deepEqual(s.slice(23, 26), expected.coverage, "resource journal has lost early whole ticks");
    assert.deepEqual(s.slice(26, 29), [0n, 40n, 0n], "population journal retains the complete run");
    assert.equal(state.resources.length, 128);
    assert.equal(BigInt(state.events.filter(event => event[2] === 0n).length), expected.births);
    assert.ok(state.resources.every(event => event[0] === run));
    assert.ok(state.events.every(event => event[0] === run));
    assert.ok(s[23] >= 1n,
      "ticks 1–40 are not fully covered by resource history, even when lifetime deaths are a known zero");
  });
}
