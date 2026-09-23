"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { initialTickState, advancePhase, finishTick } = require("../theme/tick-lab.js");

test("choice reads 73; eating later leaves a Seeking animal at 77", () => {
    const start = initialTickState();
    let state = advancePhase(start);
    assert.deepEqual([state.reserve, state.activity, state.target, state.biomass, state.completed], [73, "Seeking", 3, 8, 0]);
    state = advancePhase(state);
    assert.deepEqual([state.reserve, state.activity, state.target], [73, "Seeking", 3]);
    state = advancePhase(state);
    assert.equal(state.reserve, 73);
    state = advancePhase(state);
    assert.deepEqual([state.reserve, state.activity, state.biomass, state.completed], [77, "Seeking", 4, 0]);
    state = advancePhase(state);
    assert.equal(state.completed, 1);
    assert.deepEqual(start, initialTickState());
    const next = finishTick(state);
    assert.deepEqual([next.reserve, next.activity, next.target, next.biomass, next.completed], [76, "Idle", null, 4, 2]);
    assert.deepEqual([state.reserve, state.activity, state.target, state.biomass, state.completed], [77, "Seeking", 3, 4, 1]);
});

test("reaching 75 after eating does not skip the following maintenance", () => {
    const first = finishTick(initialTickState("seventy_two"));
    assert.deepEqual([first.reserve, first.activity, first.biomass], [75, "Seeking", 4]);
    const second = finishTick(first);
    assert.deepEqual([second.reserve, second.activity, second.target, second.biomass], [78, "Seeking", 3, 0]);
    const third = finishTick(second);
    assert.deepEqual([third.reserve, third.activity, third.target, third.biomass], [77, "Idle", null, 0]);
});

test("an absent target and retained activity have distinct meanings", () => {
    const empty = finishTick(initialTickState("empty"));
    assert.deepEqual([empty.reserve, empty.activity, empty.target, empty.biomass], [34, "Seeking", null, 0]);
    const idle = finishTick(initialTickState("idle_boundary"));
    assert.deepEqual([idle.reserve, idle.activity, idle.target, idle.biomass], [45, "Idle", null, 8]);
    const hungry = finishTick(idle);
    assert.deepEqual([hungry.reserve, hungry.activity, hungry.target, hungry.biomass], [48, "Seeking", 3, 4]);
});

test("finish resumes at the current phase and the trace stays bounded", () => {
    const afterMaintenance = advancePhase(initialTickState());
    const finished = finishTick(afterMaintenance);
    assert.equal(finished.reserve, 77);
    assert.equal(finished.completed, 1);
    assert.equal(finished.trace.length, 5);
    const next = advancePhase(finished);
    assert.equal(next.trace.length, 1);
    assert.equal(next.reserve, 76);
    assert.equal(finished.trace.length, 5);
    let state = initialTickState("empty");
    for (let i = 0; i < 50; i += 1) state = finishTick(state);
    assert.equal(state.reserve, 0);
    assert.equal(state.trace.length, 5);
    assert.equal(state.completed, 50);
});
