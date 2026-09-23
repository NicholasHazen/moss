/* Authored same-cell phase traces, independent of the live Rust simulation. */
(function () {
    "use strict";
    const PHASES = ["Maintenance", "Choice", "Movement", "Eating", "Completion"];
    const PRESETS = {
        seventy_four: { reserve: 74, activity: "Seeking", target: 3, biomass: 8 },
        seventy_two: { reserve: 72, activity: "Seeking", target: 3, biomass: 8 },
        empty: { reserve: 35, activity: "Seeking", target: null, biomass: 0 },
        idle_boundary: { reserve: 46, activity: "Idle", target: null, biomass: 8 },
    };
    function initialTickState(preset = "seventy_four") {
        if (!Object.hasOwn(PRESETS, preset)) throw new RangeError("Unknown authored tick case");
        return { ...PRESETS[preset], completed: 0, phase: 0, trace: [], explanation: "Ready for maintenance. Fern and Meadow occupy the same cell." };
    }
    function advancePhase(input) {
        const state = { ...input, trace: input.phase === 0 ? [] : input.trace.map(row => ({ ...row })) };
        const executing = input.completed + 1;
        let explanation;
        switch (input.phase) {
            case 0: {
                const spent = Math.min(state.reserve, 1);
                state.reserve -= spent;
                explanation = `Maintenance actually spends ${spent}. Choice will read reserve ${state.reserve}.`;
                break;
            }
            case 1:
                if (state.activity === "Idle" && state.reserve < 45) state.activity = "Seeking";
                else if (state.activity === "Seeking" && state.reserve >= 75) state.activity = "Idle";
                state.target = state.activity === "Seeking" && state.biomass > 0 ? 3 : null;
                explanation = `Choice reads ${state.reserve}: activity is ${state.activity}; target is ${state.target === 3 ? "Meadow" : "none"}. This result is visible before action.`;
                break;
            case 2:
                explanation = state.target === 3 ? "Fern is already on Meadow's cell. Actual travel is zero cells; travel cost is zero." : "There is no target, so movement has no accepted step and no travel charge.";
                break;
            case 3: {
                const consumed = state.target === 3 ? Math.min(100 - state.reserve, state.biomass, 4) : 0;
                state.reserve += consumed;
                state.biomass -= consumed;
                explanation = `Eating transfers ${consumed} biomass into ${consumed} energy. Activity stays ${state.activity}; eating does not rerun choice.`;
                break;
            }
            case 4:
                state.completed += 1;
                explanation = `Tick ${state.completed} is complete: reserve ${state.reserve}, ${state.activity}, ${state.biomass} biomass. The next decision follows the next maintenance phase.`;
                break;
            default: throw new RangeError("Unknown phase");
        }
        state.trace.push({ tick: executing, phase: PHASES[input.phase], reserve: state.reserve, activity: state.activity, target: state.target, biomass: state.biomass });
        state.phase = (input.phase + 1) % PHASES.length;
        state.explanation = explanation;
        return state;
    }
    function finishTick(input) {
        let state = input;
        for (let step = 0; step < PHASES.length - input.phase; step += 1) state = advancePhase(state);
        return state;
    }
    if (typeof module !== "undefined" && module.exports) module.exports = { initialTickState, advancePhase, finishTick };
    if (typeof document === "undefined") return;

    function setup(lab) {
        const preset = lab.querySelector('[data-input="preset"]');
        const advance = lab.querySelector('[data-action="advance"]');
        const finish = lab.querySelector('[data-action="finish"]');
        const reset = lab.querySelector('[data-action="reset"]');
        const explanation = lab.querySelector('[data-output="explanation"]');
        const stateOutput = lab.querySelector('[data-output="state"]');
        const rows = lab.querySelector('[data-output="trace"]');
        let state = initialTickState(preset.value);
        function render() {
            stateOutput.textContent = `Completed ticks: ${state.completed}. Reserve: ${state.reserve} / 100 energy units. Activity: ${state.activity}. Target: ${state.target === 3 ? "Meadow #3" : "none"}. Meadow: ${state.biomass} biomass units.`;
            explanation.textContent = state.explanation;
            advance.textContent = `Next: ${PHASES[state.phase].toLowerCase()}`;
            rows.replaceChildren(...state.trace.map(row => {
                const tr = document.createElement("tr");
                const phase = document.createElement("td");
                phase.textContent = `Tick ${row.tick}: ${row.phase}`;
                const values = document.createElement("td");
                values.textContent = `Reserve ${row.reserve}; ${row.activity}; target ${row.target === 3 ? "Meadow" : "none"}; biomass ${row.biomass}.`;
                tr.append(phase, values);
                return tr;
            }));
        }
        function restart() { state = initialTickState(preset.value); render(); }
        preset.addEventListener("change", restart);
        advance.addEventListener("click", () => { state = advancePhase(state); render(); });
        finish.addEventListener("click", () => { state = finishTick(state); render(); });
        reset.addEventListener("click", restart);
        for (const control of [preset, advance, finish, reset]) control.disabled = false;
        render();
    }
    function setupAll() { document.querySelectorAll('[data-lab="tick"]').forEach(setup); }
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", setupAll, { once: true });
    else setupAll();
})();
