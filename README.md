# 1537paperstreet

Локальный Markdown-ридер для macOS на Rust, Tauri 2 и Svelte 5. Работает с
папками на диске, не требует аккаунта и не отправляет документы в сеть.

Сейчас это **ридер** с простым режимом правки исходника: проекты, дерево
файлов, превью, Edit/Split, темы, Mermaid, KaTeX и экспорт PDF. CodeMirror
и история версий ещё впереди. Архитектура — [`docs/PLAN.md`](docs/PLAN.md),
задачи — [`docs/CHECKLIST.md`](docs/CHECKLIST.md), правила для агентов —
[`AGENTS.md`](AGENTS.md). Руководство пользователя собирается mdBook из
[`docs/src/`](docs/src/).

## Быстрый старт

Нужны Rust (stable), Node.js 22+ и Xcode Command Line Tools.

```sh
git clone https://github.com/mpakus/downey.jr.git
cd downey.jr
npm install
cargo tauri dev
```

Через минуту откроется окно. File → Open Folder… (или перетащите папку с
`.md` на окно) — слева дерево, справа превью.

Перед отправкой изменений:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check
npm run lint
npm run test
```

Как участвовать — [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Тестовые сборки

GitHub Release по тегу `v*` кладёт **universal** DMG (Apple Silicon и Intel)
в [Releases](https://github.com/mpakus/downey.jr/releases). Сборка не подписана
Developer ID и не нотарифицирована: при обычном запуске macOS отклоняет её.
Это тестовая сборка, а не готовый production-релиз. Первый запуск: Right-click →
Open или `xattr -cr /Applications/1537paperstreet.app` после копирования.

Локально: `npm run tauri:build:universal` (нужны оба Rust-таргета
`aarch64-apple-darwin` и `x86_64-apple-darwin`).
