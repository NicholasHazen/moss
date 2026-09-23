"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { movementAttempt } = require("../theme/moss.js");

test("movement teaching cases match the printed reserve ledger", () => {
    const start = { x: 2, y: 2 };
    const target = { x: 4, y: 4 };
    for (const [reserve, rate, expectedPosition, expectedReserve, distance] of [
        [59, 2, { x: 3, y: 2 }, 57, 1],
        [2, 2, { x: 3, y: 2 }, 0, 1],
        [1, 2, { x: 2, y: 2 }, 1, 0],
        [3, 3, { x: 3, y: 2 }, 0, 1],
        [2, 3, { x: 2, y: 2 }, 2, 0],
        [0, 0, { x: 3, y: 2 }, 0, 1],
    ]) {
        const result = movementAttempt(start, target, reserve, rate);
        assert.deepEqual(result.position, expectedPosition);
        assert.equal(result.reserve, expectedReserve);
        assert.equal(result.distance, distance);
        assert.equal(reserve - result.reserve, result.cost);
    }
    assert.deepEqual(start, { x: 2, y: 2 });
    assert.deepEqual(target, { x: 4, y: 4 });
});

test("arrival and invalid coordinates charge nothing", () => {
    for (const [start, target, reason] of [
        [{ x: 2, y: 2 }, { x: 2, y: 2 }, "arrived"],
        [{ x: 2, y: 2 }, { x: -1, y: 2 }, "invalid"],
        [{ x: -1, y: 0 }, { x: 2, y: 2 }, "invalid"],
        [{ x: 2, y: 2 }, { x: 32, y: 2 }, "invalid"],
    ]) {
        const result = movementAttempt(start, target, 10, 2);
        assert.deepEqual(result.position, start);
        assert.equal(result.reserve, 10);
        assert.equal(result.distance, 0);
        assert.equal(result.reason, reason);
    }
});

test("the rule handles both directions and selects x before y", () => {
    for (const [start, target, expected] of [
        [{ x: 1, y: 1 }, { x: 0, y: 0 }, { x: 0, y: 1 }],
        [{ x: 1, y: 1 }, { x: 2, y: 2 }, { x: 2, y: 1 }],
        [{ x: 1, y: 1 }, { x: 1, y: 0 }, { x: 1, y: 0 }],
        [{ x: 1, y: 1 }, { x: 1, y: 2 }, { x: 1, y: 2 }],
        [{ x: 30, y: 19 }, { x: 31, y: 19 }, { x: 31, y: 19 }],
    ]) {
        assert.deepEqual(movementAttempt(start, target, 2, 2).position, expected);
    }
});
