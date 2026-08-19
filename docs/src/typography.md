# Шрифты и типографика

Превью использует гарнитуры, которые уже стоят на macOS, затем запасные:

- текст: New York, Iowan Old Style, Palatino, Georgia, system-ui
- код: JetBrains Mono (если установлен), SF Mono, Menlo, ui-monospace

Кегль интерфейса: ⌘+ / ⌘− / ⌘0, тот же выбор в Settings → Typography.
Значение пишется в `config.json` и переживает перезапуск.

В Preview и Split панель справа даёт A− / A+ (кегль чтения) и − / % / +
(зум страницы на сессию, 50–200 %).

Шрифт, кегль и цвета именно превью (и правой половины Split) живут в
секции Preview & Split: `preview_font`, `preview_font_size`, `preview_bg`,
`preview_fg`. Пустое / `0` — токены темы.

JetBrains Mono в приложение не вшит: если шрифта нет, будет SF Mono.
