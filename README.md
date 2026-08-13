# 1537paperstreet

Локальный Markdown-редактор и ридер для macOS на Rust, Tauri и Svelte.

Проект находится на ранней стадии разработки. Архитектура описана в
[`docs/PLAN.md`](docs/PLAN.md), порядок работ — в
[`docs/CHECKLIST.md`](docs/CHECKLIST.md).

## Тестовые сборки

Текущий DMG для Apple Silicon не подписан Developer ID и не нотарифицирован.
При обычном запуске macOS отклоняет его как приложение без пригодной подписи.
Это тестовая сборка, а не готовый production-релиз; сайт распространения должен
показывать это предупреждение рядом со ссылкой на скачивание.

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
