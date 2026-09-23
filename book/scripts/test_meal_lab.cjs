"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { resolveMeals } = require("../theme/meal-lab.js");

test("the five-biomass account reads the current remainder on each turn", () => {
    assert.deepEqual(resolveMeals(5, [7, 1]), {
        startingBiomass: 5,
        inputOrder: [7, 1],
        resolutionOrder: [1, 7],
        steps: [
            {
                id: 1, reserveBefore: 0, capacity: 100, biteLimit: 4,
                biomassBefore: 5, consumed: 4, reserveAfter: 4, biomassAfter: 1,
            },
            {
                id: 7, reserveBefore: 0, capacity: 100, biteLimit: 4,
                biomassBefore: 1, consumed: 1, reserveAfter: 1, biomassAfter: 0,
            },
        ],
        totalConsumed: 5,
        remainingBiomass: 0,
    });
});

test("every allowed biomass has literal allocations in either input order", () => {
    const cases = [
        [0, 0, 0, 0],
        [1, 1, 0, 0],
        [2, 2, 0, 0],
        [3, 3, 0, 0],
        [4, 4, 0, 0],
        [5, 4, 1, 0],
        [6, 4, 2, 0],
        [7, 4, 3, 0],
        [8, 4, 4, 0],
        [9, 4, 4, 1],
        [10, 4, 4, 2],
        [11, 4, 4, 3],
        [12, 4, 4, 4],
    ];
    for (const [biomass, first, second, remainder] of cases) {
        const normal = resolveMeals(biomass, [1, 7]);
        const reversed = resolveMeals(biomass, [7, 1]);
        for (const result of [normal, reversed]) {
            assert.deepEqual(result.resolutionOrder, [1, 7]);
            assert.deepEqual(result.steps.map(step => step.consumed), [first, second], `Starting biomass ${biomass}`);
            assert.deepEqual(result.steps.map(step => step.reserveAfter), [first, second]);
            assert.equal(result.remainingBiomass, remainder);
            assert.equal(result.totalConsumed, first + second);
            assert.equal(result.totalConsumed + result.remainingBiomass, biomass);
            assert.equal(result.steps[0].biomassAfter, result.steps[1].biomassBefore);
            for (const step of result.steps) {
                assert.ok(step.consumed >= 0 && step.consumed <= 4);
                assert.ok(step.reserveAfter >= 0 && step.reserveAfter <= step.capacity);
                assert.equal(step.biomassBefore - step.biomassAfter, step.consumed);
            }
        }
        assert.deepEqual(normal.steps, reversed.steps);
        assert.deepEqual(normal.inputOrder, [1, 7]);
        assert.deepEqual(reversed.inputOrder, [7, 1]);
    }
});

test("the model does not mutate or retain the caller's order array", () => {
    const supplied = Object.freeze([7, 1]);
    const result = resolveMeals(5, supplied);
    assert.deepEqual(supplied, [7, 1]);
    assert.notEqual(result.inputOrder, supplied);
    assert.notEqual(result.resolutionOrder, supplied);
    assert.notEqual(result.inputOrder, result.resolutionOrder);
    result.inputOrder[0] = 99;
    result.resolutionOrder.reverse();
    result.steps[0].consumed = 99;
    assert.deepEqual(supplied, [7, 1]);
    assert.deepEqual(resolveMeals(5).steps.map(step => step.consumed), [4, 1]);
});

test("only whole biomass within the displayed bounds is accepted", () => {
    for (const value of [-1, 13, 2.5, NaN, Infinity, -Infinity, "5", "", null, undefined, 5n]) {
        assert.throws(() => resolveMeals(value), RangeError);
    }
    assert.equal(resolveMeals(0).remainingBiomass, 0);
    assert.equal(resolveMeals(12).remainingBiomass, 4);
});

test("the fixed teaching cast cannot gain duplicate or unknown contenders", () => {
    for (const order of [null, "1,7", [], [1], [1, 1], [7, 7], [1, 9], [1, 7, 9], ["1", 7]]) {
        assert.throws(() => resolveMeals(5, order), TypeError);
    }
    assert.deepEqual(resolveMeals(5).inputOrder, [1, 7]);
});
