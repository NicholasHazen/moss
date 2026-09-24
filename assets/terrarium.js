/* Presentation for a precompiled Rust moss-sim instance. No simulated tick here. */
(async function () {
  "use strict";
  const panel = document.querySelector("[data-rust-terrarium]");
  if (!panel) return;
  const status = panel.querySelector("[data-runtime-status]");
  try {
    const [metadataResponse, wasmResponse] = await Promise.all([
      fetch("runtime/build.json", { cache: "no-store" }),
      fetch("runtime/moss.wasm", { cache: "no-store" })
    ]);
    if (!metadataResponse.ok || !wasmResponse.ok) throw new Error("missing build");
    const metadata = await metadataResponse.json();
    if (metadata.api !== 1) throw new Error("unsupported runtime API");
    const { instance } = await WebAssembly.instantiate(await wasmResponse.arrayBuffer(), {});
    const api = instance.exports;
    const value = (id, field) => api.moss_fieldnotes_value(id, field);
    const display = number => number < 0 ? "Not present" : String(number);
    const names = ["Fern", "Flint", "Meadow"];
    const render = () => {
      for (let id = 1; id <= 3; id++) {
        const row = panel.querySelector(`[data-row="${id}"]`);
        const x = value(id, 0), y = value(id, 1);
        row.querySelector("[data-position]").textContent = x < 0 || y < 0 ? "Not present" : `(${x}, ${y})`;
        row.querySelector("[data-reserve]").textContent = display(value(id, 2));
        row.querySelector("[data-biomass]").textContent = display(value(id, 3));
        const actor = panel.querySelector(`[data-actor="${id}"]`);
        actor.hidden = x < 0 || y < 0;
        actor.style.left = `${(x + 0.5) / 32 * 100}%`;
        actor.style.bottom = `${(y + 0.5) / 20 * 100}%`;
        actor.textContent = `${names[id - 1]} · ${id === 3 ? value(id, 3) : value(id, 2)}`;
      }
      status.textContent = `Rust runtime ready. Run ${api.moss_fieldnotes_run()}, completed ticks ${api.moss_fieldnotes_tick()}. Fern ${value(1, 2)}, Flint ${value(2, 2)}, Meadow ${value(3, 3)}.`;
    };
    const action = work => {
      try { work(); render(); }
      catch (_) {
        status.textContent = "The Rust runtime stopped. Reload this page to start a new instance.";
        panel.querySelectorAll("button,input").forEach(control => { control.disabled = true; });
      }
    };
    panel.querySelector("[data-runtime-step]").addEventListener("click", () => action(() => api.moss_fieldnotes_step()));
    panel.querySelector("[data-runtime-ten]").addEventListener("click", () => action(() => { for (let i = 0; i < 10; i++) api.moss_fieldnotes_step(); }));
    panel.querySelector("[data-runtime-reset]").addEventListener("click", () => action(() => api.moss_fieldnotes_reset()));
    panel.querySelector("[data-runtime-redraw]").addEventListener("click", () => action(() => {}));
    panel.querySelector("#runtime-scale").addEventListener("input", event => {
      panel.querySelector(".rust-world").style.fontSize = `${event.target.value}%`;
      panel.querySelector("#runtime-scale-value").textContent = `${event.target.value}%`;
      action(() => {});
    });
    panel.querySelectorAll("button,input").forEach(control => { control.disabled = false; });
    panel.querySelector("[data-runtime-version]").textContent = `Built with Rust ${metadata.rust}, Bevy ECS ${metadata.bevyEcs}. Source fingerprint ${metadata.sourceHash.slice(0, 12)}.`;
    render();
  } catch (_) {
    status.textContent = "Rust preview unavailable. Build the optional runtime using the command below, rebuild the course, then reload. The lesson and expected trace remain available.";
  }
})();
