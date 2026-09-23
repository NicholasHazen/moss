# Browser experience — a map you can watch live

**Status:** Pan/zoom/selection, Play/Pause/Step/Reset, hidden-tab suspension and
the HTML inspector are implemented. Follow-selection, speed controls and mobile
support remain future work. [Verification](../development/verification.md) records actual coverage.

## The mental model

The world is the map. Creatures are things in that world, not icons controlled by the camera. Move the view to explore; change simulation controls to affect time.

“Google Maps-style” means familiar pan/zoom behavior. No Google integration, geographic coordinates, map tiles, API key, or backend is required.

## Navigation

Drag the world to pan. Zoom toward the pointer so the place under the cursor remains approximately anchored. Clamp zoom to useful limits and provide a **Fit world** action. A selected creature remains selected while panning.

Treat a click and a drag differently: use a small screen-space movement threshold, not immediate selection on pointer-down. Release drag state when capture is lost or the pointer leaves the interaction unexpectedly. UI panels consume their own input; scrolling an inspector must not zoom the world behind it.

Use wheel/trackpad deltas consistently rather than assuming every device reports the same units. Preserve normal page zoom and scrolling outside the simulation surface. Do not globally disable browser accessibility gestures.

Bevy's official pan-camera and viewport-conversion examples provide starting points, not a complete specification for this interaction. Check the APIs for the chosen release. [R3](../research/README.md#r3-camera-and-coordinate-conversion)

## Selection and inspection

Click a creature to see identity, kind, position, and only the state/behavior actually implemented. Follow-selection is a useful later extension; manual panning should clearly cancel following.

Offer a plain-text entity list or another non-color-only way to identify a selected creature. Shape, outline, or labels should supplement color. Keep text readable independently of map zoom. Selection should refer to a stable simulation ID, so removing a live entity does not select an unrelated new one.

## Playback

Provide Pause/Play, Step, and Reset early. Step remains available while paused and runs exactly one complete tick. Show the tick count and a clear paused/running state. Add a small set of speed controls when ordinary playback is reliable.

Changing playback speed must not change the duration or cost of a simulated action. Simulation catch-up per rendered frame is bounded so interaction remains responsive; under load, report that simulation is running slower rather than silently skipping biological rules.

## Background tabs

The starting policy is **pause on hidden, with no offline catch-up**. Record that playback was suspended, discard the wall-time backlog, and remain paused when the tab returns until the user resumes. Camera focus is not biological state.

Browsers commonly throttle timers and stop animation callbacks in hidden tabs. Use the Page Visibility API rather than assuming a continuously running desktop loop. [R6](../research/README.md#r6-browser-visibility)

## Scope of accessibility and device support

Desktop/laptop browser use with mouse or trackpad is the first verified interaction target. Browser-first does not automatically mean a polished phone interface.

Pointer-friendly input should leave room for touch pan and pinch zoom. Full mobile support is a later explicit slice with real-device tests. Visible buttons and keyboard alternatives are preferable to hidden gesture-only controls.

The controls and inspector use CSS/HTML with a small JavaScript bridge. Keep that shell independent of renderer startup so loading and failures remain visible. A UI framework needs a demonstrated requirement; it must not become a second simulation state model.

## Deployment and graphics

The application should produce statically hostable assets. No account, server-side simulation, analytics service, or paid API is required. Keep assets local to the project; public deployment is a separate authorized action.

Bevy provides browser examples for WebGL2 and WebGPU. Moss currently selects WebGL2; do not promise automatic renderer fallback or universal device support. [R2](../research/README.md#r2-bevy-browser-rendering) Trunk is the local bundler/server. [R4](../research/README.md#r4-trunk)

## A browser smoke check should exercise behavior

Record the actual browser/version and observed result. Check startup, resize, pan versus click, pointer-anchored zoom, fit-world, pause/step/reset, panel input isolation, and hidden-tab return. Unavailable checks stay explicitly unverified.

Repeat a fixed-tick fixture with the camera centered and looking elsewhere. The simulation state should match. Rendering visibility is allowed to change drawing cost, never creature behavior.
