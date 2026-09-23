# Moss documentation

Returning to work: read [NOW.md](../NOW.md). Running the app:
[development guide](development/README.md). Making the next paired edit:
[tutorial](tutorial/README.md).

## Find the right document

| Area | Start here |
| --- | --- |
| Vision and scope | [Project brief](../PROJECT_BRIEF.md) |
| Design | [Architecture and code map](design/architecture.md), [ecology model](design/ecology.md) |
| Browser and inspection | [Interaction design](design/browser-experience.md), [observability](design/observability.md) |
| Development | [Commands and setup](development/README.md), [RustRover](development/rustrover.md), [verified coverage](development/verification.md) |
| Learning | [Today's coding route](tutorial/README.md), [16 short sessions for the coming weeks](tutorial/path/README.md), [optional context](tutorial/context/README.md) |
| Agent work | [Agent contract](../AGENTS.md), [coding conventions](agents/coding-style.md), [collaboration](agents/collaboration.md), [helper assignments](agents/subagents.md) |
| Reasons and possibilities | [Decisions](design/decisions.md), [parking lot](design/parking-lot.md), [primary-source research](research/README.md) |
| Earlier work | [Dated build, session and tutorial records](history/README.md) |

For a fresh agent session, use the short [continuation prompt](../prompts/CONTINUE.md).
Bootstrap instructions are archived because the foundation is complete.

## Document ownership

Each fact has one primary home; other pages summarize it briefly and link there.

| Document | Owns |
| --- | --- |
| `NOW.md` | One active task, next edit, stopping point and runnable checkpoint. |
| `README.md` / this index | Project introduction and navigation. |
| `development/README.md` | Setup, run, build and check commands. |
| `development/verification.md` | Latest tested stack, coverage and known limits. |
| `design/architecture.md` | Code map, technical boundaries and system ownership. |
| `design/ecology.md` | Selected biological model and clearly labeled future behavior. |
| `agents/coding-style.md` | Rust, ECS, browser and test conventions used in this codebase. |
| `tutorial/` | Today's worked session; `path/` owns the future sequence, with older chapters and `context/` as companions. |
| `design/decisions.md` / `research/` | Why a choice was made and the evidence consulted. |
| `history/` | Dated outcomes, superseded proposals and original bootstrap material. |

Keep operating instructions out of historical logs. Mark a new design proposal
as proposed; a successful compile is not a browser observation. Older dated
records describe their checkpoint, even when later work supersedes them.

## Maintenance

Use descriptive filenames in the existing area. Split a document when it gains
a separate audience or responsibility; keep connected explanations together.
Prefer short paragraphs and relative Markdown links. Keep tutorial filenames
and heading anchors stable; when other documents move, update incoming links,
quoted paths, code comments and prompts in the same change.

Update the canonical owner first, then affected summaries. Verify local links,
heading anchors and code fences using the [document checks](tutorial/authoring/README.md#run-the-document-checks).
Record changed executable examples under the
[tutorial verification contract](tutorial/authoring/README.md#write-and-verify-a-checkpoint).
A link-only change does not need a new runtime test claim.

Append brief session entries in `history/sessions/YYYY-MM-DD.md`: the request,
what changed, checks actually run and their result, limitations, and the next
step. Put full build evidence in `history/builds/` and update the coverage
summary. Preserve earlier observations instead of rewriting them as current.
