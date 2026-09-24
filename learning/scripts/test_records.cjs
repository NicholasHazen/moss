const test = require('node:test');
const assert = require('node:assert/strict');
const records = require('../assets/records.js');

const lesson = (note = '', state = 'new', clips = []) => ({ state, note, clips });
const record = (lessons = {}, textSize = 'normal') => ({ version: 1, lessons, settings: { textSize } });
const clone = value => JSON.parse(JSON.stringify(value));

test('a cumulative fieldwork record survives export, import and a later note merge', () => {
  const original = record({fieldwork: lesson('Predicted two survivors', 'practicing', ['Timing matters'])});
  const restored = records.validate(JSON.parse(JSON.stringify(original)));
  const incoming = record({fieldwork: lesson('Observed one under pulsed supply', 'reviewed')});
  const merged = records.mergeImported(restored, incoming);
  assert.equal(merged.lessons.fieldwork.state, 'reviewed');
  assert.match(merged.lessons.fieldwork.note, /Predicted two survivors/);
  assert.match(merged.lessons.fieldwork.note, /Observed one under pulsed supply/);
  assert.deepEqual(merged.lessons.fieldwork.clips, ['Timing matters']);
});

test('refuge capstone notes round-trip and merge without losing earlier lesson writing', () => {
  const original = record({
    hunting: lesson('Capture needs current contact', 'reviewed', ['A target is not a reservation']),
    refuge: lesson('Predict protection at the food cell', 'practicing', ['Protection grants no energy']),
  });
  const restored = records.mergeImported(records.empty(), JSON.parse(JSON.stringify(original)));
  assert.deepEqual(restored, original);
  const incoming = record({ refuge: lesson('Observed fewer captures and more starvation', 'reviewed') });
  const merged = records.mergeImported(restored, incoming);
  assert.deepEqual(merged.lessons.hunting, original.lessons.hunting);
  assert.equal(merged.lessons.refuge.state, 'reviewed');
  assert.match(merged.lessons.refuge.note, /Predict protection at the food cell/);
  assert.match(merged.lessons.refuge.note, /Observed fewer captures and more starvation/);
  assert.deepEqual(merged.lessons.refuge.clips, ['Protection grants no energy']);
  const pending = clone(merged);
  pending.lessons.refuge.note = '';
  pending.lessons.refuge.clips = [];
  assert.deepEqual(records.reconcile(merged, pending, merged), pending);
});

test('validation returns independent records and rejects malformed data without discarding lessons', () => {
  const original = record({ '01-observe': lesson('keep', 'reading', ['quote']) });
  const checked = records.validate(original);
  checked.lessons['01-observe'].clips.push('independent');
  assert.deepEqual(original.lessons['01-observe'].clips, ['quote']);
  assert.throws(() => records.validate(record({ 'bad-id': lesson('valuable note') })), /invalid lesson/);
  assert.throws(() => records.validate(record({ '01-observe': lesson('x'.repeat(40001)) })), /limit/);
  assert.throws(() => records.validate(record({ '01-observe': lesson('', 'new', ['x'.repeat(5001)]) })), /limit/);
  assert.throws(() => records.validate(record({ '01-observe': lesson('', 'new', Array(101).fill('q')) })), /limit/);
});

test('a failed import is atomic even when its first lesson is new', () => {
  const original = record({ '01-observe': lesson('A'.repeat(30000)) });
  const incoming = record({
    '02-ownership': lesson('must not leak from a failed import'),
    '01-observe': lesson('B'.repeat(30000)),
  });
  const before = clone(original), incomingBefore = clone(incoming);
  assert.throws(() => records.mergeImported(original, incoming), /limit/);
  assert.deepEqual(original, before);
  assert.deepEqual(incoming, incomingBefore);
});

test('import preserves distinct writing and clips, furthest state, and is repeatable', () => {
  const original = record({ '01-observe': lesson('A', 'reviewed', ['one']) });
  const incoming = record({ '01-observe': lesson('B', 'reading', ['one', 'two']) }, 'large');
  const merged = records.mergeImported(original, incoming);
  assert.equal(merged.lessons['01-observe'].state, 'reviewed');
  assert.equal(merged.lessons['01-observe'].note, 'A\n\n— Imported note —\n\nB');
  assert.deepEqual(merged.lessons['01-observe'].clips, ['one', 'two']);
  assert.equal(merged.settings.textSize, 'large');
  assert.deepEqual(records.mergeImported(merged, incoming), merged);
  assert.deepEqual(records.mergeImported(merged, original).lessons, merged.lessons);
  assert.equal(original.lessons['01-observe'].note, 'A');
});

test('valid exported records larger than 2 MB can be imported without an arbitrary total cap', () => {
  const large = records.empty();
  for (let i = 1; i <= 5; i++) {
    large.lessons[`0${i}-lesson`] = lesson('note', 'reading',
      Array.from({ length: 100 }, (_, n) => `${n}`.padEnd(5000, 'x')));
  }
  const serialized = JSON.stringify(large);
  assert.ok(Buffer.byteLength(serialized) > 2000000);
  assert.deepEqual(records.mergeImported(records.empty(), JSON.parse(serialized)), large);
});

test('a stale tab preserves notes another tab saved in a different lesson', () => {
  const base = records.empty();
  const tabA = record({ '01-observe': lesson('A note') });
  const tabB = record({ '02-ownership': lesson('B note') });
  const savedA = records.reconcile(base, tabA, records.empty());
  const savedB = records.reconcile(base, tabB, savedA);
  assert.equal(savedB.lessons['01-observe'].note, 'A note');
  assert.equal(savedB.lessons['02-ownership'].note, 'B note');
});

test('concurrent writing to one lesson keeps both versions without changing any input', () => {
  const base = record({ '01-observe': lesson('Original', 'reading', ['old']) });
  const local = record({ '01-observe': lesson('My explanation', 'practicing', ['old', 'mine']) });
  const latest = record({ '01-observe': lesson('Other explanation', 'reviewed', ['old', 'theirs']) });
  const originals = [clone(base), clone(local), clone(latest)];
  const merged = records.reconcile(base, local, latest).lessons['01-observe'];
  assert.ok(merged.note.includes('My explanation'));
  assert.ok(merged.note.includes('Other explanation'));
  assert.equal(merged.state, 'reviewed');
  assert.deepEqual(new Set(merged.clips), new Set(['old', 'mine', 'theirs']));
  assert.deepEqual([base, local, latest], originals);
});

test('unchanged local fields take newer storage fields while local edits remain', () => {
  const base = record({ '01-observe': lesson('Original', 'reading') });
  const local = record({ '01-observe': lesson('Edited', 'reading') });
  const latest = record({ '01-observe': lesson('Original', 'reviewed', ['new passage']) }, 'larger');
  const merged = records.reconcile(base, local, latest);
  assert.deepEqual(merged.lessons['01-observe'], lesson('Edited', 'reviewed', ['new passage']));
  assert.equal(merged.settings.textSize, 'larger');
});

test('intentional clearing, passage removal and progress reset survive when latest is unchanged', () => {
  const base = record({ '01-observe': lesson('Original', 'reviewed', ['one', 'two']) });
  const local = record({ '01-observe': lesson('', 'new', ['two']) });
  assert.deepEqual(records.reconcile(base, local, base), local);
  assert.deepEqual(records.reconcile(base, base, local), local);
});

test('a passage removal and a different concurrent addition both survive', () => {
  const base = record({ '01-observe': lesson('', 'new', ['one', 'two']) });
  const local = record({ '01-observe': lesson('', 'new', ['two']) });
  const latest = record({ '01-observe': lesson('', 'new', ['one', 'two', 'three']) });
  assert.deepEqual(records.reconcile(base, local, latest).lessons['01-observe'].clips, ['two', 'three']);
});

test('lesson deletion is honored unless it conflicts with new writing', () => {
  const base = record({ '01-observe': lesson('Original') });
  assert.deepEqual(records.reconcile(base, records.empty(), base), records.empty());
  const latest = record({ '01-observe': lesson('New writing') });
  assert.equal(records.reconcile(base, records.empty(), latest).lessons['01-observe'].note, 'New writing');
});

test('a conflicting merge that exceeds limits fails without truncating pending notes', () => {
  const base = record({ '01-observe': lesson('Original') });
  const local = record({ '01-observe': lesson('A'.repeat(30000)) });
  const latest = record({ '01-observe': lesson('B'.repeat(30000)) });
  const before = [clone(base), clone(local), clone(latest)];
  assert.throws(() => records.reconcile(base, local, latest), /limit/);
  assert.deepEqual([base, local, latest], before);
});
