"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { createDefaultsState, setDraftTemplate, startConfiguredRun, resetRun, editFirstOwnedCost } = require("../theme/defaults-lab.js");

function ownedCosts(state) {
    return state.individuals.map(individual => individual.ownedCost);
}

test("initial equality preserves distinct authored inputs and initialization origins", () => {
    const state = createDefaultsState();
    assert.deepEqual(state, {
        activeTemplate: 1,
        draftTemplate: 1,
        authoredHares: [
            { id: 1, name: "Fern", override: null },
            { id: 7, name: "Second hare", override: 1 },
            { id: 8, name: "Third hare", override: 0 },
        ],
        individuals: [
            { id: 1, initialCost: 1, ownedCost: 1, origin: "Default-derived", runtimeEdited: false },
            { id: 7, initialCost: 1, ownedCost: 1, origin: "Explicit override", runtimeEdited: false },
            { id: 8, initialCost: 0, ownedCost: 0, origin: "Explicit override", runtimeEdited: false },
        ],
    });
});

test("draft nine leaves owned costs alone until a configured run is started", () => {
    const initial = createDefaultsState();
    const draft = setDraftTemplate(initial, 9);
    assert.equal(draft.activeTemplate, 1);
    assert.equal(draft.draftTemplate, 9);
    assert.deepEqual(ownedCosts(draft), [1, 1, 0]);
    const started = startConfiguredRun(draft);
    assert.equal(started.activeTemplate, 9);
    assert.equal(started.draftTemplate, 9);
    assert.deepEqual(ownedCosts(started), [9, 1, 0]);
    assert.deepEqual(started.individuals.map(individual => individual.initialCost), [9, 1, 0]);
    assert.deepEqual(started.individuals.map(individual => individual.origin), ["Default-derived", "Explicit override", "Explicit override"]);
    assert.deepEqual(started.authoredHares.map(hare => hare.override), [null, 1, 0]);
    assert.equal(initial.draftTemplate, 1);
    assert.deepEqual(ownedCosts(initial), [1, 1, 0]);
});

test("Reset discards runtime edits and uses active one while draft nine stays unsubmitted", () => {
    const draft = setDraftTemplate(createDefaultsState(), 9);
    const edited = editFirstOwnedCost(draft, 5);
    assert.deepEqual(ownedCosts(edited), [5, 1, 0]);
    assert.equal(edited.individuals[0].initialCost, 1);
    assert.equal(edited.individuals[0].origin, "Default-derived");
    assert.equal(edited.individuals[0].runtimeEdited, true);
    assert.deepEqual(edited.authoredHares.map(hare => hare.override), [null, 1, 0]);
    const reset = resetRun(edited);
    assert.equal(reset.activeTemplate, 1);
    assert.equal(reset.draftTemplate, 9);
    assert.deepEqual(ownedCosts(reset), [1, 1, 0]);
    assert.deepEqual(reset.individuals.map(individual => individual.runtimeEdited), [false, false, false]);
    assert.deepEqual(ownedCosts(startConfiguredRun(edited)), [9, 1, 0]);
    assert.deepEqual(ownedCosts(edited), [5, 1, 0]);
});

test("Reset follows the newly active configuration even if a different draft is pending", () => {
    const nine = startConfiguredRun(setDraftTemplate(createDefaultsState(), 9));
    const changed = editFirstOwnedCost(setDraftTemplate(nine, 2), 0);
    assert.deepEqual(ownedCosts(changed), [0, 1, 0]);
    const reset = resetRun(changed);
    assert.equal(reset.activeTemplate, 9);
    assert.equal(reset.draftTemplate, 2);
    assert.deepEqual(ownedCosts(reset), [9, 1, 0]);
    assert.deepEqual(ownedCosts(startConfiguredRun(changed)), [2, 1, 0]);
});

test("a zero template and explicit zero are present values, with different origins", () => {
    const zero = startConfiguredRun(setDraftTemplate(createDefaultsState(), 0));
    assert.deepEqual(ownedCosts(zero), [0, 1, 0]);
    assert.equal(zero.individuals[0].origin, "Default-derived");
    assert.equal(zero.individuals[2].origin, "Explicit override");
    assert.deepEqual(ownedCosts(startConfiguredRun(setDraftTemplate(zero, 9))), [9, 1, 0]);
});

test("pure transitions neither mutate nor retain input object aliases", () => {
    const initial = createDefaultsState();
    initial.authoredHares.forEach(Object.freeze);
    initial.individuals.forEach(Object.freeze);
    Object.freeze(initial.authoredHares);
    Object.freeze(initial.individuals);
    Object.freeze(initial);
    for (const next of [setDraftTemplate(initial, 9), startConfiguredRun(initial), resetRun(initial), editFirstOwnedCost(initial, 5)]) {
        assert.notEqual(next, initial);
        assert.notEqual(next.authoredHares, initial.authoredHares);
        assert.notEqual(next.individuals, initial.individuals);
        for (let i = 0; i < 3; i += 1) {
            assert.notEqual(next.authoredHares[i], initial.authoredHares[i]);
            assert.notEqual(next.individuals[i], initial.individuals[i]);
        }
        next.authoredHares[0].override = 12;
        next.individuals[1].ownedCost = 12;
    }
    assert.deepEqual(initial, createDefaultsState());
});

test("invalid draft and runtime costs are rejected rather than rounded or clamped", () => {
    const state = createDefaultsState();
    for (const value of [-1, 13, 1.5, NaN, Infinity, -Infinity, "9", "", null, undefined, 9n]) {
        assert.throws(() => setDraftTemplate(state, value), RangeError);
        assert.throws(() => editFirstOwnedCost(state, value), RangeError);
    }
    assert.deepEqual(ownedCosts(startConfiguredRun(setDraftTemplate(state, 12))), [12, 1, 0]);
    assert.deepEqual(ownedCosts(editFirstOwnedCost(state, 0)), [0, 1, 0]);
    assert.deepEqual(state, createDefaultsState());
});
