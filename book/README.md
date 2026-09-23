# Moss textbook edition

This is a separate, illustrated edition of the Moss learning guides. The
existing [tutorial](../docs/tutorial/README.md) remains the current coding route;
[NOW.md](../NOW.md) still owns the one active edit. Book examples do not install
future rules in the live simulation.

Read the [published web book](https://nicholashazen.github.io/moss/), or start
locally in [A small world, understood](src/index.md). The source uses
ordinary Markdown so chapters can also be read in RustRover. The browser edition
adds search, interactive experiments, and a print view.

## Build and preview

The publishing helpers use Python 3.11 or later; the model checks use Node.
From the repository root, install the pinned official publishing tool once:

```sh
scripts/with-toolchain.sh cargo install mdbook --version 0.5.3 --locked --root .tools/mdbook-0.5.3
```

Build or preview with:

```sh
sh book/scripts/book.sh build
sh book/scripts/book.sh serve
```

The preview binds to <http://127.0.0.1:8090/moss/> and serves a fixed build at the
same `/moss/` prefix as the public site, including its custom 404 page. After editing,
run the build command again and reload the page. Generated HTML stays in the ignored
`book/_site/` directory. This command does not start Moss itself; run the
simulation with the [development commands](../docs/development/README.md).

## Maintain the edition

The [editorial work card](editorial/STATUS.md) records the edition's actual
progress, checks and limits. Editorial notes are outside the published source
directory. Chapters must distinguish the working project from complete isolated
examples and future design. Keep source links, code examples and observations
aligned with the checkpoint they describe.

Publishing tooling is separate from the Cargo workspace. The tool version is
pinned here and in the wrapper; Moss's dependencies and lockfile are unchanged.
GitHub Pages serves the verified static export from the root of `gh-pages`.
The Moss repository's homepage links to the book. The first publication is
commit `14b93fab06513aa6440150816e94044ae7936dd9`; it leaves `main` and its
existing local edits unchanged.

The second editorial pass is committed as `776dcec` on that branch and verified
locally. Its Pages retry is pending; [the work card](editorial/STATUS.md) records
the deployment issue and the remaining public verification.

Each build also creates two deterministic downloads: the runnable Moss starter
and the editable book source with its bounded reference inputs. Their manifests
record file hashes and executable modes; adjacent checksums identify each ZIP.
The source download can rebuild the complete export, including both downloads.
See the public [edition notes](src/reference/edition-notes.md) for scope and limits.

For a later release, rebuild and run the checks below, then inspect the local
preview. Publish only `book/_site/` from a separate checkout of `gh-pages`, using
a normal commit and push. Preserve the branch history; do not replace the active
Moss checkout or include private editorial material. Confirm Pages deployment
success and check the actual public pages and downloads before recording a release.

Repository text citations are copied into the exported edition as escaped source
snapshots, with their file paths and content hashes. Figures use a restricted,
passive SVG vocabulary checked during export. Only explicitly linked
files in approved code/design/tutorial areas are packaged; the snapshot viewer
does not follow their links or publish the rest of the checkout. Source Markdown
retains its ordinary relative links for RustRover. `NOW.md`, editorial notes
and collaboration material stay outside the export.

## Verification

After building, check the source links, exact copies of canonical Rust examples,
and exported HTML navigation with:

```sh
python3 -B book/scripts/check_book.py
python3 -B -m unittest discover -s book/scripts -p 'test_*.py'
node --test book/scripts/test_*lab*.cjs
```

Execute the canonical worked Rust references in isolated workspaces using:

```sh
python3 docs/tutorial/authoring/check_path_examples.py
python3 docs/tutorial/authoring/check_path_examples.py --movement
```

The first command checks the sixteen future reference modules. The second checks
the movement answer and activates the prepared schedule only in its temporary
copy. Neither changes the live exercise. Passing these checks does not replace
a browser check of the book's models, a review of its prose, or live Moss
integration evidence. Exact results belong in the editorial work card.
