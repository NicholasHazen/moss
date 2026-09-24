/* Pure reading-record operations. A failed merge never changes an input. */
(function (root, factory) {
  "use strict";
  const records = factory();
  if (typeof module !== "undefined" && module.exports) module.exports = records;
  else root.MossRecords = records;
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  "use strict";
  const STATES = ["new", "reading", "practicing", "reviewed"];
  const SIZES = ["normal", "large", "larger"];
  const SEPARATOR = "\n\n— Imported note —\n\n";
  const isObject = value => value !== null && typeof value === "object" && !Array.isArray(value);
  const empty = () => ({ version: 1, lessons: {}, settings: { textSize: "normal" } });
  const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);

  function validate(raw) {
    if (!isObject(raw) || raw.version !== 1 || !isObject(raw.lessons)) {
      throw new Error("This is not a Fieldnotes version 1 reading record.");
    }
    const clean = empty();
    for (const [id, item] of Object.entries(raw.lessons)) {
      if ((!/^[0-9]{2}-[a-z0-9-]+$/.test(id) && !["fieldwork", "returns", "population", "mobile", "resting", "hunting", "refuge"].includes(id)) || !isObject(item) ||
          !STATES.includes(item.state) || typeof item.note !== "string") {
        throw new Error(`The reading record contains an invalid lesson: ${id}.`);
      }
      const clips = item.clips === undefined ? [] : item.clips;
      if (!Array.isArray(clips) || clips.some(clip => typeof clip !== "string")) {
        throw new Error(`The reading record contains invalid saved passages: ${id}.`);
      }
      if (item.note.length > 40000 || clips.length > 100 || clips.some(clip => clip.length > 5000)) {
        throw new Error(`The reading record exceeds the note or passage limit for ${id}. Nothing was discarded.`);
      }
      clean.lessons[id] = { state: item.state, note: item.note, clips: [...clips] };
    }
    if (raw.settings !== undefined) {
      if (!isObject(raw.settings) || !SIZES.includes(raw.settings.textSize)) {
        throw new Error("The reading record contains an invalid text size.");
      }
      clean.settings.textSize = raw.settings.textSize;
    }
    return clean;
  }

  function mergeNotes(first, second) {
    if (!second || first === second || first.includes(second)) return first;
    if (!first || second.includes(first)) return second;
    // Recognize our own previous merge boundaries so importing a backup again
    // does not append its passages repeatedly. Complete distinct text is kept.
    return [...new Set([...first.split(SEPARATOR), ...second.split(SEPARATOR)])].join(SEPARATOR);
  }

  function combineLessons(first, second) {
    return {
      state: STATES[Math.max(STATES.indexOf(first.state), STATES.indexOf(second.state))],
      note: mergeNotes(first.note, second.note),
      clips: [...new Set([...first.clips, ...second.clips])],
    };
  }

  function mergeImported(existing, incoming) {
    const result = validate(existing);
    const imported = validate(incoming);
    for (const [id, item] of Object.entries(imported.lessons)) {
      result.lessons[id] = result.lessons[id] ? combineLessons(result.lessons[id], item) : item;
    }
    if (incoming.settings !== undefined) result.settings = imported.settings;
    return validate(result);
  }

  function resolveValue(base, local, latest, conflict) {
    if (equal(local, base)) return latest;
    if (equal(latest, base) || equal(local, latest)) return local;
    return conflict(local, latest);
  }

  function reconcileClips(base, local, latest) {
    const before = new Set(base), here = new Set(local), there = new Set(latest);
    // Membership is merged per passage: a deliberate removal need not discard
    // a different passage that another tab added at the same time.
    return [...new Set([...latest, ...local])].filter(clip =>
      here.has(clip) !== before.has(clip) ? here.has(clip) : there.has(clip));
  }

  function reconcile(base, local, latest) {
    const before = validate(base), here = validate(local), there = validate(latest);
    const result = empty();
    const ids = new Set([...Object.keys(before.lessons), ...Object.keys(here.lessons), ...Object.keys(there.lessons)]);
    for (const id of ids) {
      const old = before.lessons[id], ours = here.lessons[id], theirs = there.lessons[id];
      let resolved;
      if (equal(ours, old)) resolved = theirs;
      else if (equal(theirs, old) || equal(ours, theirs)) resolved = ours;
      else if (!ours) resolved = theirs; // Concurrent writing survives a stale deletion.
      else if (!theirs) resolved = ours;
      else {
        resolved = {
          state: resolveValue(old?.state, ours.state, theirs.state,
            (a, b) => STATES[Math.max(STATES.indexOf(a), STATES.indexOf(b))]),
          note: resolveValue(old?.note, ours.note, theirs.note, mergeNotes),
          clips: reconcileClips(old?.clips || [], ours.clips, theirs.clips),
        };
      }
      if (resolved !== undefined) result.lessons[id] = resolved;
    }
    result.settings.textSize = resolveValue(before.settings.textSize,
      here.settings.textSize, there.settings.textSize, (ours) => ours);
    return validate(result);
  }

  return { empty, validate, mergeImported, reconcile };
});
