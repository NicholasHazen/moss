# External source link review — September 23, 2026

**Result:** all 23 distinct external URLs found in the reference shelf, editorial
source map, and chapter citations retrieved the named subject. No dead or
off-topic destination was observed. No replacement is recommended from this pass.

**Audit window:** 2026-09-23 **08:53:51–08:55:25 UTC**. The inventory below covers
every external URL in `src/reference/reading-shelf.md` and `editorial/SOURCES.md`,
plus the additional `Option::map` citation in chapter 3. Repeated chapter links
to Rust borrowing, Bevy Query/chain, fixed timesteps and MIT's exercise were
deduplicated against that inventory.

Retrieval used the web tool against the cited primary pages. “Retrieved” means
the tool returned the expected document and relevant text, not that a separate
browser/HTTP status probe was performed. Returned crawl dates ranged from today
to last month. The tool did not expose a complete HTTP redirect chain; it showed
no off-topic destination or changed host for these requests.

## Reader-facing references and chapter citations

| Audited URL | Observed destination and result |
| --- | --- |
| [Rust: References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) | Retrieved the named Rust Book chapter, including mutable references and dereferencing. The chapter 1 citation remains relevant. Living documentation, as labeled in the book. |
| [Rust: What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) | Retrieved the named Rust Book ownership chapter. Correct neighboring destination for the shelf's ownership/copying question. |
| [Iterator::min_by_key](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.min_by_key) | Retrieved `std::iter::Iterator`; located the named method and its equal-minimum/empty-input behavior. Header reports Rust **1.98.1**, and this method reports stability since **1.6.0**, matching the source-map caveat. |
| [Option::unwrap_or](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or) | Retrieved `std::option::Option`; located the named method, fallback examples and eager-evaluation note. Correct destination for absent versus supplied defaults. This is a living std page. |
| [Option::map](https://doc.rust-lang.org/std/option/enum.Option.html#method.map) | Retrieved the same Option page; located `map` and its contained-value/absence behavior. Correct primary citation for chapter 3. |
| [Bevy ECS 0.18.1 Query](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Query.html) | Retrieved the exact **0.18.1** API page. Optional access, disjoint queries, and initialization rejection for incompatible query access are present. No version drift observed. |
| [Bevy 0.18.1 IntoScheduleConfigs::chain](https://docs.rs/bevy/0.18.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html#method.chain) | Retrieved the exact **0.18.1** re-export page; located `chain` and the deferred-application condition. The explicit Bevy re-export URL remains suitable; there is no need to change crate dependencies. |
| [Bevy ECS 0.18.1 Commands](https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/system/struct.Commands.html) | Retrieved the exact **0.18.1** Commands page, including deferred world changes and `ApplyDeferred`. Correct companion to the ordering reference. |
| [Glenn Fiedler: Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/) | Retrieved the original essay, dated **June 10, 2004**, with fixed-step, accumulator and catch-up discussion. Named author/topic/date match. |
| [MDN: Page Visibility API](https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API) | Retrieved the named MDN Web APIs article. Correct browser visibility reference; it remains living documentation rather than Moss-specific runtime evidence. |
| [MIT OCW: Mapping the Stock and Flow Structure of Systems](https://ocw.mit.edu/courses/15-871-introduction-to-system-dynamics-fall-2013/resources/mit15_871f13_ass3/) | Retrieved the resource page for **15.871 Fall 2013, assignment 3**. The title and resource correspond to the shelf's stock/flow invitation. |
| [NetLogo: BehaviorSpace](https://docs.netlogo.org/behaviorspace.html) | Retrieved **Behavior Space — NetLogo 7.0.4 User Manual**. The version matches the source map; the destination is the experiment manual rather than a model runner. |
| [NetLogo Models Library: Wolf Sheep Predation](https://ccl.northwestern.edu/netlogo/models/WolfSheepPredation) | Retrieved the named model page, with both implicit-grass and explicit-grass variants and a link to its Web runner. No model or external application was executed. |
| [Flecs: Component Traits](https://www.flecs.dev/flecs/ComponentTraits.html) | Retrieved the named documentation; located **OnInstantiate** and its Override/Inherit/DontInherit policies. This is the intended living comparison page, not a pinned Bevy API. |
| [Epic: Gameplay Attributes and Attribute Sets, Unreal 5.6](https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-attributes-and-attribute-sets-for-the-gameplay-ability-system-in-unreal-engine?application_version=5.6) | Retrieved the named **Unreal Engine 5.6 Documentation** page with base/current values. The version query is honored in the returned document title. |
| [Factorio: Friday Facts #204](https://www.factorio.com/blog/post/fff-204) | Retrieved **Another day, another optimisation**, dated **August 18, 2017**, by kovarex and Zulan. Its Prefetching section contains the reported layout change without measurable improvement, matching the shelf's limited use. |

## Additional source-map links

| Audited URL | Observed destination and result |
| --- | --- |
| [Rust Book MIT license](https://github.com/rust-lang/book/blob/main/LICENSE-MIT) | Retrieved `rust-lang/book`, `main`, `LICENSE-MIT`; the license body is available. No repository or filename mismatch. |
| [Rust COPYRIGHT](https://github.com/rust-lang/rust/blob/main/COPYRIGHT) | Retrieved `rust-lang/rust`, `main`, `COPYRIGHT`; the stated Apache/MIT and exception context is present. |
| [Bevy v0.18.1 MIT license](https://github.com/bevyengine/bevy/blob/v0.18.1/LICENSE-MIT) | Retrieved `bevyengine/bevy`, tag **v0.18.1**, `LICENSE-MIT`; the license body is available. Exact release destination retained. |
| [MDN attribution and copyright licensing](https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Attrib_copyright_license) | Retrieved the named MDN policy page. Its CC-BY-SA discussion is present; this remains a policy reference, not blanket permission for every asset. |
| [MIT assignment 3 direct PDF](https://ocw.mit.edu/courses/15-871-introduction-to-system-dynamics-fall-2013/b950bd7b4f565b7b19145fb16bd55c8e_MIT15_871F13_ass3.pdf) | Retrieved a **9-page PDF** with the matching assignment title. Text extraction includes stocks, flows, units, and model-boundary questions. No visual PDF audit was performed. |
| [MIT OCW Privacy and Terms of Use](https://ocw.mit.edu/pages/privacy-and-terms-of-use/) | Retrieved the named OCW terms page with Creative Commons licensing sections. No destination mismatch. |
| [NetLogo Copyright and License Information](https://docs.netlogo.org/copyright.html) | Retrieved the **NetLogo 7.0.4** manual page, including its manual-specific CC BY-SA 3.0 statement. Correct distinction from application/model licensing. |

## Findings, replacements and limits

- **Required link replacements: none.** Method fragments were checked against
  the returned API's named method sections; browser scrolling to a fragment was
  not tested in this review.
- **Version caveats remain necessary.** The std pages report Rust 1.98.1 while
  Moss pins 1.93.1. The book already separates living upstream documentation
  from independently checked Moss examples. Bevy references remain pinned to
  0.18.1; Epic's query-selected page remains 5.6; the NetLogo manual remains
  7.0.4.
- The cited source pages stayed on topic. This does not establish availability
  from every network, future link permanence, or operation of their interactive
  runners. No licenses, source material or artwork were imported.
- Root separately identified the official replacement for an old Trunk domain.
  No Trunk URL occurs in this audited source inventory. Starter/setup links and
  the public deployment URL are outside this pass and remain root's checks.
- Only this report was created. No chapter, guide, runtime, build tool,
  publishing configuration or deployment was changed. No builds, native tests,
  browser control or broad crawl was performed.
