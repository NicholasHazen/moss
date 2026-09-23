# RustRover analysis incident — September 21, 2026

Historical diagnostics from build 263.5153.48. Current workflow and limitations
are in the [RustRover guide](../development/rustrover.md).

## False errors and stalled analysis — September 21, 2026

On build **263.5153.48**, `lessons.rs` showed red errors on valid
`SpeciesEnergyRules` and query code while analysis remained at “Analyzing…”.
The same source passed all seven native simulation tests. The IDE log repeatedly
reported `NoClassDefFoundError` from `EduHtmlPsiPatterns`, caused by missing
`com.intellij.patterns.XmlPatterns`, and explicitly blamed **JetBrains Academy
2026.8-2026.2-396**.

Disable that Academy version under **Settings → Plugins → Installed**, then
restart RustRover. This is reversible and does not remove the Markdown tutorial
or Rust language support. After the restart in this session, `lessons.rs` had a
green analysis checkmark, the red underlines disappeared, and Problems reported
“No problems in lessons.rs”. Cargo Check on-the-fly remained enabled. The fresh
startup log confirmed Academy was disabled and had no recurrence of its analysis
exception during the verification window. Long-term recurrence is not yet tested.
The IDE's **Moss - Maintenance test** also passed after restarting; the terminal
simulation suite passed all seven tests. The formatter corrected one trailing
comma, without changing the rule.

Keep RustRover for now. If false errors return, compare the exact diagnostic
with Cargo and check the current IDE log before changing correct source or
disabling inspections. A Cargo reload is appropriate for a stale project model;
it does not repair a crashing plugin. Only consider another editor if the
remaining problem persists with a healthy analysis pass.

JetBrains documents [disabling plugins](https://www.jetbrains.com/help/rust/managing-plugins.html#disable-plugins)
and [reloading Cargo projects](https://www.jetbrains.com/help/rust/loading-cargo-projects.html).
The evidence above is from the local IDE, not a claim that those docs identify
this specific Academy bug. Native Debug and the new rule's browser acceptance
remain separately unverified.
