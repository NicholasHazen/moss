/* Original teaching model. This does not execute Moss's Rust simulation. */
(function () {
    "use strict";

    // This lab begins after eligibility checks. Both eaters have reserve 0,
    // capacity 100 and a bite limit of 4; the conversion is 1 energy/biomass.
    function resolveMeals(startingBiomass, inputOrder = [1, 7]) {
        if (!Number.isSafeInteger(startingBiomass) || startingBiomass < 0 || startingBiomass > 12) {
            throw new RangeError("Starting biomass must be a whole number from 0 to 12.");
        }
        if (!Array.isArray(inputOrder) || inputOrder.length !== 2 ||
            !inputOrder.includes(1) || !inputOrder.includes(7)) {
            throw new TypeError("Input order must contain IDs 1 and 7 exactly once.");
        }

        const suppliedOrder = [...inputOrder];
        const resolutionOrder = [...inputOrder].sort((a, b) => a - b);
        let remainingBiomass = startingBiomass;
        const steps = resolutionOrder.map(id => {
            const biomassBefore = remainingBiomass;
            const reserveBefore = 0;
            const capacity = 100;
            const biteLimit = 4;
            const consumed = Math.min(biteLimit, biomassBefore, capacity - reserveBefore);
            remainingBiomass -= consumed;
            return {
                id,
                reserveBefore,
                capacity,
                biteLimit,
                biomassBefore,
                consumed,
                reserveAfter: reserveBefore + consumed,
                biomassAfter: remainingBiomass,
            };
        });
        return {
            startingBiomass,
            inputOrder: suppliedOrder,
            resolutionOrder,
            steps,
            totalConsumed: startingBiomass - remainingBiomass,
            remainingBiomass,
        };
    }

    if (typeof module !== "undefined" && module.exports) module.exports = { resolveMeals };
    if (typeof document === "undefined") return;

    function setupMeal(lab) {
        const biomassInput = lab.querySelector('[data-input="biomass"]');
        const resolveButton = lab.querySelector('[data-action="resolve"]');
        const reverseButton = lab.querySelector('[data-action="reverse"]');
        const resetButton = lab.querySelector('[data-action="reset"]');
        const orderOutput = lab.querySelector('[data-output="order"]');
        const transfersOutput = lab.querySelector('[data-output="transfers"]');
        const remainingOutput = lab.querySelector('[data-output="remaining"]');
        const explanation = lab.querySelector('[data-output="explanation"]');
        let reversed = false;
        let resolvedCount = 0;

        function card(heading, value, detail) {
            const box = document.createElement("div");
            box.className = "state-card";
            const title = document.createElement("h4");
            title.textContent = heading;
            const amount = document.createElement("div");
            amount.className = "coordinate";
            amount.textContent = value;
            const text = document.createElement("p");
            text.textContent = detail;
            box.append(title, amount, text);
            return box;
        }

        function readResult() {
            const value = Number(biomassInput.value);
            if (biomassInput.value.trim() === "" || !biomassInput.validity.valid ||
                !Number.isSafeInteger(value) || value < 0 || value > 12) return null;
            return resolveMeals(value, reversed ? [7, 1] : [1, 7]);
        }

        function render() {
            reverseButton.setAttribute("aria-pressed", String(reversed));
            orderOutput.replaceChildren(
                card("Supplied input order", reversed ? "7 → 1" : "1 → 7", "The order in which the two eligible eaters arrive in the input."),
                card("Resolution order", "1 → 7", "Ascending stable ID: Fern resolves before ID 7 in either input order.")
            );

            const result = readResult();
            if (result === null) {
                biomassInput.setAttribute("aria-invalid", "true");
                transfersOutput.replaceChildren();
                remainingOutput.textContent = "No biomass account is shown while the starting value is invalid.";
                explanation.textContent = "Enter a whole starting biomass from 0 to 12. No eater resolves with an invalid value.";
                resolveButton.disabled = true;
                resolveButton.textContent = "Resolve ID 1";
                return;
            }

            biomassInput.removeAttribute("aria-invalid");
            const completedSteps = result.steps.slice(0, resolvedCount);
            const currentBiomass = resolvedCount === 0 ? result.startingBiomass : completedSteps[resolvedCount - 1].biomassAfter;
            const totalConsumed = result.startingBiomass - currentBiomass;
            remainingOutput.textContent = `Meadow now holds ${currentBiomass} biomass units. Started with ${result.startingBiomass}; actually consumed so far: ${totalConsumed}.`;
            transfersOutput.replaceChildren(...result.steps.map((step, index) => {
                const name = step.id === 1 ? "Fern · ID 1" : "Second hare · ID 7";
                if (index >= resolvedCount) {
                    return card(name, "Waiting", "Reserve: 0 of 100 energy units. This eater has not resolved yet.");
                }
                return card(name, String(step.consumed), `Biomass consumed: ${step.consumed}. Reserve: ${step.reserveAfter} of 100 energy units. Meadow had ${step.biomassBefore} before this turn and ${step.biomassAfter} afterward.`);
            }));

            if (resolvedCount === 0) {
                explanation.textContent = `Input order is ${result.inputOrder.join(" then ")}; resolution order is 1 then 7. Both reserves start at zero. Resolve ID 1 to begin with ${result.startingBiomass} biomass.`;
                resolveButton.textContent = "Resolve ID 1";
            } else if (resolvedCount === 1) {
                const first = result.steps[0];
                explanation.textContent = `Fern, ID 1, consumed ${first.consumed} biomass and now has ${first.reserveAfter} energy units. Meadow has ${first.biomassAfter} biomass left. ID 7 will read this current remainder.`;
                resolveButton.textContent = "Resolve ID 7";
            } else {
                explanation.textContent = `Resolution complete. ID 1 consumed ${result.steps[0].consumed}; ID 7 consumed ${result.steps[1].consumed}. Their reserves are ${result.steps[0].reserveAfter} and ${result.steps[1].reserveAfter} energy units. ${result.totalConsumed} biomass was consumed, and ${result.remainingBiomass} remains in Meadow. Reverse the input or change biomass to begin a fresh example.`;
                resolveButton.textContent = "Resolution complete";
            }
            resolveButton.disabled = resolvedCount === result.steps.length;
        }

        function restart() {
            resolvedCount = 0;
            render();
        }

        resolveButton.addEventListener("click", () => {
            if (readResult() !== null) resolvedCount = Math.min(2, resolvedCount + 1);
            render();
        });
        reverseButton.addEventListener("click", () => {
            reversed = !reversed;
            restart();
        });
        resetButton.addEventListener("click", () => {
            biomassInput.value = "5";
            reversed = false;
            restart();
        });
        biomassInput.addEventListener("input", restart);
        biomassInput.addEventListener("change", restart);
        biomassInput.disabled = false;
        reverseButton.disabled = false;
        resetButton.disabled = false;
        render();
    }

    function setupLabs() {
        document.querySelectorAll('[data-lab="meal"]').forEach(setupMeal);
    }
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", setupLabs, { once: true });
    else setupLabs();
})();
