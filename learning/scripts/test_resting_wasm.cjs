// Direct compiled-WASM checks, not browser, accessibility, or native-source QA.
// Run: node --test learning/scripts/test_resting_wasm.cjs
// The digest binds executed bytes to their record. Source/lock/host freshness is
// checked separately by fieldwork_preview.current(metadata, "resting").
const {test} = require("node:test");
const assert = require("node:assert/strict");
const {createHash} = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const directory = path.join(__dirname, "../work/previews/resting");
const metadataPath = path.join(directory, "build.json");
const optional = {
  skip: !fs.existsSync(metadataPath) && "Build fieldwork_preview.py --resting first",
};

async function artifact() {
  const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf8"));
  assert.equal(metadata.api, 1);
  assert.equal(metadata.mode, "resting");
  assert.ok(["learner", "reference"].includes(metadata.sourceKind));
  for (const field of ["sourceHash", "buildHash", "wasmHash"]) {
    assert.match(metadata[field], /^[a-f0-9]{64}$/, field);
  }
  assert.equal(metadata.wasm, `resting-${metadata.buildHash}.wasm`);
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
    assert.equal(typeof api[`moss_resting_${name}`], "function");
  }
  return api;
}

function read(api, table, row, column) {
  assert.equal(api.moss_resting_valid(table, row, column), 1,
    `available scalar ${table}/${row}/${column}`);
  return BigInt.asUintN(64, api.moss_resting_read(table, row, column));
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
  return {
    summary, config, space, policy, rest,
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
  return sum(state.individuals, 1) + sum(state.patches, 1);
}
test("resting artifact binds the executed binary to its digest and imports no JavaScript rules", optional, async () => {
  const {metadata, module} = await artifact();
  assert.equal(metadata.rust, "1.93.1");
  const exported = WebAssembly.Module.exports(module).map(value => value.name);
  for (const name of ["reset", "step", "read", "valid"]) {
    assert.ok(exported.includes(`moss_resting_${name}`));
  }
});

test("initial and reset projections expose actual policy, zero fatigue, and explicit scalar availability", optional, async () => {
  const api = await host();
  let state = snapshot(api);
  assert.deepEqual(state.policy, [10n, 2n, 6n, 2n, 2n, 2n, 128n]);
  assert.deepEqual(state.rest.slice(0, 14), Array(14).fill(0n));
  assert.equal(state.rest[14], 128n);
  assert.deepEqual(state.resting, [[1n, 0n, 0n, 0n, 0n, 0n, 0n, 0n]]);
  assert.equal(stores(state), 40n);
  assert.deepEqual(state.space.slice(0, 2), [6n, 2n]);
  assert.deepEqual(state.initialActors, [[1n, 0n, 0n, 5n, 1n, 1n]]);
  assert.deepEqual(state.patchCells, [[100n, 5n, 0n]]);
  for (const address of [[15, 1, 0], [15, 0, 7], [16, 1, 0], [16, 0, 15], [17, 1, 0], [17, 0, 8], [18, 0, 0], [19, 0, 0]]) {
    assert.equal(api.moss_resting_valid(...address), 0);
    assert.equal(api.moss_resting_read(...address), -1n);
  }
  for (let count = 0; count < 5; count++) api.moss_resting_step();
  const run = read(api, 0, 0, 0);
  assert.equal(api.moss_resting_reset(1, 3, 6), 1);
  state = snapshot(api);
  assert.deepEqual(state.summary.slice(0, 3), [run + 1n, 0n, 0n]);
  assert.deepEqual(state.policy, [10n, 2n, 6n, 2n, 3n, 6n, 128n]);
  assert.deepEqual(state.rest.slice(0, 14), Array(14).fill(0n));
  assert.deepEqual(state.summary.slice(11, 29), Array(18).fill(0n));
  assert.deepEqual(state.space.slice(4, 13), Array(9).fill(0n));
  assert.ok(state.resting.every(value => value.slice(1).every(field => field === 0n)));
  assert.equal(stores(state), 96n);
});

test("repeated complete observations and invalid selectors preserve commitment and the next transition", optional, async () => {
  const api = await host(), comparison = await host();
  for (let tick = 1; tick <= 7; tick++) {
    api.moss_resting_step(); comparison.moss_resting_step();
    const state = snapshot(api);
    for (const inputs of [[2, 2, 2], [0, 1, 2], [0, 4, 2], [0, 2, 0], [0, 2, 3], [-1, 2, 2]]) {
      assert.equal(api.moss_resting_reset(...inputs), 0);
      assert.deepEqual(snapshot(api), state);
    }
    for (let repeat = 0; repeat < 3; repeat++) assert.deepEqual(snapshot(api), state);
    assert.deepEqual(state, snapshot(comparison), "invalid resets and reads preserve future state too");
  }
});

test("ten compiled journey ticks retain the native distinction between fatigue, commitment and reserve", optional, async () => {
  const api = await host();
  // [x, reserve, fatigue, activity, remaining commitment, observed tick]
  const native = [
    [1n, 18n, 2n, 0n, 0n, 1n], [2n, 16n, 4n, 0n, 0n, 2n],
    [3n, 14n, 6n, 0n, 0n, 3n], [3n, 13n, 4n, 1n, 1n, 3n],
    [3n, 12n, 2n, 1n, 0n, 3n], [4n, 10n, 4n, 0n, 0n, 6n],
    [5n, 10n, 6n, 0n, 0n, 7n], [5n, 9n, 4n, 1n, 1n, 7n],
    [5n, 8n, 2n, 1n, 0n, 7n], [5n, 9n, 2n, 0n, 0n, 10n],
  ];
  let previous = snapshot(api);
  for (const [index, expected] of native.entries()) {
    api.moss_resting_step();
    const state = snapshot(api), tick = BigInt(index + 1), r = state.resting[0], a = state.actors[0];
    assert.equal(state.summary[1], tick);
    assert.deepEqual([a[1], state.individuals[0][1], r[1], r[2], r[3], a[12]], expected);
    assert.equal(stores(state) + state.summary[16] + state.space[8] + state.summary[21], stores(previous) + state.summary[15]);
    if (r[2] === 1n) {
      assert.equal(state.summary[16], 1n, "upkeep still spends reserve while resting");
      assert.equal(state.summary[17], 0n, "fatigue recovery creates neither a meal nor energy");
      assert.equal(state.space[7], 0n);
      assert.equal(a[6], 0n);
      assert.equal(state.individuals[0][5], 0n);
      assert.deepEqual(r.slice(4, 6), [1n, tick]);
      assert.equal(state.rest[2], 1n);
      assert.deepEqual(state.observations, previous.observations, "the old local view is retained with its old tick");
    }
    previous = state;
  }
  assert.deepEqual(previous.rest.slice(5, 10), [10n, 8n, 4n, 2n, 2n]);
  assert.deepEqual(previous.restEvents.filter(event => event[2] === 0n).map(event => event.slice(1, 7)), [
    [4n, 0n, 1n, 6n, 2n, 0n], [8n, 0n, 1n, 6n, 2n, 0n],
  ]);
  assert.deepEqual(previous.restEvents.filter(event => event[2] === 1n).map(event => event.slice(1, 5)), [
    [6n, 1n, 1n, 2n], [10n, 1n, 1n, 2n],
  ]);
});

test("fast recovery still executes a zero-recovery action, while an extra commitment delays waking", optional, async () => {
  const fast = await host(), longer = await host();
  fast.moss_resting_reset(0, 2, 6);
  longer.moss_resting_reset(0, 3, 2);
  for (let tick = 1; tick <= 5; tick++) {fast.moss_resting_step(); longer.moss_resting_step();}
  const fifth = snapshot(fast);
  assert.deepEqual(fifth.resting[0].slice(1, 6), [0n, 1n, 0n, 1n, 5n]);
  assert.deepEqual(fifth.rest.slice(0, 3), [0n, 0n, 1n]);
  assert.deepEqual(fifth.restEvents.filter(event => event[1] === 5n), [[fifth.summary[0], 5n, 3n, 1n, 0n, 0n, 0n, 0n]]);
  fast.moss_resting_step(); longer.moss_resting_step();
  assert.deepEqual(snapshot(fast).resting[0].slice(1, 4), [2n, 0n, 0n]);
  assert.equal(read(fast, 10, 0, 1), 4n);
  const sixth = snapshot(longer);
  assert.deepEqual(sixth.resting[0].slice(1, 4), [0n, 1n, 0n]);
  assert.equal(sixth.actors[0][1], 3n);
  assert.equal(sixth.individuals[0][1], 11n);
  assert.equal(sixth.actors[0][12], 3n);
  longer.moss_resting_step();
  assert.equal(read(longer, 17, 0, 2), 0n);
  assert.equal(read(longer, 10, 0, 12), 7n);
});

// Literal native examples/resting.rs measurements, all other inputs equal.
const comparisons = [
  {minimum: 2, endpoints: [[8, 3, 3, 4, 15, 7, 31, 18], [12, 3, 4, 5, 19, 12, 28, 34],
    [40, 0, 7, 11, 36, 21, 0, 36], [120, 0, 7, 11, 36, 21, 0, 36]],
    flows: [98n, 108n, 14n, 36n], counters: [72n, 42n, 21n, 11n, 9n], ids: []},
  {minimum: 3, endpoints: [[8, 4, 3, 3, 14, 8, 32, 18], [12, 3, 4, 5, 18, 11, 37, 22],
    [40, 2, 7, 9, 46, 33, 26, 18], [120, 1, 10, 13, 71, 53, 24, 31]],
    flows: [308n, 258n, 20n, 71n], counters: [142n, 106n, 53n, 18n, 17n], ids: [1n]},
];
for (const expected of comparisons) {
  test(`minimum ${expected.minimum} preserves native endpoints and separate energy/fatigue accounting for 120 ticks`, optional, async () => {
    const api = await host();
    api.moss_resting_reset(1, expected.minimum, 2);
    let previous = snapshot(api), lostFatigue = 0n;
    const counters = Array(5).fill(0n), flows = Array(4).fill(0n);
    const checkpoints = new Map(expected.endpoints.map(values => [BigInt(values[0]), values.slice(1).map(BigInt)]));
    for (let tick = 1n; tick <= 120n; tick++) {
      api.moss_resting_step();
      const state = snapshot(api), s = state.summary, m = state.space, r = state.rest;
      assert.equal(stores(state) + s[16] + m[8] + s[21], stores(previous) + s[15], `energy tick ${tick}`);
      assert.equal(s[7] + s[12], s[10] + s[11]);
      assert.equal(s[7], s[8] + s[9]);
      assert.ok(s[7] <= state.config[6]);
      for (let c = 0; c < 5; c++) counters[c] += r[c];
      assert.deepEqual(r.slice(5, 10), counters);
      [s[15], s[16], s[21], m[8]].forEach((value, c) => {flows[c] += value;});
      const events = state.restEvents.filter(event => event[1] === tick);
      assert.equal(sum(events.filter(event => event[2] === 2n), 5), r[0]);
      assert.equal(sum(events.filter(event => event[2] === 3n), 4), r[1]);
      assert.equal(BigInt(events.filter(event => event[2] === 3n).length), r[2]);
      assert.equal(r[0], m[7] * state.policy[1], "only completed travel earns fatigue");
      for (const old of previous.resting) {
        if (state.resting.some(value => value[0] === old[0])) continue;
        const own = events.filter(event => event[3] === old[0]);
        lostFatigue += old[1] + sum(own.filter(event => event[2] === 2n), 5) - sum(own.filter(event => event[2] === 3n), 4);
      }
      assert.equal(sum(state.resting, 1) + lostFatigue, counters[0] - counters[1],
        "departed animals remove their fatigue from the living census, not by recovery or energy transfer");
      assert.ok(state.resting.every(value => value[1] <= state.policy[0]));
      if (checkpoints.has(tick)) {
        assert.deepEqual([s[7], s[11], s[12], m[10], r[7], sum(state.individuals, 1), sum(state.patches, 1)], checkpoints.get(tick));
      }
      previous = state;
    }
    assert.deepEqual(flows, expected.flows);
    assert.deepEqual(counters, expected.counters);
    assert.deepEqual(previous.individuals.map(value => value[0]), expected.ids);
    assert.equal(96n + flows[0], stores(previous) + flows[1] + flows[2] + flows[3]);
  });
}

test("rested actions exclude travel, meals and parenting; newborns start without rest state history", optional, async () => {
  const api = await host();
  api.moss_resting_reset(1, 3, 2);
  let checkedRest = 0, checkedBirth = 0, checkedWake = 0;
  for (let tick = 1n; tick <= 120n; tick++) {
    api.moss_resting_step();
    const state = snapshot(api);
    assert.deepEqual(state.resting.map(value => value[0]), state.actors.map(value => value[0]));
    assert.deepEqual(state.resting.map(value => value[0]), state.individuals.map(value => value[0]));
    const events = state.restEvents.filter(event => event[1] === tick);
    const births = state.population.filter(event => event[1] === tick && event[2] === 0n);
    for (const action of events.filter(event => event[2] === 3n)) {
      checkedRest++;
      const id = action[3];
      assert.equal(state.journeys.filter(event => event[1] === tick && event[2] === 1n && event[3] === id).length, 0);
      assert.equal(state.resources.filter(event => event[1] === tick && event[2] === 2n && event[3] === id).length, 0);
      assert.ok(births.every(event => event[6] !== id && event[7] !== id));
      const actor = state.actors.find(value => value[0] === id);
      if (actor) {
        assert.equal(actor[6], 0n);
        assert.ok(actor[12] < tick, "rest does not refresh the copied local observation");
        assert.equal(state.individuals.find(value => value[0] === id)[5], 0n);
        assert.equal(state.resting.find(value => value[0] === id)[2], 1n);
      }
    }
    for (const wake of events.filter(event => event[2] === 1n)) {
      checkedWake++;
      const actor = state.actors.find(value => value[0] === wake[3]);
      if (actor) assert.equal(actor[12], tick, "wake grants a fresh current-tick observation");
    }
    for (const birth of births) {
      checkedBirth++;
      const newborn = state.resting.find(value => value[0] === birth[3]);
      assert.ok(newborn);
      assert.ok(newborn.slice(1).every(field => field === 0n));
      assert.equal(events.filter(event => event[3] === birth[3]).length, 0);
    }
  }
  assert.deepEqual([checkedRest, checkedBirth, checkedWake], [53, 10, 17]);
});

test("rest journal coverage distinguishes a known quiet tail from evicted early actions", optional, async () => {
  const api = await host();
  for (const minimum of [2, 3]) {
    api.moss_resting_reset(1, minimum, 2);
    for (let tick = 0; tick < 120; tick++) api.moss_resting_step();
    const state = snapshot(api), s = state.summary, m = state.space, r = state.rest;
    const expected = minimum === 2
      ? [[2n, 120n, 11n], [0n, 120n, 0n], [0n, 120n, 0n], [0n, 120n, 0n]]
      : [[68n, 120n, 296n], [0n, 120n, 0n], [6n, 120n, 28n], [10n, 120n, 31n]];
    assert.deepEqual([s.slice(23, 26), s.slice(26, 29), m.slice(4, 7), r.slice(11, 14)], expected);
    assert.equal(r[14], state.policy[6]);
    assert.ok(r[10] <= r[14]);
    assert.ok(state.restEvents.every(event => event[0] === s[0] && event[1] >= 1n && event[1] <= 120n));
    const retained = BigInt(state.restEvents.filter(event => event[2] === 3n).length);
    if (minimum === 2) {
      assert.equal(retained, r[7], "all recorded actions remain available");
      assert.equal(state.restEvents.filter(event => event[1] > 40n).length, 0,
        "the fully covered late interval is known to contain no rest actions after extinction");
    } else {
      assert.equal(state.restEvents.length, 128);
      assert.ok(retained < r[7]);
      assert.ok(1n <= r[11], "full-run absence queries cannot assume evicted early events were zero");
      assert.equal(r[7], 53n);
    }
  }
});
