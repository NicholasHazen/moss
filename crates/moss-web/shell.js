/* The browser owns input and presentation. Rust owns all simulation state. */
(() => {
  "use strict";

  const byId = (id) => document.getElementById(id);
  const canvas = byId("moss-canvas");
  const controls = ["play", "step", "reset", "fit", "zoom-in", "zoom-out"].map(byId);
  const maxActions = 128;
  let actions = [];
  let hiddenEpoch = 0;
  let pointer = null;
  let ready = false;
  let failed = false;
  let lastInspector = "";
  let lastEntities = "";
  let lastEvents = "";
  let latestSnapshot = null;

  function enqueue(action) {
    if (!ready || failed || document.hidden) return;
    // The queue is bounded even if rendering stalls. Consecutive pan events merge.
    const previous = actions[actions.length - 1];
    if (action.type === "pan" && previous?.type === "pan") {
      previous.dx += action.dx;
      previous.dy += action.dy;
    } else if (actions.length < maxActions) {
      actions.push(action);
    }
  }

  function point(event) {
    const rect = canvas.getBoundingClientRect();
    return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }

  function endPointer() {
    const id = pointer?.id;
    pointer = null;
    canvas.classList.remove("dragging");
    if (id !== undefined && canvas.hasPointerCapture(id)) canvas.releasePointerCapture(id);
  }

  canvas.addEventListener("pointerdown", (event) => {
    if (!ready || failed || event.button !== 0 || !event.isPrimary || pointer) return;
    const current = point(event);
    pointer = { id: event.pointerId, start: current, last: current, dragging: false };
    canvas.setPointerCapture(event.pointerId);
    canvas.focus({ preventScroll: true });
    event.preventDefault();
  });

  canvas.addEventListener("pointermove", (event) => {
    if (!pointer || pointer.id !== event.pointerId) return;
    const current = point(event);
    if (!pointer.dragging && Math.hypot(current.x - pointer.start.x, current.y - pointer.start.y) >= 5) {
      pointer.dragging = true;
      canvas.classList.add("dragging");
      enqueue({ type: "pan", dx: current.x - pointer.start.x, dy: current.y - pointer.start.y });
    } else if (pointer.dragging) {
      enqueue({ type: "pan", dx: current.x - pointer.last.x, dy: current.y - pointer.last.y });
    }
    pointer.last = current;
  });

  canvas.addEventListener("pointerup", (event) => {
    if (!pointer || pointer.id !== event.pointerId) return;
    const current = point(event);
    const isClick = !pointer.dragging && Math.hypot(current.x - pointer.start.x, current.y - pointer.start.y) < 5;
    if (isClick) enqueue({ type: "click", ...current });
    endPointer();
  });
  canvas.addEventListener("pointercancel", endPointer);
  canvas.addEventListener("lostpointercapture", endPointer);
  window.addEventListener("blur", endPointer);

  canvas.addEventListener("wheel", (event) => {
    if (event.ctrlKey || event.metaKey || !ready || failed) return;
    event.preventDefault();
    const modeScale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1;
    const exponent = Math.max(-0.5, Math.min(0.5, event.deltaY * modeScale * 0.0015));
    enqueue({ type: "zoom", ...point(event), factor: Math.exp(exponent) });
  }, { passive: false });

  function zoom(factor) {
    enqueue({ type: "zoom", x: canvas.clientWidth / 2, y: canvas.clientHeight / 2, factor });
  }
  byId("play").addEventListener("click", () => enqueue({ type: "toggle_play" }));
  byId("step").addEventListener("click", () => enqueue({ type: "step" }));
  byId("reset").addEventListener("click", () => enqueue({ type: "reset" }));
  byId("fit").addEventListener("click", () => enqueue({ type: "fit" }));
  byId("zoom-in").addEventListener("click", () => zoom(0.8));
  byId("zoom-out").addEventListener("click", () => zoom(1.25));

  // Shortcuts belong to the canvas only; normal buttons remain keyboard accessible.
  canvas.addEventListener("keydown", (event) => {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    const directions = { ArrowLeft: [40, 0], ArrowRight: [-40, 0], ArrowUp: [0, 40], ArrowDown: [0, -40] };
    if (directions[event.key]) {
      const [dx, dy] = directions[event.key];
      enqueue({ type: "pan", dx, dy });
    } else if (event.key === "+" || event.key === "=") zoom(0.8);
    else if (event.key === "-") zoom(1.25);
    else if (event.key.toLowerCase() === "f") enqueue({ type: "fit" });
    else return;
    event.preventDefault();
  });

  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      hiddenEpoch = (hiddenEpoch + 1) >>> 0;
      actions = [];
      endPointer();
    }
  });

  function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function entityLabel(entity) {
    return `${entity.species}${entity.category === "patch" ? " patch" : ""} #${entity.id}`;
  }

  function selectButton(entity, className = "entity-button") {
    const button = element("button", className, entityLabel(entity));
    button.title = `${entity.name} · ${entity.role}`;
    button.type = "button";
    button.addEventListener("click", () => enqueue({ type: "select", id: entity.id }));
    return button;
  }

  function drawInspector(snapshot) {
    const selected = snapshot.entities.find((entity) => entity.id === snapshot.selected);
    const signature = JSON.stringify(selected ?? null);
    if (signature === lastInspector) return;
    lastInspector = signature;
    const inspector = byId("inspector");
    inspector.replaceChildren();
    if (!selected) {
      inspector.append(element("p", "empty-state", "Choose a creature or food patch to see its starting state."));
      return;
    }
    inspector.append(element("p", "entity-name", entityLabel(selected)));
    inspector.append(element("p", "entity-kind", `${selected.category === "creature" ? "Nicknamed" : "Labeled"} ${selected.name}`));
    const facts = element("dl", "facts");
    const fact = (name, value) => facts.append(element("dt", "", name), element("dd", "", value));
    fact("Identity", `#${selected.id}`);
    fact("Species", selected.species);
    fact("Ecological role", selected.role);
    fact("Position · cells", `${selected.x}, ${selected.y}`);
    if (selected.energy !== null) fact("Energy · units", `${selected.energy} / ${selected.capacity ?? "unknown"}`);
    if (selected.biomass !== null) fact("Biomass · units", String(selected.biomass));
    fact("Behavior", "Not autonomous yet");
    fact("Physical size", "Not modeled");
    inspector.append(facts);
    if (selected.energy !== null && selected.capacity > 0) {
      const track = element("div", "energy-track");
      track.setAttribute("aria-hidden", "true");
      const fill = element("div", "energy-fill");
      fill.style.width = `${Math.max(0, Math.min(100, selected.energy / selected.capacity * 100))}%`;
      track.append(fill);
      inspector.append(track);
    }
  }

  function drawEntities(snapshot) {
    const signature = JSON.stringify(snapshot.entities.map(({ id, name, species, role, category }) => ({ id, name, species, role, category })));
    const list = byId("entity-list");
    if (signature !== lastEntities) {
      lastEntities = signature;
      list.replaceChildren();
      for (const entity of snapshot.entities) {
        const button = selectButton(entity);
        button.dataset.simId = entity.id;
        const kind = ["producer", "hunter", "grazer"].includes(entity.role) ? entity.role : "unknown";
        const symbol = element("span", `entity-symbol ${kind}`);
        symbol.setAttribute("aria-hidden", "true");
        button.prepend(symbol);
        list.append(button);
      }
    }
    // Preserve the focused button when only selection changes.
    for (const button of list.children) {
      button.setAttribute("aria-pressed", String(snapshot.selected === button.dataset.simId));
    }
  }

  function drawEvents(snapshot) {
    const signature = JSON.stringify([snapshot.run, snapshot.events, snapshot.retained, snapshot.evicted, snapshot.event_capacity]);
    if (signature === lastEvents) return;
    lastEvents = signature;
    const list = byId("events");
    list.replaceChildren();
    for (const event of [...snapshot.events].reverse()) {
      const item = element("li");
      item.append(element("span", "event-time", `Tick ${event.tick}`), element("span", "", event.text));
      if (event.participants.length) {
        const participants = element("div", "event-participants");
        for (const id of event.participants) {
          const entity = snapshot.entities.find((candidate) => candidate.id === id);
          if (entity) participants.append(selectButton(entity, "participant"));
          else participants.append(element("span", "annotation", `#${id} · not in current state`));
        }
        item.append(participants);
      }
      list.append(item);
    }
    if (!snapshot.events.length) list.append(element("li", "empty-state", "No events retained for this run."));
    byId("coverage").textContent = `${snapshot.retained} of ${snapshot.event_capacity} event slots used · ${snapshot.evicted} evicted. Memory only; Reset starts a fresh run. Movement, eating, and decisions are not collected.`;
  }

  function fail(message) {
    failed = true;
    actions = [];
    endPointer();
    controls.forEach((control) => { control.disabled = true; });
    byId("startup").hidden = false;
    byId("startup").classList.add("error");
    byId("startup-title").textContent = "Moss could not continue.";
    byId("startup-detail").textContent = `${String(message)} Reload the page after resolving the error. The browser console may have more detail.`;
    byId("playback-state").textContent = "Error";
  }

  globalThis.MossBridge = {
    takeInput() {
      const pending = actions;
      actions = [];
      return JSON.stringify({
        now_ms: performance.now(), hidden: document.hidden, hidden_epoch: hiddenEpoch,
        width: canvas.clientWidth, height: canvas.clientHeight, actions: pending,
      });
    },
    render(snapshotJSON) {
      if (failed) return;
      try {
        const snapshot = JSON.parse(snapshotJSON);
        latestSnapshot = snapshot;
        if (!ready) {
          ready = true;
          controls.forEach((control) => { control.disabled = false; });
          byId("startup").hidden = true;
          const gl = canvas.getContext("webgl2");
          console.info("Moss ready", navigator.userAgent, gl ? gl.getParameter(gl.VERSION) : "WebGL2 context unavailable");
        }
        byId("tick").textContent = snapshot.tick;
        byId("play").textContent = snapshot.running ? "Pause" : "Play";
        byId("play").setAttribute("aria-pressed", String(snapshot.running));
        byId("playback-state").textContent = snapshot.suspended ? "Paused after hiding" : snapshot.slowed ? "Running · limited" : snapshot.running ? "Running" : "Paused";
        byId("run-label").textContent = `Run ${snapshot.run}`;
        byId("dimensions").textContent = `${snapshot.width} × ${snapshot.height} cells`;
        byId("zoom-level").textContent = `${Number(snapshot.zoom).toFixed(1)} px / cell`;
        drawInspector(snapshot);
        drawEntities(snapshot);
        drawEvents(snapshot);
      } catch (error) {
        fail(`The inspection view could not be updated: ${error.message}`);
      }
    },
    fail,
    // A copy of the last presentation snapshot helps browser checks without exposing the ECS world.
    snapshot() { return latestSnapshot ? JSON.parse(JSON.stringify(latestSnapshot)) : null; },
  };

  window.addEventListener("error", (event) => fail(event.message || "A browser startup error occurred."));
  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason;
    fail(reason?.message || String(reason ?? "The WebAssembly module failed to start."));
  });
  window.setTimeout(() => {
    if (ready || failed) return;
    byId("startup-title").textContent = "Still waiting for the workbench…";
    byId("startup-detail").textContent = "The first download or shader setup can take a while. If this continues, check the browser console and confirm the WASM bundle loaded. WebGL2 must be available.";
  }, 30_000);
})();
