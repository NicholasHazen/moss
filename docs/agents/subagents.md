# Bounded helper assignments

Use actual delegation only when supported, preferably no more than two helpers
at once. The lead owns integration, verification and the explanation to Nick.
The [agent contract](../../AGENTS.md) and [coding conventions](coding-style.md)
apply to every helper. Archived bootstrap exercises are not current assignments.

## Shared contract

Every assignment supplies:

- One concrete question or deliverable and its stopping point.
- Relevant source, current worktree context and applicable conventions.
- Permitted files, forbidden work and whether editing is allowed.
- Evidence expected: findings with locations, commands actually run, results
  and remaining limits.

Give editors disjoint files. Keep reviewers read-only unless a narrow edit is
explicitly assigned. Never delegate a learner-owned rule as a workaround for
the paired boundary. Avoid competing full builds against a shared target;
coordinate checks or use separate scratch workspaces and target directories.

## Helper A — web/build investigator

**Question:** what is the smallest verified change needed for the reported
browser or build problem?

Read the current architecture, development guide and version-matched official
references. Inspect installed tools, target features and the dependency graph.
Default to read-only; allow only named manifests or scratch files when needed.
Return concrete evidence, a small recommendation and unverified runtime checks.
No biology, broad engine comparison or public deployment.

## Helper B — correctness and boundary reviewer

**Question:** does this change preserve its promised behavior and ownership?

Read the actual diff and affected tests. Check tick/action order, reset,
camera independence, IDs, bounded history, error handling and the relevant
code conventions. Return actionable findings with locations and severity, or
state that none were found, plus the limits of the review. Do not redesign the
project or edit a learner's implementation while reviewing it.

## Later — targeted thought partner

When behavior reveals a design question, request one counterexample or small
comparison. For example: compare start/stop thresholds with a short commitment
timer for a grazer that switches activity every tick. Return a worked trace and
one recommendation; implement neither unless explicitly assigned.

## Return format

State the result first, then changed files or findings, actual checks and
remaining uncertainty. Identify proposed code as proposed. The lead confirms
integration and updates the current documentation; helpers do not independently
change `NOW.md` or activate later features.
