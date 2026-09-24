# Source of the published cumulative previews

This is the complete worked project used to build the five cumulative browser
previews. It includes the investigations retained while following the course
through refuge. Its source fingerprint is
`2ae9855a1ba77d93e5b45fad33661261b8347fdb3f937dd7776c37511faa37aa`.
The browser label calls this a learner project because the verification followed
the same saved-project route as a reader. It is an authored example, not your
own edited file, and the public website never receives local edits.

Begin your own project with the fieldwork guide. Keep this complete answer
beside your work without replacing your saved files or investigations.

To check this exact source from the repository root:

```sh
python3 learning/scripts/fieldwork.py check --project learning/published-preview --stage refuge
```

To rebuild an earlier browser view, pass this directory to that guide's
`fieldwork_preview.py --project` command. Refuge itself remains a native Cargo
investigation. Rust 1.93.1, the locked dependencies and the WASM target for
browser builds must be installed separately.
