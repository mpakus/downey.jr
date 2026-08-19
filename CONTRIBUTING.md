# Участие в разработке

Перед началом прочитайте [`AGENTS.md`](AGENTS.md),
[`docs/PLAN.md`](docs/PLAN.md) и [`docs/CHECKLIST.md`](docs/CHECKLIST.md).

Одна задача из чеклиста выполняется в отдельной ветке `task/T-xxx-short-name`
и закрывается отдельным PR. Сначала добавляется проверка, затем реализация.
Коммиты используют Conventional Commits, а код, комментарии и сообщения
коммитов пишутся на английском. Документация для людей (`PLAN.md`,
`CHECKLIST.md`, `AGENTS.md`, mdBook) — на русском.

Все проверки из Definition of Done в `AGENTS.md` обязательны, включая
`npm run check`. Типы, пересекающие IPC, генерируются из Rust: после изменения
структуры запустите

```sh
UPDATE_TS_BINDINGS=1 cargo test -p ps-core --test typescript
```

и закоммитьте `ui/src/lib/generated/core.ts`.

Тесты UI мокают IPC через `ui/src/lib/ipc.mock.ts`. Тесты файловой системы —
только на `tempfile`, без обращения к реальному `$HOME`.

Редактор, история версий и ZIP-экспорт не входят в текущий ридер. Не добавляйте
их «заодно»: это отдельные фазы в чеклисте.

Релиз: тег `vX.Y.Z` (версия как в `package.json`) запускает
`.github/workflows/release.yml` — universal `.app`/DMG на `macos-14` и GitHub
Release. Подпись и нотаризация ещё не в этом workflow.
