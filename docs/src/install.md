# Установка

Минимальная версия macOS — 12.0 (Monterey).

**Из исходников** (сейчас основной способ):

1. Установите Rust (rustup, stable), Node.js 22+ и Xcode Command Line Tools.
2. Клонируйте репозиторий.
3. `npm install`
4. `cargo tauri dev` — окно разработки; `cargo tauri build` — `.app` текущей
   архитектуры в `target/release/bundle`. Universal (arm64 + x86_64, macOS 12+):
   `npm run tauri:build:universal` — `.app` и DMG в
   `target/universal-apple-darwin/release/bundle`. Иконка приложения —
   `icon.png` в корне репозитория; `npx tauri icon icon.png` записывает
   PNG/ICNS в `crates/ps-app/icons/`.
   Тег `v*` на GitHub собирает тот же universal DMG и публикует Release.

Закрытие окна (красный светофор) не завершает программу: в правой части
строки меню macOS остаётся иконка (`icon-system.png`). Клик возвращает окно;
Quit в меню иконки или ⌘Q выходят полностью.

File → About 1537paperstreet (и пункт About в меню приложения) открывает
окно с логотипом, версией и ссылкой на [aomega.co](https://aomega.co).

Тестовые DMG без Developer ID macOS не запускает без обхода Gatekeeper.
Production-релизы будут подписаны и нотаризированы; до этого сборка
считается тестовой.

Homebrew cask и автообновления появятся вместе с первой подписанной версией.
