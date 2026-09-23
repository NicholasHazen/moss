# Independent teaching-model review

Reviewed 2026-09-23. No simulation-rule correctness findings in the five
JavaScript teaching models within their documented, bounded UI inputs. One
accessibility refinement is recommended before publication.

## Finding

**P2 — Give enabled numeric inputs a stronger visible boundary.**
`book/theme/moss.css:92` uses the decorative `--moss-border` token for input
and select borders. Its contrast against the surrounding lab surface is
approximately **1.50:1 in light themes** (`#cbd5c6` / `#fffef9`) and **2.10:1
in dark themes** (`#46614e` / `#1d2e24`). The input fill is also close to the
surrounding surface, so an unfocused numeric field can become hard to
distinguish from a displayed value. Use a separate control-border token that
reaches 3:1 against the surrounding surface, or use the existing accent token
for these controls. Decorative card and section borders can retain their
current colors. This recommendation follows W3C's
[non-text contrast guidance for input boundaries](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html).
The measurements are calculated from the authored CSS colors, not a screen
reader or visual-perception test.

## What was checked

- **Movement:** copied proposal, x-before-y cardinal step, invalid target
  rejection, exact-price acceptance, actual-distance charging, and unchanged
  caller state before the final phase agree with the worked Rust helper in
  `docs/tutorial/today-v2.md`. Invalid input disables advancement; Reset restarts
  the selected inputs rather than silently restoring the default numbers.
- **Shared meal:** both input orders resolve IDs 1 then 7 and read the current
  remainder. The five-biomass account is 4 / 1 / 0. The model displays a
  zero-transfer resolution even though the Rust reference's event vector omits
  zero meals; it does not label these UI turns as Rust events or claim byte-for-byte
  execution equivalence. Eligibility, capacity, bite limit and conversion are
  stated beside the controls.
- **Tick phases:** maintenance precedes the 45/75 transition, target changes are
  visible before action, same-cell travel costs zero, and eating does not rerun
  choice. The 74 and 72 starting traces agree with session 04. Finishing a partial
  tick does not repeat maintenance. The retained phase trace stays at five rows.
- **Defaults:** draft and active configuration are distinct. Runtime edits touch
  owned values only. Reset reconstructs from the active template while preserving
  the unsubmitted draft; Start configured run commits that draft. `None`,
  `Some(1)` and `Some(0)` retain separate authored meanings even when resulting
  costs match. This is labelled a selected future policy, not existing live wiring.
- **Daylight:** executing tick, completed tick and phase remain distinct. Tick
  120 is night; tick 240 is daylight. Bounded growth precedes upkeep and the
  meal. Stored food remains edible at night; requests and actual transfers are
  separate. Boundary advancement executes every intervening tick. The lab
  explicitly omits choice, travel and death and therefore honestly retains a
  zero-reserve animal that can eat at dawn.
- **Static accessibility:** each control has a wrapping label; buttons are native
  buttons; lab regions have names; action explanations are polite status regions;
  the tick table has a caption and column headers. Controls start disabled without
  JavaScript, and adjacent prose/static traces retain the lesson. Meaning is
  expressed in text and numbers rather than color alone. Generated scripts occur
  after chapter markup, so movement's immediate setup has a parsed lab to find.
- **Scope claims:** all five labs identify themselves as teaching illustrations,
  and their descriptions distinguish them from live Rust execution. None supplies
  evidence that a future feature is installed in Moss.

## Checks actually run

`node --test book/scripts/test_*lab*.cjs` passed **27 tests, zero failures**.
An additional read-only inline Node audit checked:

- 4,444 movement combinations: four displayed cases × reserves 0–100 × rates
  0–10, checking actual position distance, charged cost and bounded reserves.
- 12,000 daylight combinations: completed ticks 0–479 × five reserve values ×
  five biomass values, checking store conservation and capacity bounds.
- Four tick presets for 1,000 ticks each, checking completed counts, store bounds
  and bounded five-row traces.

All passed. These extra checks were ephemeral reviewer probes, not new committed
tests or a formal proof of the Rust implementation. The references read were
the movement helper and sessions 01–04, 06–07 and 09–12, with the chapter-specific
prose explaining the additional configuration example.

## Limits

This was a read-only code and markup audit. I did not operate a browser, run a
screen reader, measure rendered target sizes or inspect responsive layouts.
Keyboard operation, spoken status announcements and actual color rendering need
the lead's browser/accessibility checks. I did not build Rust, WASM or the book,
and did not verify deployment. The pure JavaScript functions are scoped teaching
models, not general-purpose APIs; movement and tick transitions do not validate
arbitrary externally constructed states. Their displayed controls restrict the
inputs reviewed here. New diagrams added after this review are outside this
report's visual scope.

Only this editorial report was written. No chapter, guide, production code or
learner-owned exercise was changed.
