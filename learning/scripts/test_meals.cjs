const {test} = require("node:test");
const assert = require("node:assert/strict");
const {resolveMeals} = require("../assets/models.js");
const consumers = () => [{id:9,reserve:0,capacity:10,request:3},{id:2,reserve:0,capacity:10,request:3}];

test("later consumer sees authoritative depletion independent of input order",()=>{
  const result=resolveMeals(5,consumers());
  assert.deepEqual(result.outcomes,[{id:2,taken:3},{id:9,taken:2}]);
  assert.equal(result.biomass,0);
  assert.deepEqual(resolveMeals(5,consumers().reverse()),result);
  assert.deepEqual(resolveMeals(result.biomass,result.consumers).outcomes,[{id:2,taken:0},{id:9,taken:0}]);
});

test("capacity changes allocation without destroying unconsumed units",()=>{
  const rows=consumers(); rows[1].capacity=2;
  const result=resolveMeals(8,rows);
  assert.deepEqual(result.outcomes,[{id:2,taken:2},{id:9,taken:3}]);
  assert.equal(result.biomass,3);
  assert.equal(result.consumers.reduce((sum,c)=>sum+c.reserve,0)+result.biomass,8);
  assert.deepEqual(rows.map(c=>c.reserve),[0,0],"input readings remain owned snapshots");
});

test("invalid reserve and duplicate IDs reject without changing inputs",()=>{
  const rows=consumers();rows[1].reserve=11;
  const before=JSON.stringify(rows);
  assert.throws(()=>resolveMeals(5,rows));
  assert.equal(JSON.stringify(rows),before);
  assert.throws(()=>resolveMeals(5,[consumers()[0],consumers()[0]]));
});
