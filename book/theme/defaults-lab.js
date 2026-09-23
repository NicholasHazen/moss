/* Original teaching model of a future policy, not Moss's live simulation. */
(function () {
    "use strict";

    // The small input range is a teaching control, not the Rust u32 limit.
    function validateCost(value) {
        if (!Number.isSafeInteger(value) || value < 0 || value > 12) {
            throw new RangeError("Use a whole cost from 0 to 12 energy units per tick.");
        }
    }

    function constructIndividuals(template, authoredHares) {
        return authoredHares.map(hare => {
            const cost = hare.override === null ? template : hare.override;
            return {
                id: hare.id,
                initialCost: cost,
                ownedCost: cost,
                origin: hare.override === null ? "Default-derived" : "Explicit override",
                runtimeEdited: false,
            };
        });
    }

    function copyState(state) {
        return {
            activeTemplate: state.activeTemplate,
            draftTemplate: state.draftTemplate,
            authoredHares: state.authoredHares.map(hare => ({ ...hare })),
            individuals: state.individuals.map(individual => ({ ...individual })),
        };
    }

    function createDefaultsState() {
        const authoredHares = [
            { id: 1, name: "Fern", override: null },
            { id: 7, name: "Second hare", override: 1 },
            { id: 8, name: "Third hare", override: 0 },
        ];
        return {
            activeTemplate: 1,
            draftTemplate: 1,
            authoredHares,
            individuals: constructIndividuals(1, authoredHares),
        };
    }

    function setDraftTemplate(state, cost) {
        validateCost(cost);
        const next = copyState(state);
        next.draftTemplate = cost;
        return next;
    }

    function startConfiguredRun(state) {
        const next = copyState(state);
        next.activeTemplate = next.draftTemplate;
        next.individuals = constructIndividuals(next.activeTemplate, next.authoredHares);
        return next;
    }

    function resetRun(state) {
        const next = copyState(state);
        next.individuals = constructIndividuals(next.activeTemplate, next.authoredHares);
        return next;
    }

    function editFirstOwnedCost(state, cost) {
        validateCost(cost);
        const next = copyState(state);
        next.individuals[0].ownedCost = cost;
        next.individuals[0].runtimeEdited = true;
        return next;
    }

    if (typeof module !== "undefined" && module.exports) {
        module.exports = { createDefaultsState, setDraftTemplate, startConfiguredRun, resetRun, editFirstOwnedCost };
    }
    if (typeof document === "undefined") return;

    function setupDefaults(lab) {
        const draftInput = lab.querySelector('[data-input="draft-template"]');
        const runtimeInput = lab.querySelector('[data-input="runtime-cost"]');
        const startButton = lab.querySelector('[data-action="start-configured"]');
        const resetButton = lab.querySelector('[data-action="reset"]');
        const editButton = lab.querySelector('[data-action="edit-owned"]');
        const configurationOutput = lab.querySelector('[data-output="configuration"]');
        const individualsOutput = lab.querySelector('[data-output="individuals"]');
        const explanation = lab.querySelector('[data-output="explanation"]');
        let state = createDefaultsState();

        function card(heading, value, details) {
            const box = document.createElement("div");
            box.className = "state-card";
            const title = document.createElement("h4");
            title.textContent = heading;
            const amount = document.createElement("div");
            amount.className = "coordinate";
            amount.textContent = value;
            box.append(title, amount);
            for (const detail of details) {
                const text = document.createElement("p");
                text.textContent = detail;
                box.append(text);
            }
            return box;
        }

        function readCost(input) {
            const value = Number(input.value);
            if (input.value.trim() === "" || !input.validity.valid ||
                !Number.isSafeInteger(value) || value < 0 || value > 12) return null;
            return value;
        }

        function ownedValues() {
            return state.individuals.map(individual => `ID ${individual.id}: ${individual.ownedCost}`).join("; ");
        }

        function render(message) {
            const draft = readCost(draftInput);
            const runtimeCost = readCost(runtimeInput);
            const errors = [];
            if (draft === null) {
                draftInput.setAttribute("aria-invalid", "true");
                errors.push("Draft template is invalid: enter a whole cost from 0 to 12. Start configured run is unavailable; Reset still uses the active configuration.");
            } else draftInput.removeAttribute("aria-invalid");
            if (runtimeCost === null) {
                runtimeInput.setAttribute("aria-invalid", "true");
                errors.push("The proposed runtime cost is invalid: enter a whole cost from 0 to 12 before applying it to Fern.");
            } else runtimeInput.removeAttribute("aria-invalid");
            startButton.disabled = draft === null;
            editButton.disabled = runtimeCost === null;

            configurationOutput.replaceChildren(
                card("Active hare template", String(state.activeTemplate), ["Energy units per tick. Fixed for this run; Reset reconstructs from this value."]),
                card("Draft hare template", draft === null ? "Invalid" : String(draft), ["Used only by Start configured run. Editing this draft does not change existing owned costs."])
            );
            individualsOutput.replaceChildren(...state.individuals.map(individual => {
                const authored = state.authoredHares.find(hare => hare.id === individual.id);
                const input = authored.override === null ? "None" : `Some(${authored.override})`;
                const details = [
                    `Authored input: ${input}.`,
                    `Initialization: ${individual.origin}; constructed cost ${individual.initialCost}.`,
                    `Owned current cost: ${individual.ownedCost} energy units per tick.`,
                ];
                if (individual.runtimeEdited) {
                    details.push("A runtime edit has been applied. It changed this owned value, not the retained authored input.");
                }
                return card(`${authored.name} · ID ${individual.id}`, String(individual.ownedCost), details);
            }));
            explanation.textContent = [...errors, message].filter(Boolean).join(" ");
        }

        draftInput.addEventListener("input", () => {
            const cost = readCost(draftInput);
            if (cost !== null) state = setDraftTemplate(state, cost);
            render(`Active template remains ${state.activeTemplate}. Owned costs remain ${ownedValues()}.`);
        });
        runtimeInput.addEventListener("input", () => {
            render(`The runtime cost is a proposed edit until you apply it. Owned costs remain ${ownedValues()}.`);
        });
        startButton.addEventListener("click", () => {
            const cost = readCost(draftInput);
            if (cost === null) return;
            state = startConfiguredRun(setDraftTemplate(state, cost));
            render(`Started a configured run with active template ${state.activeTemplate}. Reconstructed owned costs: ${ownedValues()}. Authored overrides are retained, including explicit one and zero.`);
        });
        resetButton.addEventListener("click", () => {
            state = resetRun(state);
            render(`Reset reconstructed from active template ${state.activeTemplate}. Owned costs: ${ownedValues()}. Reset did not submit the draft; the draft input stays as entered. Runtime edits were discarded.`);
        });
        editButton.addEventListener("click", () => {
            const cost = readCost(runtimeInput);
            if (cost === null) return;
            state = editFirstOwnedCost(state, cost);
            render(`Fern's owned cost is now ${cost}. Active template ${state.activeTemplate}, the draft, and all authored inputs are unchanged. Other owned costs are unchanged. Reset will reconstruct Fern's cost as ${state.activeTemplate}.`);
        });
        draftInput.disabled = false;
        runtimeInput.disabled = false;
        resetButton.disabled = false;
        render("Active template and draft both begin at 1. Fern uses None; ID 7 uses Some(1); ID 8 uses Some(0). Owned costs are 1, 1 and 0. Change the draft to 9 to compare Reset with Start configured run.");
    }

    function setupLabs() {
        document.querySelectorAll('[data-lab="defaults"]').forEach(setupDefaults);
    }
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", setupLabs, { once: true });
    else setupLabs();
})();
