(function () {
  "use strict";
  const KEY = "moss-fieldnotes-v1";
  const page = document.body.dataset.page;
  const { empty, validate, mergeImported, reconcile } = window.MossRecords;
  let record = empty();
  let baseRecord = empty();
  let storageAvailable = true;
  let recoveryBytes = null;
  let storageBlocked = false;
  try {
    const saved = localStorage.getItem(KEY);
    if (saved) {
      try { record = validate(JSON.parse(saved)); baseRecord = validate(record); }
      catch { recoveryBytes = saved; storageBlocked = true; storageAvailable = false; }
    }
  } catch { storageAvailable = false; }
  const status = document.getElementById("save-status");
  const dataStatus = document.getElementById("data-status");
  let saveQueue = Promise.resolve();
  let pendingSaves = 0;
  let unsavedChanges = false;
  function save() {
    pendingSaves++; unsavedChanges = true;
    const persist = () => {
      if (storageBlocked) throw new Error("Existing saved data could not be read and has not been overwritten. Download it for recovery; export this session separately.");
      const bytes = localStorage.getItem(KEY);
      const latest = bytes ? validate(JSON.parse(bytes)) : empty();
      const merged = reconcile(baseRecord, record, latest);
      localStorage.setItem(KEY, JSON.stringify(merged));
      record = merged; baseRecord = validate(merged); storageAvailable = true;
      unsavedChanges = false;
      sync();
      if (status) status.textContent = "Saved in this browser.";
      return true;
    };
    saveQueue = saveQueue.then(() => navigator.locks ? navigator.locks.request(KEY, persist) : persist()).catch(error => {
      storageAvailable = false;
      const message = `${error.message} Export this session’s record to keep it.`;
      if (status) status.textContent = message; else dataStatus.textContent = message;
      return false;
    }).finally(() => { pendingSaves--; });
    return saveQueue;
  }
  function drawProgress() {
    document.querySelectorAll("[data-progress-for]").forEach(dot => {
      const state = record.lessons[dot.dataset.progressFor]?.state || "new";
      dot.dataset.state = state;
      dot.parentElement.title = state === "new" ? "Not started" : state;
    });
  }
  const stateControl = document.getElementById("lesson-state");
  const note = document.getElementById("lesson-note");
  const clips = document.getElementById("clippings");
  function lesson() { return record.lessons[page] ||= { state: "new", note: "", clips: [] }; }
  function drawClips() {
    if (!clips) return;
    clips.replaceChildren();
    lesson().clips.forEach((text, index) => {
      const quote = document.createElement("blockquote");
      const p = document.createElement("p"); p.textContent = text;
      const remove = document.createElement("button"); remove.type = "button"; remove.textContent = "Remove passage";
      remove.addEventListener("click", () => { lesson().clips.splice(index, 1); save(); drawClips(); document.getElementById("save-selection").focus(); });
      quote.append(p, remove); clips.append(quote);
    });
  }
  function sync() {
    document.getElementById("text-size").value = record.settings.textSize;
    document.documentElement.style.setProperty("--reading-size", {normal:"1.1rem",large:"1.25rem",larger:"1.4rem"}[record.settings.textSize]);
    if (stateControl) {
      stateControl.value = lesson().state;
      if (note.value !== lesson().note) {
        const start = note.selectionStart, end = note.selectionEnd;
        note.value = lesson().note;
        if (document.activeElement === note) note.setSelectionRange(start, end);
      }
      drawClips();
    }
    drawProgress();
  }
  if (stateControl) {
    stateControl.addEventListener("change", () => { lesson().state = stateControl.value; save(); });
    note.maxLength = 40000;
    note.addEventListener("input", () => { lesson().note = note.value; save(); });
    document.getElementById("save-selection").addEventListener("click", () => {
      const selection = window.getSelection();
      const article = document.querySelector("article");
      const text = selection?.toString().trim();
      if (!text || !article.contains(selection.anchorNode) || !article.contains(selection.focusNode)) { status.textContent = "Select a passage in the lesson, then keep it here."; return; }
      if (text.length > 5000 || lesson().clips.length >= 100) { status.textContent = "Save up to 100 passages of 5,000 characters each per lesson."; return; }
      lesson().clips.push(text); save(); drawClips();
    });
    if (!storageAvailable) status.textContent = "Saved data could not be loaded. Export your session before leaving.";
  }
  document.querySelectorAll(".reading-toolbar,.data-controls,.search-control,.practice-state").forEach(node => { node.hidden = false; });
  if (!navigator.locks) dataStatus.textContent = "This browser cannot coordinate simultaneous saves. Keep one Fieldnotes tab open when editing notes.";
  window.addEventListener("storage", event => {
    if (event.key !== KEY || event.storageArea !== localStorage || storageBlocked) return;
    try {
      const bytes = localStorage.getItem(KEY);
      const latest = bytes ? validate(JSON.parse(bytes)) : empty();
      record = reconcile(baseRecord, record, latest); baseRecord = latest; sync();
    } catch { dataStatus.textContent = "Another tab saved data this tab could not merge. Export this session before reloading."; }
  });
  window.addEventListener("beforeunload", event => {
    if (pendingSaves || unsavedChanges) { event.preventDefault(); event.returnValue = ""; }
  });
  document.getElementById("text-size").addEventListener("change", event => { record.settings.textSize = event.target.value; sync(); save(); });
  document.getElementById("focus-mode").addEventListener("click", event => {
    const on = document.body.classList.toggle("focus-mode"); event.target.setAttribute("aria-pressed", String(on)); event.target.textContent = on ? "Leave focus" : "Focus";
  });
  function downloadRecord(bytes, name) {
    const blob = new Blob([bytes], { type: "application/json" });
    const url = URL.createObjectURL(blob); const a = document.createElement("a");
    a.href = url; a.download = name; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  document.getElementById("export-progress").addEventListener("click", async () => {
    await saveQueue;
    downloadRecord(JSON.stringify(record, null, 2), "moss-fieldnotes-reading-record.json");
    dataStatus.textContent = "Reading record exported.";
  });
  if (recoveryBytes !== null) {
    const recovery = document.getElementById("export-recovery"); recovery.hidden = false;
    recovery.addEventListener("click", () => downloadRecord(recoveryBytes, "moss-fieldnotes-unreadable-backup.json"));
    dataStatus.textContent = "Existing saved data could not be read. It has been preserved. Download it before any recovery work.";
  }
  async function importBytes(bytes) {
    try {
      const incoming = JSON.parse(bytes);
      record = mergeImported(record, incoming); sync();
      const stored = await save();
      dataStatus.textContent = stored ? "Imported and merged with your existing reading record." : "Imported into this session, but browser storage is unavailable. Export to keep your merged record.";
      return true;
    } catch (error) { dataStatus.textContent = `Import failed: ${error.message}`; return false; }
  }
  document.getElementById("import-progress").addEventListener("change", async event => {
    const file = event.target.files[0]; if (!file) return;
    try { await importBytes(await file.text()); }
    catch (error) { dataStatus.textContent = `Import failed: ${error.message}`; }
    event.target.value = "";
  });
  document.getElementById("copy-record").addEventListener("click", async () => {
    await saveQueue;
    try { await navigator.clipboard.writeText(JSON.stringify(record, null, 2)); dataStatus.textContent = "Reading record copied."; }
    catch { dataStatus.textContent = "Clipboard unavailable. Use Export reading record to download it."; }
  });
  document.getElementById("import-pasted").addEventListener("click", async () => {
    const input = document.getElementById("paste-record");
    if (await importBytes(input.value)) input.value = "";
  });
  let indexPromise;
  let searchVersion = 0;
  document.getElementById("search").addEventListener("input", async event => {
    const version = ++searchVersion;
    const query = event.target.value.trim().toLowerCase(); const results = document.getElementById("search-results");
    results.replaceChildren(); if (query.length < 2) return;
    try {
      indexPromise ||= fetch("assets/search.json").then(response => { if (!response.ok) throw new Error("search index unavailable"); return response.json(); });
      const index = await indexPromise; if (version !== searchVersion) return;
      const terms = query.split(/\s+/);
      const found = index.filter(item => terms.every(term => `${item.title} ${item.text}`.toLowerCase().includes(term))).slice(0, 8);
      if (!found.length) results.textContent = "No matching lesson. Try a broader term.";
      for (const item of found) { const a = document.createElement("a"); a.href = `${item.id}.html`; a.textContent = item.title; results.append(a); }
    } catch { if (version === searchVersion) results.textContent = "Search needs the local web server. The learning path links still work."; indexPromise = null; }
  });
  document.querySelectorAll("article pre").forEach(pre => {
    const code = pre.querySelector("code"); if (!code) return;
    const button = document.createElement("button"); button.type = "button"; button.className = "copy-code"; button.textContent = "Copy"; button.setAttribute("aria-label", "Copy code block");
    button.addEventListener("click", async () => {
      try { await navigator.clipboard.writeText(code.textContent); button.textContent = "Copied"; }
      catch { const range = document.createRange(); range.selectNodeContents(code); const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range); button.textContent = "Selected — copy manually"; }
    }); pre.prepend(button);
  });
  document.querySelectorAll(".video-supplement[data-video]").forEach(panel => {
    const id = panel.dataset.video;
    if (!/^[A-Za-z0-9_-]{11}$/.test(id)) return;
    const button = document.createElement("button"); button.type = "button"; button.textContent = "Load video here";
    const message = document.createElement("p"); message.className = "quiet"; message.textContent = "Loads a publisher-hosted YouTube player. If it is unavailable, use the original link or the text route below.";
    panel.append(button, message);
    button.addEventListener("click", () => {
      const frame = document.createElement("iframe");
      frame.src = `https://www.youtube-nocookie.com/embed/${id}?start=${Number(panel.dataset.videoStart) || 0}`;
      frame.title = panel.dataset.videoTitle || "Supplemental video";
      frame.allow = "encrypted-media; fullscreen; picture-in-picture";
      frame.referrerPolicy = "strict-origin-when-cross-origin";
      frame.allowFullscreen = true;
      panel.append(frame); button.remove();
    });
  });
  function revealAnchor() {
    let target; try { target = document.getElementById(decodeURIComponent(location.hash.slice(1))); } catch { return; }
    if (!target) return;
    let parent = target.parentElement;
    while (parent) { if (parent.tagName === "DETAILS") parent.open = true; parent = parent.parentElement; }
  }
  window.addEventListener("hashchange", revealAnchor); revealAnchor();
  document.getElementById("print-lesson").addEventListener("click", () => window.print());
  let printedDetails = null;
  window.addEventListener("beforeprint", () => {
    if (printedDetails !== null) return;
    printedDetails = [...document.querySelectorAll("article details:not([open])")];
    printedDetails.forEach(node => { node.open = true; });
  });
  window.addEventListener("afterprint", () => {
    if (printedDetails === null) return;
    printedDetails.forEach(node => { node.open = false; });
    printedDetails = null;
  });
  const listen = document.getElementById("listen");
  const pause = document.getElementById("pause-reading");
  const stop = document.getElementById("stop-reading");
  const speechStatus = document.getElementById("speech-status");
  let generation = 0;
  let speakingNode;
  function clearHighlight() { speakingNode?.classList.remove("reading-active"); speakingNode = null; }
  function stopReading() {
    generation++; window.speechSynthesis?.cancel(); clearHighlight();
    pause.hidden = true; stop.hidden = true; listen.disabled = false; pause.textContent = "Pause";
  }
  if (!("speechSynthesis" in window)) { listen.disabled = true; listen.title = "Browser speech is unavailable. Use your preferred read-aloud extension or OS reader."; }
  else {
    listen.addEventListener("click", () => {
      stopReading(); const session = generation;
      const blocks = [...document.querySelectorAll("article h1,article h2,article h3,article p,article li,article figcaption,article table")].filter(node => !node.closest(".experiment,.narration-player,pre") && !node.classList.contains("eyebrow") && !node.querySelector("p") && node.getClientRects().length && !node.closest("details:not([open])"));
      let i = 0; listen.disabled = true; pause.hidden = false; stop.hidden = false;
      function next() {
        if (session !== generation) return;
        clearHighlight();
        if (i >= blocks.length) { stopReading(); speechStatus.textContent = "Finished reading this lesson."; return; }
        speakingNode = blocks[i++]; speakingNode.classList.add("reading-active");
        speakingNode.scrollIntoView({ block: "center", behavior: "auto" });
        let passage = speakingNode.textContent;
        if (speakingNode.tagName === "TABLE") {
          const headers = [...speakingNode.querySelectorAll("thead th")].map(cell => cell.textContent.trim());
          passage = (speakingNode.querySelector("caption")?.textContent || "Table") + ". " + [...speakingNode.querySelectorAll("tbody tr")].map(row => [...row.cells].map((cell, index) => `${headers[index] || "Column " + (index + 1)}: ${cell.textContent.trim()}`).join(". ")).join(". ");
        }
        const utterance = new SpeechSynthesisUtterance(passage); utterance.lang = "en-US";
        let started = false;
        utterance.onstart = () => { started = true; };
        utterance.onend = () => { if (session !== generation) return; if (started) next(); else { stopReading(); speechStatus.textContent = "The browser did not start speaking. Try your external reader or another installed voice."; } };
        utterance.onerror = event => { if (session === generation && !["interrupted", "canceled"].includes(event.error)) { stopReading(); speechStatus.textContent = "Browser speech stopped. You can restart or use an external reader."; } };
        speechStatus.textContent = `Reading passage ${i} of ${blocks.length}. Code blocks are skipped.`;
        window.speechSynthesis.speak(utterance);
      }
      next();
    });
    pause.addEventListener("click", () => { const paused = window.speechSynthesis.paused; if (paused) window.speechSynthesis.resume(); else window.speechSynthesis.pause(); pause.textContent = paused ? "Pause" : "Resume"; });
    stop.addEventListener("click", () => { stopReading(); speechStatus.textContent = "Reading stopped."; });
    window.addEventListener("pagehide", stopReading);
  }
  sync();
})();
