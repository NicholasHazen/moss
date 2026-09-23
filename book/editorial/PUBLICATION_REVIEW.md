# Static publication review

Reviewed September 23, 2026. This is an internal review, outside `book/src` and
the exported reading edition. No deployment, browser operation, common build,
runtime change or Git setting change was performed by this reviewer.

The current export has sensible source boundaries, but **repository-subpath
validation needs repair before release**. The publication fixtures also expose
two robustness gaps in the snapshot packager. The assertions are intentionally
ordinary failing tests until those defects are resolved.

## Findings

### P2 — the checker does not model the repository URL or HTML base

The generated `404.html` currently contains `<base href="/">`. On the intended
repository site, that sends its relative stylesheet, script and navigation
requests to the host root instead of `/moss/`. Normal chapter-relative navigation
is not the failing case; the custom error page is.

[`check_book.py`](../scripts/check_book.py), lines 115–119 at review time, maps
every root-relative URL directly beneath the export directory. It treats a
`base` element's `href` as an ordinary destination and never applies it to the
other URLs. The fixture with `site-url = "/moss/"` therefore rejects a correct
`/moss/` base, while accepting the incorrect `/` base.

**Small fix:** set the production mdBook `output.html.site-url` to `/moss/`;
make HTML checking aware of that configured prefix and the first applicable
`base` element. Resolve document URLs using browser-like URL joining, then map
only URLs under the configured site prefix back into the export. Check a served
subpath and the published error page after the change. The installed mdBook
0.5.3 source explicitly uses `site-url` to generate this base.

Fixtures: `test_checker_accepts_configured_repository_base` and
`test_checker_rejects_base_that_escapes_repository_site`.

### P2 — attribute-shaped example text is rewritten as a real link

[`finish_site.py`](../scripts/finish_site.py), line 98 at review time, runs its
`href`/`src` regular expression over the whole HTML document. It consequently
treats an escaped HTML example, HTML comment or JavaScript string containing
`href="../../NOW.md"` as a request to publish that file. The current allowlist
correctly stops the private-file example, but the otherwise legitimate page
then fails to package. A literal naming an allowed file could instead be
silently changed, corrupting an example and publishing an uncited snapshot.

**Small fix:** identify real start-tag attributes with an HTML parser; perform
URL replacements only within those actual start tags. Preserve data, comments
and script text verbatim. This protects printed examples as well as the meaning
of “explicitly cited.”

Fixture: `test_attribute_looking_text_is_not_a_publication_request`.

### P2 — SVG is a raw-copy exception to the inert snapshot promise

`finish_site.py`, lines 40–42 at review time, copies any directly cited `.svg`
under an allowed project directory unchanged. An SVG with a script or event
handler therefore remains active when opened as its own document. The current
exported figures were inspected for script, event-handler, external-reference,
foreign-object and entity declarations; no such pattern was found. This is a
packager guarantee gap, not a finding of active content in the current book.

**Small fix:** make the distinction explicit between escaped source snapshots
and renderable figures, and restrict exported figures to the approved passive
SVG subset. Reject script, event handlers, embedded HTML, external resources
and entity declarations, or use a deliberate immutable figure allowlist. Do
not describe unvalidated raw SVG as inert source text.

Fixture: `test_active_svg_is_rejected_instead_of_exported_as_inert`.

## Protections confirmed

The isolated fixtures establish the following for the current production code:

- Rust text is HTML-escaped, includes a content digest and has line anchors.
  Markdown snapshots preserve duplicate-heading suffixes and line anchors.
- Direct citations are rewritten into one local snapshot per source; ordinary
  internal assets and external URLs remain unchanged. mdBook's `.html` rewrite
  can still resolve a repository `.md` source.
- `NOW.md`, `AGENTS.md`, collaboration notes, tutorial authoring material and
  book editorial files are rejected. A symlink in an allowed directory cannot
  disguise a disallowed source through the supported `main()` path. Traversal
  outside the repository and unsupported source file types are rejected.
- Links *inside* Markdown snapshots remain escaped source text and are not
  recursively followed. The existing 43 exported project files are code,
  teaching/design/research text or figures; no editorial/agent file was present.
- The HTML checker catches ordinary missing local files, missing fragments,
  duplicate IDs and links escaping the export directory.

## Stale output and reproducibility

The normal `book.sh` route invokes mdBook before snapshot packaging. Inspection
of the installed mdBook 0.5.3 renderer confirmed that it removes the destination's
old contents before rendering (`hbs_renderer.rs`, lines 316–319). Therefore a
normal successful wrapper build does not retain old source snapshots. Calling
`finish_site.py` alone is a postprocessing operation, not a clean rebuild; it
does not prune an earlier export. Keep the wrapper as the publication entry.

The wrapper pins and checks mdBook 0.5.3 and fails when either mdBook or packaging
fails. It does not itself run the navigation tests. A failed package can leave
partial output in `_site`; only publish after the complete validation sequence
succeeds, preferably from a fresh staging export.

Source snapshot hashes capture working-tree content, not a commit identity.
That is accurately disclosed in each snapshot. A public release manifest should
record the source checkpoint and tool version so readers can distinguish this
edition from later repository edits. The review did not establish build-byte
reproducibility across operating systems or Python versions.

## Evidence and limits

Added [`test_publication.py`](../scripts/test_publication.py). Ran:

```sh
python3 -B -m unittest discover -s book/scripts -p test_publication.py -v
```

**17 tests ran: 13 passed, 3 assertions failed and 1 raised an error.** The four
nonpassing tests reproduce the three findings above. There are no skipped or
expected-failure tests. Every fixture uses its own temporary tree; the live
export was read but not modified. Production scripts and configuration were
not edited.

This review does not establish browser accessibility, external-link availability,
the correctness of live Rust rules, hosted HTTP behavior, or a complete security
audit of third-party mdBook assets. The parent agent owns repairs, full rebuild,
browser verification and deployment.
