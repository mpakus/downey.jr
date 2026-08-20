# Contributing

Read [`AGENTS.md`](AGENTS.md), [`docs/PLAN.md`](docs/PLAN.md), and
[`docs/CHECKLIST.md`](docs/CHECKLIST.md) before you start.

One checklist task per branch `task/T-xxx-short-name`, closed by its own PR.
Write the test first, then the code. Commits use Conventional Commits. Code,
comments, and commit messages are English. User-facing docs (README, this file,
CHANGELOG, the mdBook in `docs/src/`) are English. `PLAN.md`, `CHECKLIST.md`,
and `AGENTS.md` stay Russian for now.

Every Definition of Done item in `AGENTS.md` is required, including
`npm run check`. IPC types are generated from Rust. After changing a struct:

```sh
UPDATE_TS_BINDINGS=1 cargo test -p ps-core --test typescript
```

Commit `ui/src/lib/generated/core.ts`. UI tests mock IPC through
`ui/src/lib/ipc.mock.ts`. Filesystem tests use `tempfile` only — never the
real `$HOME`.

History (P10) and ZIP export (P12) are out of scope unless someone explicitly
asks. Do not add them “while you’re here.”

Release: tag `vX.Y.Z` (same version as `package.json`) runs
`.github/workflows/release.yml` — universal `.app`/DMG on `macos-14` and a
GitHub Release. Signing and notarization are not in that workflow yet.
