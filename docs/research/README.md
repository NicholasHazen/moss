# Research and technical references

**Checked:** September 20, 2026 for R1–R11; September 21 for R12–R13.

**Purpose:** Support technical decisions with primary sources.

**Evidence boundary:** These references do not establish Moss build success or
performance. Actual project verification is recorded in [build notes](../development/verification.md).

The product vision and learning approach are Nick's requirements and the kit's proposals. They are not claims drawn from these references. Consult version-matched documentation again when implementing; a `latest` documentation link is not a dependency pin.

[Teaching small changes in Moss](learning-design.md) records the September 22
learning-design research: worked examples with practice, functional labels, and
Rust ownership explanations. It separates the studies' results from our teaching
choices and their still-unmeasured fit for Nick.

## R1. Bevy ECS

[Official ECS quick start](https://bevy.org/learn/quick-start/getting-started/ecs/)

Bevy's introduction uses ordinary Rust structs for components, functions for systems, and queries for selecting data. It also demonstrates explicit system ordering with chained systems. This supports a direct teaching approach without a custom ECS wrapper. It does not supply Moss's tick semantics or conflict policy.

## R2. Bevy browser rendering

[Official WebGL2 examples](https://bevy.org/examples/)  
[Official WebGPU examples](https://bevy.org/examples-webgpu/)  
[MDN WebGPU API and compatibility](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API)

Bevy publishes both browser example sets. MDN describes WebGPU's availability and secure-context requirements. Moss uses the validated WebGL2 2D path; that is a project choice, not proof of universal support or automatic fallback.

Check the actual browser/OS/graphics environment. Do not copy an old support warning from an example page as a current browser matrix. Record the tested backend and feature set in build notes.

## R3. Camera and coordinate conversion

[Official Pan Camera example](https://bevy.org/examples/camera/pan-camera-controller/)  
[Official 2D Viewport To World example](https://bevy.org/examples/2d-rendering/2d-viewport-to-world/)

The examples cover camera control and coordinate conversion. They are useful implementation references for a map-like view. Moss implements drag/click separation, cursor anchoring, panel input isolation and keyboard alternatives. The coverage summary records which interactions have been exercised. APIs and feature flags must match the selected release.

## R4. Trunk

[Project site](https://trunk-rs.github.io/trunk/)  
[Installation guide](https://trunk-rs.github.io/trunk/guide/getting-started/installation.html)

Trunk builds and bundles Rust WebAssembly applications from an HTML entry point and produces assets that can be served on the web. Use the maintained project site linked here for installation references. Verify the actual Bevy integration, root/subpath asset behavior, and dev/release commands in the local workspace.

## R5. WebAssembly target

[Rust platform support: wasm32-unknown-unknown](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html)

This target is commonly used for web/JavaScript environments. Its standard library is not equivalent to a desktop operating system: the documentation specifically notes limitations involving filesystem operations, ordinary thread spawning, and printing. Browser-specific logging and storage belong in the web layer. Do not build core rules around native filesystem or thread assumptions.

## R6. Browser visibility

[MDN Page Visibility API](https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API)

Browsers can throttle inactive timers and stop animation callbacks for hidden pages. Visibility events let the application respond explicitly. Moss's implemented response is to pause and discard wall-time backlog; the API itself does not implement that policy.

## R7. Browser storage

[MDN storage quotas and eviction](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria)

Browser-managed data is subject to storage policies, quotas, and possible eviction; private browsing can also change retention. IndexedDB is one browser storage option. Moss has bounded in-memory history. Export and durable storage remain future work; browser storage is not an implied permanent archive.

## R8. Deferred ECS commands

[bevy_ecs Commands documentation](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/struct.Commands.html)

Queued structural changes are applied through deferred processing. Consumption and lifecycle resolution therefore need explicit eligibility rules; queuing a despawn is not itself a guarantee that a later attempt in the current pass sees the target as gone. Use version-matched APIs when implementing the actual resolver.

## R9. Numeric IDs in browser exports

[MDN Number.MAX_SAFE_INTEGER](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER)

JavaScript numbers do not exactly represent every large integer. Moss serializes IDs and tick counters as decimal strings in its browser snapshots. This preserves exact integer identity across the Rust/JavaScript boundary.

## R10. Rust learning reference

[The Rust Programming Language](https://doc.rust-lang.org/book/)

Use the book as a focused reference when a current edit raises a question about ownership, structs, enums, collections, or tests. Completing it before beginning Moss is not a project prerequisite.

## R11. Bevy setup reference

[Official setup guide](https://bevy.org/learn/quick-start/getting-started/setup/)

Check current installation and development guidance before selecting tools. Keep native simulation tests, WebAssembly compilation, and actual browser execution distinct in the evidence recorded.

## R12. Attributes, shared defaults and individual overrides

[Practical attribute and energy design](attributes-and-energy.md)

Compares Flecs copy/inherit policies, Unity instance/shared storage, Veloren's
Rust energy and buff implementation, Unreal's base/effective distinction, and
Factorio's measured optimization experience. Separates engine contracts from
production evidence and Moss-specific recommendations. The [current work card](../../NOW.md) selects the next edit. This research adds
no runtime implementation or performance claim.

## R13. Large property sets beyond games

[Property bags and defaults](property-bags-and-defaults.md)

Broadens the comparison to WPF, CSS, OpenUSD, Figment and Typesafe Config.
Examines sparse authored values, explicit precedence, missing versus explicit
values, provenance, typed consumer views and cache invalidation. These are
documented contracts; no performance comparison or new framework implementation
was performed.

## What the bootstrap agent must record

This recording rule now applies to ongoing development as well as bootstrap.

Record new evidence in `docs/history/builds/` and update the [coverage summary](../development/verification.md) after doing the work. Include the exact Rust toolchain, Bevy/Bevy ECS and relevant library versions, Trunk version, selected features/backend, operating system, actual browser coverage, commands run with working directories, results, and limitations.

Do not infer a successful application from a source link, successful dependency resolution, or a native-only compile. Keep build/dependency work out of Nick's way, but keep its evidence available.
