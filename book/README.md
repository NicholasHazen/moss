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
GitHub Pages serves a verified static export committed on `gh-pages`, deployed
by the manually triggered [publication workflow](../.github/workflows/publish-book.yml)
on `main`. The repository homepage links to the book. The first publication was
export `14b93fab06513aa6440150816e94044ae7936dd9`.

The second editorial pass, export `776dcec`, is now publicly verified. A
[fresh workflow run](https://github.com/NicholasHazen/moss/actions/runs/35897888968)
published it after the managed branch-publishing rerun became stuck. See the
[recovery record](editorial/WORKFLOW_RECOVERY.md) for settings and evidence.

Each build also creates two deterministic downloads: the runnable Moss starter
and the editable book source with its bounded reference inputs. Their manifests
record file hashes and executable modes; adjacent checksums identify each ZIP.
The source download can rebuild the complete export, including both downloads.
See the public [edition notes](src/reference/edition-notes.md) for scope and limits.

For a later release, rebuild and run the checks below, then inspect the local
preview. Commit only `book/_site/` to a separate checkout of `gh-pages`, using
a normal commit and push. Preserve the branch history; do not replace the active
Moss checkout or include editorial material in the export. Verify the remote
export against the reviewed files, then use **Actions → Publish Moss book → Run
workflow** on `main`, supplying that export's full commit SHA. A push alone no
longer deploys. With an authenticated account that can publish this repository,
the equivalent CLI command is:

```sh
gh workflow run publish-book.yml --repo NicholasHazen/moss --ref main \
  -f export_commit=FULL_VERIFIED_EXPORT_COMMIT_SHA
```

Replace the placeholder with the verified export SHA. The workflow validates
its format and basic export landmarks; it does not replace the release checks.
Confirm deployment success and compare the public pages and downloads with
the export manifest before recording a release. Record both the workflow commit
and the content's export commit. Before publishing newer content, check the old
queued run's state as described in the recovery record.

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
