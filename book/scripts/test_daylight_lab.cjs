"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { DAYLIGHT_RULES, DAYLIGHT_PRESETS, stepDaylight, advanceToBoundary, daylightPhaseView } = require("../theme/daylight-lab.js");

test("the strip previews the next executing phase and wraps independently of the count", () => {
    for (const [completed, next, phase, daylight] of [
        [118, 119, 119, true], [119, 120, 120, false],
        [238, 239, 239, false], [239, 240, 0, true], [240, 241, 1, true],
    ]) {
        const view = daylightPhaseView(completed);
        assert.equal(view.completedTick, completed);
        assert.equal(view.nextTick, next);
        assert.equal(view.phase, phase);
        assert.equal(view.daylight, daylight);
        assert.ok(view.markerPercent > 0 && view.markerPercent < 100);
        assert.equal(view.markerPercent < 50, daylight);
        assert.equal(stepDaylight({ completedTick: completed, reserve: 0, biomass: 0 }).phase, view.phase);
    }
    assert.ok(daylightPhaseView(239).markerPercent < 1);
    assert.ok(daylightPhaseView(238).markerPercent > 99);
    for (const invalid of [-1, 1.5, "119", NaN, Infinity, Number.MAX_SAFE_INTEGER]) {
        assert.throws(() => daylightPhaseView(invalid), RangeError);
    }
});

test("literal executing-tick boundaries sample the next tick before completing it", () => {
    for (const [completedTick, executingTick, phase, daylight, growth, reserve] of [
        [118, 119, 119, true, 1, 20],
        [119, 120, 120, false, 0, 19],
        [238, 239, 239, false, 0, 19],
        [239, 240, 0, true, 1, 20],
    ]) {
        const result = stepDaylight({ completedTick, reserve: 20, biomass: 0 });
        assert.equal(result.before.completedTick, completedTick);
        assert.equal(result.executingTick, executingTick);
        assert.equal(result.phase, phase);
        assert.equal(result.daylight, daylight);
        assert.equal(result.requestedGrowth, growth);
        assert.equal(result.actualGrowth, growth);
        assert.equal(result.requestedMaintenance, 1);
        assert.equal(result.actualMaintenance, 1);
        assert.equal(result.actualMeal, growth);
        assert.deepEqual(result.after, { completedTick: executingTick, reserve, biomass: 0 });
    }
});

test("the dusk preset reproduces the chapter's continuous two-tick account", () => {
    const day = stepDaylight(DAYLIGHT_PRESETS.dusk);
    assert.deepEqual(day.after, { completedTick: 119, reserve: 20, biomass: 0 });
    const night = stepDaylight(day.after);
    assert.deepEqual(night.after, { completedTick: 120, reserve: 19, biomass: 0 });
    assert.equal(day.actualMeal, 1);
    assert.equal(night.actualMeal, 0);
});

test("night suppresses production but leaves stored food available for a meal", () => {
    const result = stepDaylight(DAYLIGHT_PRESETS.nightFood);
    assert.equal(result.daylight, false);
    assert.equal(result.requestedGrowth, 0);
    assert.equal(result.actualGrowth, 0);
    assert.equal(result.biomassAfterGrowth, 4);
    assert.equal(result.actualMaintenance, 1);
    assert.equal(result.actualMeal, 4);
    assert.deepEqual(result.after, { completedTick: 120, reserve: 23, biomass: 0 });
});

test("growth runs before the meal opens capacity, and the reserve bounds that meal", () => {
    const result = stepDaylight(DAYLIGHT_PRESETS.fullPatch);
    assert.equal(result.requestedGrowth, 1);
    assert.equal(result.actualGrowth, 0);
    assert.equal(result.biomassAfterGrowth, 100);
    assert.equal(result.reserveAfterMaintenance, 99);
    assert.equal(result.actualMeal, 1);
    assert.deepEqual(result.after, { completedTick: 119, reserve: 100, biomass: 99 });
});

test("saturating upkeep reports zero spent at zero without applying death", () => {
    const night = stepDaylight(DAYLIGHT_PRESETS.dawn);
    assert.equal(night.requestedMaintenance, 1);
    assert.equal(night.actualMaintenance, 0);
    assert.equal(night.actualMeal, 0);
    assert.deepEqual(night.after, { completedTick: 239, reserve: 0, biomass: 0 });
    const dawn = stepDaylight(night.after);
    assert.equal(dawn.actualMaintenance, 0);
    assert.equal(dawn.actualGrowth, 1);
    assert.equal(dawn.actualMeal, 1);
    assert.deepEqual(dawn.after, { completedTick: 240, reserve: 1, biomass: 0 });
});

test("bounded boundary advances execute all intervening upkeep rather than skip it", () => {
    const dusk = advanceToBoundary(DAYLIGHT_PRESETS.dusk);
    assert.deepEqual(dusk.steps.map(step => step.executingTick), [119, 120]);
    assert.deepEqual(dusk.after, { completedTick: 120, reserve: 19, biomass: 0 });
    const dawn = advanceToBoundary(dusk.after);
    assert.equal(dawn.steps.length, 120);
    assert.equal(dawn.steps[0].executingTick, 121);
    assert.equal(dawn.steps.at(-1).executingTick, 240);
    assert.deepEqual(dawn.after, { completedTick: 240, reserve: 1, biomass: 0 });
    assert.equal(dawn.steps.reduce((sum, step) => sum + step.actualMaintenance, 0), 19);
    assert.equal(dawn.steps.reduce((sum, step) => sum + step.actualGrowth, 0), 1);
    assert.equal(dawn.steps.reduce((sum, step) => sum + step.actualMeal, 0), 1);
    for (const completedTick of [0, 118, 119, 120, 238, 239, 240, 359]) {
        const result = advanceToBoundary({ completedTick, reserve: 20, biomass: 0 });
        assert.ok(result.steps.length >= 1 && result.steps.length <= 120);
        assert.equal(result.after.completedTick % 120, 0);
        assert.equal(result.after.completedTick - completedTick, result.steps.length);
    }
});

test("pure results do not mutate or retain the supplied state or authored presets", () => {
    const input = Object.freeze({ completedTick: 118, reserve: 20, biomass: 0 });
    const step = stepDaylight(input);
    const batch = advanceToBoundary(input);
    step.before.reserve = 99;
    step.after.biomass = 99;
    batch.before.reserve = 99;
    batch.after.biomass = 99;
    assert.deepEqual(input, { completedTick: 118, reserve: 20, biomass: 0 });
    assert.deepEqual(DAYLIGHT_PRESETS.dusk, input);
    assert.deepEqual(stepDaylight(input).after, { completedTick: 119, reserve: 20, biomass: 0 });
    assert.equal(batch.steps.at(-1).after.biomass, 0);
    assert.equal(DAYLIGHT_RULES.energyCapacity, 100);
});

test("invalid stores and inexact clocks cannot silently create plausible accounts", () => {
    for (const state of [null, undefined, "118"]) assert.throws(() => stepDaylight(state), TypeError);
    const valid = { completedTick: 118, reserve: 20, biomass: 0 };
    for (const value of [-1, 1.5, NaN, Infinity, "119", Number.MAX_SAFE_INTEGER + 1]) {
        assert.throws(() => stepDaylight({ ...valid, completedTick: value }), RangeError);
    }
    for (const field of ["reserve", "biomass"]) {
        for (const value of [-1, 101, 1.5, NaN, Infinity, "20", undefined]) {
            assert.throws(() => stepDaylight({ ...valid, [field]: value }), RangeError);
        }
    }
    assert.throws(() => stepDaylight({ ...valid, completedTick: Number.MAX_SAFE_INTEGER }), RangeError);
});
