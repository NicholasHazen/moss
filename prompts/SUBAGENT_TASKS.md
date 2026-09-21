# Bounded subagent tasks

These are task templates, not tool definitions or claims that subagents are available. The lead uses real delegation only when supported. One helper is often enough; default to no more than two at a time.

## Shared contract

Give each helper the relevant project documents and the actual current repository state. State its question, permitted files, forbidden work, expected evidence, and stopping point. Assign separate files or keep reviewers read-only. The lead integrates all changes and explains important decisions to Nick.

All helpers must respect the learning boundary: no energy-depletion solution or creature behaviors during bootstrap, and no implementation of a later learner-owned task without explicit authorization.

## Helper A — web/build investigator

**Question:** What is the smallest compatible Rust/Bevy web setup in this environment?

Read the architecture and research notes. Verify current official references, existing tools, target features, and a proposed dependency combination. Check actual build constraints rather than relying on older example code.

**Permitted work:** Read-only investigation by default. If explicitly assigned, own only manifests, bundler configuration, or a separate scratch build. Coordinate dependency changes with the lead.

**Return:** Exact findings, sources, commands actually run, observed results, the recommended small setup, and remaining uncertainty. No creature logic, broad engine survey, or public deployment.

## Helper B — correctness and boundary reviewer

**Question:** Does the scaffold preserve the simulation boundary and the promised learning task?

Inspect ticking, pause/step/reset, camera independence, configuration validation, IDs, event retention, and web error handling. Check whether any learner-owned implementation was accidentally included. Review tests for what they actually establish.

**Permitted work:** Read-only by default. Supply concrete findings with file locations and small suggested fixes. Test edits require separate explicit ownership.

**Return:** Correctness issues, evidence and severity, missing verification, and one or two focused tests. Do not redesign the entire architecture.

## Later — targeted thought partner

When a real behavior fails, use one helper to construct a counterexample or compare two small policies on a shared fixture.

Example: “The grazer alternates rest and food-seeking every tick. Compare a pair of start/stop thresholds against a short commitment timer. Do not implement either in Nick's exercise. Return a worked trace and one recommendation.”

Avoid a standing council of agents designing an entire theory of life. Delegate a question that removes friction from the next visible experiment.
