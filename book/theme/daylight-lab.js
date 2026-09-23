/* Independent teaching model. This does not execute Moss's Rust simulation. */
(function () {
    "use strict";

    const DAYLIGHT_RULES = Object.freeze({
        ticksPerDay: 240,
        daylightPhases: 120,
        biomassCapacity: 100,
        energyCapacity: 100,
        fullLightGrowth: 1,
        maintenance: 1,
        biteLimit: 4,
    });
    const DAYLIGHT_PRESETS = Object.freeze({
        dusk: Object.freeze({ completedTick: 118, reserve: 20, biomass: 0 }),
        nightFood: Object.freeze({ completedTick: 119, reserve: 20, biomass: 4 }),
        dawn: Object.freeze({ completedTick: 238, reserve: 0, biomass: 0 }),
        fullPatch: Object.freeze({ completedTick: 118, reserve: 100, biomass: 100 }),
    });

    function validateState(state) {
        if (state === null || typeof state !== "object") {
            throw new TypeError("Supply completedTick, reserve and biomass as a state object.");
        }
        if (!Number.isSafeInteger(state.completedTick) || state.completedTick < 0) {
            throw new RangeError("Completed tick must be a nonnegative safe integer.");
        }
        for (const [field, capacity] of [["reserve", DAYLIGHT_RULES.energyCapacity], ["biomass", DAYLIGHT_RULES.biomassCapacity]]) {
            if (!Number.isSafeInteger(state[field]) || state[field] < 0 || state[field] > capacity) {
                throw new RangeError(`${field} must be a whole number from 0 to ${capacity}.`);
            }
        }
    }

    // Contact and grazer eligibility are assumed. No choice, travel or death runs.
    function stepDaylight(state) {
        validateState(state);
        if (state.completedTick === Number.MAX_SAFE_INTEGER) {
            throw new RangeError("The next executing tick exceeds JavaScript's exact integer range.");
        }
        const before = { completedTick: state.completedTick, reserve: state.reserve, biomass: state.biomass };
        const executingTick = before.completedTick + 1;
        const phase = executingTick % DAYLIGHT_RULES.ticksPerDay;
        const daylight = phase < DAYLIGHT_RULES.daylightPhases;
        const requestedGrowth = daylight ? DAYLIGHT_RULES.fullLightGrowth : 0;
        const actualGrowth = Math.min(requestedGrowth, DAYLIGHT_RULES.biomassCapacity - before.biomass);
        const biomassAfterGrowth = before.biomass + actualGrowth;
        const requestedMaintenance = DAYLIGHT_RULES.maintenance;
        const actualMaintenance = Math.min(before.reserve, requestedMaintenance);
        const reserveAfterMaintenance = before.reserve - actualMaintenance;
        const actualMeal = Math.min(
            DAYLIGHT_RULES.energyCapacity - reserveAfterMaintenance,
            biomassAfterGrowth,
            DAYLIGHT_RULES.biteLimit
        );
        return {
            before,
            executingTick,
            phase,
            daylight,
            requestedGrowth,
            actualGrowth,
            biomassAfterGrowth,
            requestedMaintenance,
            actualMaintenance,
            reserveAfterMaintenance,
            actualMeal,
            after: {
                completedTick: executingTick,
                reserve: reserveAfterMaintenance + actualMeal,
                biomass: biomassAfterGrowth - actualMeal,
            },
        };
    }

    // Execute every intervening tick; never jump the clock over omitted biology.
    // A boundary is the next multiple of 120, reached in 1–120 steps.
    function advanceToBoundary(state) {
        validateState(state);
        const count = DAYLIGHT_RULES.daylightPhases - state.completedTick % DAYLIGHT_RULES.daylightPhases;
        const before = { completedTick: state.completedTick, reserve: state.reserve, biomass: state.biomass };
        let current = { ...before };
        const steps = [];
        for (let index = 0; index < count; index += 1) {
            const result = stepDaylight(current);
            steps.push(result);
            current = result.after;
        }
        return { before, after: { ...current }, steps };
    }

    // Presentation only: place the *next executing* phase at its cell's center.
    // Reading this projection does not execute any of the exchanges above.
    function daylightPhaseView(completedTick) {
        if (!Number.isSafeInteger(completedTick) || completedTick < 0
            || completedTick === Number.MAX_SAFE_INTEGER) {
            throw new RangeError("The preview needs an exact completed count and next tick.");
        }
        const nextTick = completedTick + 1;
        const phase = nextTick % DAYLIGHT_RULES.ticksPerDay;
        return {
            completedTick,
            nextTick,
            phase,
            daylight: phase < DAYLIGHT_RULES.daylightPhases,
            markerPercent: (phase + 0.5) / DAYLIGHT_RULES.ticksPerDay * 100,
        };
    }

    if (typeof module !== "undefined" && module.exports) {
        module.exports = { DAYLIGHT_RULES, DAYLIGHT_PRESETS, stepDaylight, advanceToBoundary, daylightPhaseView };
    }
    if (typeof document === "undefined") return;

    function setupDaylight(lab) {
        const presetInput = lab.querySelector('[data-input="preset"]');
        const stepButton = lab.querySelector('[data-action="step"]');
        const boundaryButton = lab.querySelector('[data-action="boundary"]');
        const resetButton = lab.querySelector('[data-action="reset"]');
        const startOutput = lab.querySelector('[data-output="start"]');
        const stateOutput = lab.querySelector('[data-output="state"]');
        const outcomesOutput = lab.querySelector('[data-output="outcomes"]');
        const explanation = lab.querySelector('[data-output="explanation"]');
        if (![presetInput, stepButton, boundaryButton, resetButton, startOutput, stateOutput, outcomesOutput, explanation].every(Boolean)) return;

        let state = { ...DAYLIGHT_PRESETS.dusk };
        let lastAction = null;

        const phaseVisual = document.createElement("figure");
        phaseVisual.className = "daylight-phase-visual";
        const phaseReadout = document.createElement("div");
        phaseReadout.className = "daylight-phase-readout";
        const completedLabel = document.createElement("span");
        const nextLabel = document.createElement("strong");
        phaseReadout.append(completedLabel, nextLabel);
        const phaseStrip = document.createElement("div");
        phaseStrip.className = "daylight-phase-strip";
        phaseStrip.setAttribute("role", "img");
        for (const [kind, label, range] of [["day", "Daylight", "phases 0–119"], ["night", "Night", "phases 120–239"]]) {
            const segment = document.createElement("div");
            segment.className = `daylight-phase-segment daylight-phase-${kind}`;
            segment.setAttribute("aria-hidden", "true");
            const title = document.createElement("strong");
            title.textContent = label;
            const phases = document.createElement("span");
            phases.textContent = range;
            segment.append(title, phases);
            phaseStrip.append(segment);
        }
        const phaseMarker = document.createElement("span");
        phaseMarker.className = "daylight-phase-marker";
        phaseMarker.setAttribute("aria-hidden", "true");
        phaseStrip.append(phaseMarker);
        const phaseAxis = document.createElement("div");
        phaseAxis.className = "daylight-phase-axis";
        phaseAxis.setAttribute("aria-hidden", "true");
        for (const label of ["0 · dawn", "120 · dusk", "239"]) {
            const tick = document.createElement("span");
            tick.textContent = label;
            phaseAxis.append(tick);
        }
        const phaseCaption = document.createElement("figcaption");
        const wrapLabel = document.createElement("p");
        wrapLabel.className = "daylight-phase-wrap";
        wrapLabel.textContent = "↶ After phase 239, the next phase is 0. The phase wraps; the tick count keeps increasing.";
        phaseVisual.append(phaseReadout, phaseStrip, phaseAxis, phaseCaption, wrapLabel);
        stateOutput.before(phaseVisual);

        function renderPhase(view) {
            const light = view.daylight ? "daylight" : "night";
            completedLabel.textContent = `Completed tick ${view.completedTick}`;
            nextLabel.textContent = `Next: tick ${view.nextTick} · phase ${view.phase}`;
            phaseMarker.style.left = `${view.markerPercent}%`;
            phaseStrip.setAttribute("aria-label", `A 240-phase cycle: daylight from 0 through 119, night from 120 through 239. The next executing tick is ${view.nextTick}; its marker is at phase ${view.phase}, in ${light}.`);
            phaseVisual.dataset.light = view.daylight ? "day" : "night";
            if (view.phase === 119) {
                phaseCaption.textContent = "The marker previews the last daylight phase. Step executes it; the following preview will cross into night at phase 120.";
            } else if (view.phase === 120) {
                phaseCaption.textContent = "The marker previews the first night phase. Step will use night light for this tick, then record it as completed.";
            } else if (view.phase === 239) {
                phaseCaption.textContent = "The marker previews the last night phase. Step executes it; the following preview will wrap to daylight at phase 0.";
            } else if (view.phase === 0) {
                phaseCaption.textContent = `The marker has wrapped to the first daylight phase. Step will execute tick ${view.nextTick}, using phase 0; the completed count has not advanced yet.`;
            } else {
                phaseCaption.textContent = `The marker previews ${light} for the next executing tick. Step performs that tick's exchanges, then advances the completed count.`;
            }
        }

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

        function render() {
            const preset = DAYLIGHT_PRESETS[presetInput.value];
            startOutput.textContent = `Selected starting state: completed tick ${preset.completedTick}; Fern reserve ${preset.reserve} of 100 energy units; Meadow biomass ${preset.biomass} of 100. Reset restores these values without executing a tick.`;
            const phaseView = daylightPhaseView(state.completedTick);
            const nextTick = phaseView.nextTick;
            const nextPhase = phaseView.phase;
            const nextLight = phaseView.daylight ? "day" : "night";
            renderPhase(phaseView);
            stateOutput.replaceChildren(
                card("Completed ticks now", String(state.completedTick), "All displayed stores include the work of this completed count."),
                card("Next executing tick", String(nextTick), `If stepped, phase ${nextPhase} supplies ${nextLight} light. This preview performs no growth or meal.`),
                card("Fern's reserve now", String(state.reserve), "Energy units; capacity 100. Zero does not remove Fern in this illustration."),
                card("Meadow's biomass now", String(state.biomass), "Biomass units; capacity 100. Stored food remains edible at night.")
            );
            const untilBoundary = DAYLIGHT_RULES.daylightPhases - state.completedTick % DAYLIGHT_RULES.daylightPhases;
            boundaryButton.textContent = `Step to next boundary (${untilBoundary} ${untilBoundary === 1 ? "tick" : "ticks"})`;
            if (lastAction === null) {
                outcomesOutput.replaceChildren();
                explanation.textContent = `No tick has executed since this preset was loaded. Step will execute tick ${nextTick}, using phase ${nextPhase} (${nextLight}), then complete it. All contact is assumed; this model applies no death policy.`;
                return;
            }
            const result = lastAction.steps[lastAction.steps.length - 1];
            outcomesOutput.replaceChildren(
                card("Last executing tick", String(result.executingTick), `Completed before: ${result.before.completedTick}; sampled phase ${result.phase}: ${result.daylight ? "day" : "night"}; completed after: ${result.after.completedTick}.`),
                card("Growth in that tick", `${result.requestedGrowth} → ${result.actualGrowth}`, `Requested → actually retained biomass. Meadow: ${result.before.biomass} before growth, ${result.biomassAfterGrowth} after growth.`),
                card("Upkeep in that tick", `${result.requestedMaintenance} → ${result.actualMaintenance}`, `Requested → actually deducted energy. Reserve: ${result.before.reserve} before upkeep, ${result.reserveAfterMaintenance} afterward.`),
                card("Meal in that tick", String(result.actualMeal), `Biomass actually transferred at 1 energy per biomass unit. After the meal: reserve ${result.after.reserve}, biomass ${result.after.biomass}.`)
            );
            const totals = lastAction.steps.reduce((sum, step) => ({
                growth: sum.growth + step.actualGrowth,
                maintenance: sum.maintenance + step.actualMaintenance,
                meals: sum.meals + step.actualMeal,
            }), { growth: 0, maintenance: 0, meals: 0 });
            const count = lastAction.steps.length;
            explanation.textContent = `${count === 1 ? "Executed one complete tick" : `Executed every tick from ${lastAction.steps[0].executingTick} through ${result.executingTick} (${count} ticks)`}. Completed count changed from ${lastAction.before.completedTick} to ${state.completedTick}. Across this action, actual growth was ${totals.growth} biomass, upkeep deducted ${totals.maintenance} energy, and meals transferred ${totals.meals} biomass. The cards above detail only the last tick: phase ${result.phase}, ${result.daylight ? "day" : "night"}. Fern remains present even at zero; no death policy runs.`;
        }

        function reset() {
            state = { ...DAYLIGHT_PRESETS[presetInput.value] };
            lastAction = null;
            render();
        }

        stepButton.addEventListener("click", () => {
            const result = stepDaylight(state);
            lastAction = { before: result.before, after: result.after, steps: [result] };
            state = { ...result.after };
            render();
        });
        boundaryButton.addEventListener("click", () => {
            lastAction = advanceToBoundary(state);
            state = { ...lastAction.after };
            render();
        });
        resetButton.addEventListener("click", reset);
        presetInput.addEventListener("change", reset);
        presetInput.disabled = false;
        stepButton.disabled = false;
        boundaryButton.disabled = false;
        resetButton.disabled = false;
        reset();
    }

    function setupLabs() {
        document.querySelectorAll('[data-lab="daylight"]').forEach(setupDaylight);
    }
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", setupLabs, { once: true });
    else setupLabs();
})();
