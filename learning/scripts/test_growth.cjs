const {test}=require("node:test");
const assert=require("node:assert/strict");
const {growthTick}=require("../assets/models.js");
const initial=biomass=>({tick:0,biomass,reserve:0,added:0,eaten:0});
test("phase order changes the first meal and the later traces converge",()=>{
  let a=initial(0),b=initial(0);const traces=[];
  for(let i=0;i<4;i++){a=growthTick(a,"growth-first");b=growthTick(b,"meal-first");traces.push([a.biomass,a.reserve,b.biomass,b.reserve]);}
  assert.deepEqual(traces,[[1,2,3,0],[2,4,4,2],[0,6,2,4],[0,6,0,6]]);
  assert.equal(a.eaten,0);assert.equal(b.eaten,2);
});
test("meal opens room for external supply at a full patch",()=>{
  const start=initial(4);const a=growthTick(start,"growth-first"),b=growthTick(start,"meal-first");
  assert.deepEqual([a.added,a.biomass,a.reserve],[0,2,2]);
  assert.deepEqual([b.added,b.biomass,b.reserve],[2,4,2]);
  assert.deepEqual(start,initial(4));
});
test("bounded stores conserve transfers and only actual growth adds supply",()=>{
  for(const order of ["growth-first","meal-first"]){let state=initial(0);for(let i=0;i<16;i++){const next=growthTick(state,order);assert.equal(next.biomass+next.reserve,state.biomass+state.reserve+next.added);assert.ok(next.biomass>=0&&next.biomass<=4);assert.ok(next.reserve>=0&&next.reserve<=20);if(!next.lit)assert.equal(next.added,0);state=next;}}
});
