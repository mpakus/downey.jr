# 1537paperstreet

Локальный Markdown-редактор и ридер для macOS на Rust, Tauri и Svelte.

Проект находится на ранней стадии разработки. Архитектура описана в
[`docs/PLAN.md`](docs/PLAN.md), порядок работ — в
[`docs/CHECKLIST.md`](docs/CHECKLIST.md).

## Разработка

```sh
npm install
cargo tauri dev
```

Перед отправкой изменений:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run lint
npm run test
```
