# 1537paperstreet

A local Markdown reader for macOS. Open the folders you already keep on disk.
There is no account, no cloud, and no network call when you read a document.

Made in Austin ✩ Texas · [aomega.co](https://aomega.co)

[Download the latest release](https://github.com/mpakus/1537paperstreet/releases/latest)

![Preview of a Markdown file with syntax highlighting, the Projects list, and the file tree](docs/screen01.png)

## Why it exists

Most Markdown tools want a vault format, a sync service, or a browser tab.
1537paperstreet is a native window over ordinary folders: notes, specs, READMEs,
and wikis that already live next to your code.

- **Your files stay yours.** Projects are paths on disk. Removing a project from
  the list does not delete the folder.
- **Read first.** Preview GitHub Flavored Markdown with a table of contents,
  tabs, and a full-size reading view. Edit the source when you need to; save
  with ⌘S.
- **Private by default.** Themes, Mermaid, and KaTeX are local. The only
  optional network request is **Check for Updates**, which asks GitHub Releases
  when you click it.

![Document preview with a table of contents, tabs, and a rendered architecture diagram](docs/screen02.png)

## What you can do

- Keep several folders in **Projects**. Open Folder…, File → Open Folder…, or
  drop a folder onto the window — it is added to the list automatically.
  Opening the same folder again reuses the existing entry.
- Browse a lazy file tree, open Markdown in tabs, and switch Preview / Edit /
  Split.
- Read tables, task lists, footnotes, alerts, YAML front matter, wiki links
  `[[Note]]`, fenced code (Rust, Python, Ruby, Elixir, YAML, JS/TS, and more),
  Mermaid diagrams, and KaTeX math.
- Export the open document to PDF. Copy or save a diagram as SVG/PNG.
- Twelve built-in themes, session light/dark with ⌘⌥T, and custom JSON themes
  in `~/.1537paperstreet/themes/`.

![About 1537paperstreet, with the file tree and a Mermaid sequence diagram in the preview](docs/screen03.png)

## Install

macOS 12 (Monterey) or newer, Apple Silicon or Intel. GitHub Releases ship a
**universal** `.app` and DMG.

Release 0.3.0 and earlier are **unsigned**. Gatekeeper will warn for those
downloads. First launch: Right-click → Open, or after copying to Applications:

```sh
xattr -cr /Applications/1537paperstreet.app
```

New releases are published only after Developer ID signing, Apple
notarization, and Gatekeeper verification succeed.

Closing the red traffic light hides the window; the menu-bar icon stays.
Click it to show the window again. Quit from that menu or ⌘Q.

File → Check for Updates… (also in About) compares your version to GitHub
Releases and can open the download page. It does not install in place yet.

More detail: the [user guide](docs/src/index.md) (`docs/src/`).

---

## For developers

Rust + Tauri 2 + Svelte 5. Architecture is [`docs/PLAN.md`](docs/PLAN.md),
tasks are [`docs/CHECKLIST.md`](docs/CHECKLIST.md), agent rules are
[`AGENTS.md`](AGENTS.md).

### Run from source

Rust (stable), Node.js 22+, and Xcode Command Line Tools:

```sh
git clone https://github.com/mpakus/1537paperstreet.git
cd 1537paperstreet
npm install
cargo tauri dev
```

Universal `.app` and DMG:

```sh
npm run tauri:build:universal
```

Before a pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check
npm run lint
npm run test
```

How we work: [`CONTRIBUTING.md`](CONTRIBUTING.md). After changing IPC types:

```sh
UPDATE_TS_BINDINGS=1 cargo test -p ps-core --test typescript
```

A `v*` tag builds the universal binary and publishes a GitHub Release only
after the app and DMG pass Developer ID signature, notarization-ticket, and
Gatekeeper verification. Required repository secrets are documented in
[`CONTRIBUTING.md`](CONTRIBUTING.md).
