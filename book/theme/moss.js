/* Original teaching models. These do not execute Moss's Rust simulation. */
(function () {
    "use strict";

    function movementAttempt(start, target, reserve, rate, width = 32, height = 20) {
        const inBounds = p => p.x >= 0 && p.x < width && p.y >= 0 && p.y < height;
        const original = { ...start };
        if (!inBounds(start) || !inBounds(target)) {
            return { original, proposal: null, position: original, reserve, distance: 0, cost: 0, reason: "invalid" };
        }
        const proposal = { ...start };
        if (proposal.x !== target.x) proposal.x += proposal.x < target.x ? 1 : -1;
        else if (proposal.y !== target.y) proposal.y += proposal.y < target.y ? 1 : -1;
        const proposedDistance = Math.abs(proposal.x - start.x) + Math.abs(proposal.y - start.y);
        const proposedCost = proposedDistance * rate;
        const accepted = reserve >= proposedCost;
        return {
            original, proposal, proposedDistance, proposedCost,
            position: accepted ? { ...proposal } : original,
            reserve: accepted ? reserve - proposedCost : reserve,
            distance: accepted ? proposedDistance : 0,
            cost: accepted ? proposedCost : 0,
            reason: !accepted ? "unaffordable" : proposedDistance === 0 ? "arrived" : "accepted",
        };
    }

    // Pure model exported for the book's Node checks; no browser automation needed.
    if (typeof module !== "undefined" && module.exports) module.exports = { movementAttempt };
    if (typeof document === "undefined") return;

    function setupMovement(lab) {
        lab.querySelectorAll("input, select, button").forEach(control => { control.disabled = false; });
        const reserveInput = lab.querySelector('[data-input="reserve"]');
        const rateInput = lab.querySelector('[data-input="rate"]');
        const caseInput = lab.querySelector('[data-input="case"]');
        const advance = lab.querySelector('[data-action="advance"]');
        const explanation = lab.querySelector('[data-output="explanation"]');
        const diagram = lab.querySelector('[data-output="diagram"]');
        let phase = 0;
        const names = ["1. Propose", "2. Validate", "3. Commit or reject", "Attempt complete"];
        const coordinate = p => p ? `(${p.x}, ${p.y})` : "—";
        function card(heading, position, detail) {
            const box = document.createElement("div");
            box.className = "state-card";
            const title = document.createElement("h4");
            title.textContent = heading;
            const point = document.createElement("div");
            point.className = "coordinate";
            point.textContent = coordinate(position);
            const text = document.createElement("p");
            text.textContent = detail;
            box.append(title, point, text);
            return box;
        }
        function render() {
            const valid = [reserveInput, rateInput].every(input => input.value !== "" && input.validity.valid && Number.isInteger(Number(input.value)));
            if (!valid) {
                diagram.replaceChildren();
                explanation.textContent = "Enter a whole reserve from 0 to 100 and a whole travel rate from 0 to 10. No attempt runs with an invalid input.";
                advance.disabled = true;
                advance.textContent = names[0];
                return;
            }
            const reserve = Number(reserveInput.value);
            const rate = Number(rateInput.value);
            const targets = { east: { x: 4, y: 4 }, north: { x: 2, y: 4 }, arrived: { x: 2, y: 2 }, invalid: { x: -1, y: 2 } };
            const result = movementAttempt({ x: 2, y: 2 }, targets[caseInput.value], reserve, rate);
            const committed = phase === 3;
            diagram.replaceChildren(
                card("Caller's real state", committed ? result.position : result.original, `${committed ? result.reserve : reserve} energy units in reserve`),
                card("Local proposal: next", phase > 0 ? result.proposal : null, phase === 0 ? "No proposal yet" : result.reason === "invalid" ? "Rejected by the bounds guard" : `${result.proposedDistance} cell × ${rate} energy units/cell = ${result.proposedCost} energy units`)
            );
            const messages = [
                "The original position is (2, 2). Propose a step to inspect an independent copy before writing to it.",
                result.reason === "invalid" ? "The bounds guard rejects this target before a proposal is made. The caller's values stay unchanged." : "Only the local proposal exists so far. The caller's position and reserve have not changed.",
                result.reason === "invalid" ? "Invalid coordinates remain rejected; no payment or write is allowed." : result.reason === "unaffordable" ? `The proposed cost is ${result.proposedCost}, but the reserve is ${reserve}. This attempt must not commit.` : `The proposed cost is ${result.proposedCost} and the reserve is ${reserve}. The affordability check passes. The real state is still unchanged.`,
                result.reason === "accepted" ? `Committed: position ${coordinate(result.position)}, reserve ${result.reserve}. The function returns 1 cell actually traveled.` : result.reason === "arrived" ? `Already at the target: no travel, no charge, reserve ${result.reserve}. The function returns 0.` : `Rejected: the original position and reserve remain unchanged. The function returns 0 cells traveled and charges 0.`,
            ];
            explanation.textContent = messages[phase];
            advance.textContent = names[phase];
            advance.disabled = committed;
        }
        function reset() { phase = 0; render(); }
        advance.addEventListener("click", () => { phase = Math.min(3, phase + 1); render(); });
        lab.querySelector('[data-action="reset"]').addEventListener("click", reset);
        for (const input of [reserveInput, rateInput]) {
            input.addEventListener("input", reset);
            input.addEventListener("change", reset);
        }
        caseInput.addEventListener("change", reset);
        render();
    }
    document.querySelectorAll('[data-lab="movement"]').forEach(setupMovement);
})();
