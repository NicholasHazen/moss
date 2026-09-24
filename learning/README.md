# Moss Fieldnotes

A new browser-first learning experience beside the published mdBook edition.
Read [Moss Fieldnotes](https://nicholashazen.github.io/moss/fieldnotes/).
This edition preserves `NOW.md`, the unfinished movement helper, and the existing
simulation/presentation boundary. The original book remains at its existing URL.

Fieldnotes connects Rust and ECS lessons with a continuing meadow you build in
one saved Cargo project. The numbered lessons isolate a language or scheduling
question; the cumulative guides bring those ideas together through supply,
competing meals, births, inherited upkeep, local movement, committed rest and
hunting. The [refuge capstone](content/refuge.html) then asks you to design a
complete extension and an experiment of your own, with staged hints and full
worked answers available. The source supports the complete continuing project route; the hosted browser
views run precompiled examples, while Rust edits and tests run locally.

Fieldnotes 0.3 adds chapter overviews, closing summaries and section navigation.
The [ecosystem atlas identity](IDENTITY.md) pairs a generated habitat mark and
meadow cover with a quieter reading layout. Chapter menus collapse on narrow
screens; the prose, code and explanatory diagrams remain selectable.

## Run

From the repository root, with Python 3.11+ (Node 22+ is used for the JavaScript checks):

```sh
python3 learning/scripts/build.py
python3 learning/scripts/serve.py
```

Open <http://127.0.0.1:8091>. Generated pages are in ignored `learning/_site`.
The site has no runtime packages, account, network fonts, analytics, or backend.
The JavaScript panels are labeled teaching models. The optional Rust terrarium
runs a precompiled `moss-sim` instance; Rust editing and compilation remain local.

To include it, install the pinned `wasm32-unknown-unknown` target, run
`python3 learning/scripts/runtime.py check` and
`python3 learning/scripts/runtime.py build`, then rebuild the course.
The [terrarium guide](content/terrarium.html) explains its evidence boundary.

Lesson 13 also compiles an isolated learner exercise into a browser panel:

```sh
python3 learning/scripts/preview.py evidence
python3 learning/scripts/build.py
```

After preparing and editing the lesson's starter, pass
`--file work/evidence-practice/evidence.rs` to `preview.py evidence`. The command
checks the exact source against the canonical tests before compiling it. The
panel labels a printed reference separately from a learner build. Both optional
Rust hosts require `rustup target add --toolchain 1.93.1 wasm32-unknown-unknown`.
Builds run local code with normal local permissions, without a remote compiler.

The cumulative **fieldwork** guide keeps edits and learner-authored tests in one
Cargo project across several investigations:

```sh
python3 learning/scripts/fieldwork.py prepare --dest work/fieldwork
python3 learning/scripts/fieldwork.py check --project work/fieldwork
python3 learning/scripts/fieldwork_preview.py --project work/fieldwork
python3 learning/scripts/build.py
```

Open `fieldwork.html`. Preparation refuses to overwrite an existing copy; return
to that same directory to continue. The checker snapshots saved source, tests,
examples and benches, runs separate required acceptance assertions, then runs
the project's targets without replacing learner tests. It supports this
standalone package shape; external path dependencies, workspace members, custom
build scripts and escaping target paths are rejected with an explanation.
The preview checks and compiles the selected saved source, labels it as a learner
project, and is omitted from a later static build if its inputs changed. Use
`--reference` instead of `--project` to explicitly select the course reference.
This first arc combines upkeep, bounded growth, finite meals, death and retained
history at fixed feeding contacts. It does not yet integrate travel or births.

Then open **population.html** and extend that same project through modelling,
maturation, birth transactions and inherited upkeep. Keep earlier tests. The
population check requires both the first meadow and population contracts:

```sh
python3 learning/scripts/fieldwork.py check --project work/fieldwork --stage population
python3 learning/scripts/fieldwork_preview.py --project work/fieldwork --population
python3 learning/scripts/build.py
```

The guide supplies a review-only change map, complete module answers and a
separate downloadable checkpoint. `prepare` always starts at the stable first
meadow; it never upgrades or replaces an existing learner directory. Native
adapter tests and final WASM compilation use an invocation-private target to
keep simultaneous preview modes from substituting artifacts. The browser
compares actual tick traces, parentage, measured flows and bounded histories.
The mobile, resting and hunting guides extend those same sources with local
observation, paid travel, committed rest and contested captures.

**mobile.html** adds the third cumulative checkpoint: represent space, write an
owned observation system, pay for movement, revalidate contact and place newborns.
Its map distinguishes the observation origin from the current cell, and its tables
keep recorded biomass separate from current food. The optional browser build uses
the same saved project:

```sh
python3 learning/scripts/fieldwork.py check --project work/fieldwork --stage mobile
python3 learning/scripts/fieldwork_preview.py --project work/fieldwork --mobile
python3 learning/scripts/build.py
```

**resting.html** adds fatigue and committed rest to that same project. Write the
activity system, keep its transitions separate from energy payments, and test
fast recovery that still owes another rest action. Compare two matched policies
in the compiled browser without changing the running world merely by selecting
a menu:

```sh
python3 learning/scripts/fieldwork.py check --project work/fieldwork --stage resting
python3 learning/scripts/fieldwork_preview.py --project work/fieldwork --resting
python3 learning/scripts/build.py
```

**hunting.html** follows two hunters that select the same prey, then explains why
only one can complete a capture. Extend local observation and travel to the new
role, revalidate current contact, and make the accepted removal immediately
visible to later actions. The browser compares the captured and starved census,
actual energy flows and retained observations:

```sh
python3 learning/scripts/fieldwork.py check --project work/fieldwork --stage hunting
python3 learning/scripts/fieldwork_preview.py --project work/fieldwork --hunting
python3 learning/scripts/build.py
```

The [refuge capstone](content/refuge.html) continues in that same directory.
Choose a representation for static shelter, implement its complete membership
system, and connect current protection to hunter observation and capture. Then
write an investigation and compare refuge placements with matched starting
conditions. Keep your earlier tests as you extend the project:

```sh
python3 learning/scripts/fieldwork.py check --project work/fieldwork --stage refuge
cargo +1.93.1 test --offline --locked --manifest-path work/fieldwork/Cargo.toml --test refuge_investigation
cargo +1.93.1 run --offline --locked --manifest-path work/fieldwork/Cargo.toml --example refuges
```

The guide explains where the investigation and example files belong and supplies
complete answers for comparison. Refuge results are native: there is no refuge
browser mode. Changes to your saved source make earlier browser artifacts stale;
rebuild those with their existing preview commands if you want to keep using
them. They still select the earlier scenarios without refuge configuration or
projection, even though they now compile from your current source.

Stable answers live in `learning/checkpoints`; each guide links readable complete
source pages, raw files, a review-only change map and a separate reference ZIP.
The Copy complete file button copies the exact displayed source. Preparing an
initial project still starts from the first meadow checkpoint, never a later answer.

**returns.html** offers six mixed retrieval cases with staged hints and complete
reasoning. Its five worked tests can live beside the earlier investigation in
the same project. In lesson 13, the additional test-design route runs your own
tests against the correct program and six deliberate defects; a compile failure
does not count as detecting a behavioral defect.

## Optional narration

On macOS with an installed voice and `say` / `afconvert` available:

```sh
python3 learning/scripts/narration.py all
python3 learning/scripts/build.py
```

Use a lesson or static guide ID (such as `fieldwork`) in place of `all` to
generate one track. Generated navigation pages are excluded. The default voice is
`Samantha (English (US))`; `--voice` and `--rate` customize generation. Audio is
generated locally under ignored `learning/work/narration`, with passage cues
measured from the generated PCM audio. Prose and tables are read; code and
answer disclosures are skipped. The static build copies only current audio into
`_site`; playback and passage highlighting work independently of macOS. No
runtime audio service is required. A changed lesson must be regenerated. The
browser speech button remains an optional fallback when a voice service works.
If a restricted shell cannot reach macOS's speech service, run this command in
your normal local terminal; the rest of the course remains usable without it.

### ElevenLabs narration

The optional ElevenLabs runner uses the same offline player and passage cues.
It needs Python and `ffmpeg` for audio assembly; it does not require macOS speech.
Start with a dry run, which reads local text without contacting ElevenLabs:

```sh
python3 learning/scripts/narration_elevenlabs.py 02-ownership --voice VOICE_ID --sample-passages 7 --output-directory work/audio-audition
```

The first seven passages form an approximately two-minute voice audition. The
plan reports the exact generation text and which chunks need a new request.
Choose a voice and inspect that plan before generation. With
`ELEVENLABS_API_KEY` configured in your local environment, the explicit generation
command also requires a ceiling on the uncached text:

```sh
python3 learning/scripts/narration_elevenlabs.py 02-ownership --voice VOICE_ID --sample-passages 7 --generate --max-characters 5000
```

That character ceiling is not a currency quote; model, account and provider
pricing determine the charge. Auditions are saved separately under
`learning/work/narration-auditions`, so they never replace a complete lesson.
Omit `--sample-passages` to generate a complete page, or use `all` for eligible
pages after reviewing the full dry-run estimate. Then run `build.py` to publish
only source-current audio. Existing Mac recordings remain available until a
complete replacement has passed validation.

Generation sends the selected prose and its declared continuity context to
ElevenLabs. The key stays in the local generation process; the static course
contains audio and timing metadata, with no runtime API requirement. Chunks are
cached by their exact text, voice, model, settings and context. A failed request
is not retried automatically because the provider may already have billed it.
The generator has offline tests; no live ElevenLabs recording is included.
Mac system-voice recordings are for the local personal edition and are excluded
from public distribution. The hosted course retains browser speech and passage
highlighting.

## Check

```sh
python3 learning/scripts/check.py
node --test learning/scripts/test_*.cjs
python3 -m unittest discover -s learning/scripts -p 'test_*.py'
python3 learning/scripts/labs.py check all
```

The local lab runner extracts exact HTML code blocks. It checks Rust 1.93.1 and
the ECS lab uses Bevy ECS 0.18.1 with its own committed lockfile. Cargo uses cached
dependencies and an isolated target directory. References and learner files are
reported distinctly, and required tests cannot disappear into a zero-test pass.
No commands here complete or schedule the live movement exercise.

Lesson 13 also has a separate test-design task. Unlike the implementation runner,
this command preserves the learner's tests and runs them against correct and
deliberately wrong implementations:

```sh
python3 learning/scripts/evidence_practice.py prepare --dest work/evidence-test-design
python3 learning/scripts/evidence_practice.py check --file work/evidence-test-design/evidence_tests.rs
```

A weak test can pass every program while leaving the task incomplete. Feedback
names each surviving defect; only an executed test failure in a compiling wrong
program counts as detecting that defect. The full lesson supplies hints and a
complete worked test file. `check --answer` checks that answer separately.

## Portable reading edition

After generating the optional narration and Rust hosts, package the current site:

```sh
python3 learning/scripts/package.py /path/to/moss-fieldnotes-0.3.zip
```

The ZIP includes a loopback server, a reading guide and a SHA-256 manifest. Extract
it and run `python3 serve.py`. Reading, delivered audio and compiled previews work
without the source checkout; native Rust practice still uses this checkout and its
pinned runner. External reference links and publisher videos need a connection.

## Authoring

`course.json` owns available lesson order and metadata. `content` holds semantic
HTML fragments. The build wraps fragments into complete pages and indexes their
text for local search. All prose and disclosures work without JavaScript. Use
`code[data-runnable="id"]` for a complete reference, and
`code[data-starter="id"]` for a complete editable starter. Each example must end
with a `#[cfg(test)] mod tests` block containing meaningful named tests.

[SOURCES.md](SOURCES.md) records research provenance and permitted use.
The project's live verification record owns actual Moss runtime evidence;
course examples do not activate future biology in that simulation.

## Source of the hosted previews

[The published worked project](published-preview/README.md) contains the exact
source used for the five cumulative browser previews, including its saved
investigations. The website does not compile or receive your local edits.

## Prepare a public export

After rebuilding and reviewing the complete local site, including the compiled
previews, prepare a separate public directory:

```sh
python3 learning/scripts/publish.py --destination work/public-fieldnotes --source-archive /path/to/reviewed-source.zip
```

The exporter checks the current source and binary metadata, excludes locally
generated Mac recordings, retains browser Listen, includes the source download
and writes a byte-hash manifest. It refuses a nonempty destination and never
uploads or changes the personal reading edition. Review this separate export
before deploying it alongside the existing book.
