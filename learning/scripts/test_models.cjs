const test = require("node:test");
const assert = require("node:assert/strict");
const { tick, ownershipTrace, selectedEntities, forecast, targetView } = require("../assets/models.js");

test("three upkeep ticks match the lesson and preserve the input snapshot", () => {
  const original = { tick: 0, reserve: 60, cost: 1 };
  let state = original;
  for (let i = 0; i < 3; i++) state = tick(state);
  assert.deepEqual(state, { tick: 3, reserve: 57, cost: 1 });
  assert.deepEqual(original, { tick: 0, reserve: 60, cost: 1 });
  assert.deepEqual(tick({ tick: 8, reserve: 0, cost: 10 }), { tick: 9, reserve: 0, cost: 10 });
  assert.deepEqual(tick({ tick: 0, reserve: 3, cost: 0 }), { tick: 1, reserve: 3, cost: 0 });
});

test("copying and borrowing write to different owners", () => {
  assert.deepEqual(ownershipTrace("copy"), { original: 9, inspected: 4 });
  assert.deepEqual(ownershipTrace("borrow"), { original: 4, inspected: 4 });
});

test("missing energy, zero energy, and absent marker give distinct query membership", () => {
  const rows = [{ name:"Fern",creature:true,energy:0 },{ name:"Meadow",creature:false,energy:null },{ name:"Incomplete",creature:true,energy:null },{ name:"Marker",creature:false,energy:4 }];
  const names = (filter, optional) => selectedEntities(rows, filter, optional).map(row => row.name);
  assert.deepEqual(names(true, false), ["Fern"]);
  assert.deepEqual(names(true, true), ["Fern", "Incomplete"]);
  assert.deepEqual(names(false, false), ["Fern", "Marker"]);
  assert.deepEqual(names(false, true), ["Fern", "Meadow", "Incomplete", "Marker"]);
});

test("forecast keeps requested and paid quantities separate and rejects overflow", () => {
  assert.deepEqual(forecast(4, 3, 2), {requested:6,paid:4,remaining:0});
  assert.deepEqual(forecast(60, 3, 2), {requested:6,paid:6,remaining:54});
  assert.deepEqual(forecast(4, 0, 2), {requested:0,paid:0,remaining:4});
  assert.deepEqual(forecast(1, 4294967295, 2), {error:"overflow"});
  assert.deepEqual(forecast(4294967295, 4294967295, 1), {requested:4294967295,paid:4294967295,remaining:0});
  assert.deepEqual(forecast(4, -1, 2), {error:"invalid"});
  assert.deepEqual(forecast(4, NaN, 2), {error:"invalid"});
});

test("the inspector never invents a target or hides a present zero", () => {
  const rows = [{id:3,biomass:0},{id:8,biomass:20}];
  assert.deepEqual(targetView(null, rows), {kind:"Unchosen"});
  assert.deepEqual(targetView(3, rows), {kind:"Present",id:3,biomass:0});
  assert.deepEqual(targetView(99, rows), {kind:"Missing",id:99});
});
