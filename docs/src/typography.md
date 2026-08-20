# Fonts and typography

Preview uses typefaces already on macOS, then fallbacks:

- body: New York, Iowan Old Style, Palatino, Georgia, system-ui
- code: JetBrains Mono (if installed), SF Mono, Menlo, ui-monospace

Chrome type size: ⌘+ / ⌘− / ⌘0, same control in Settings → Typography.
The value is written to `config.json` and survives relaunch.

In Preview and Split the right side of the bar has A− / A+ (reading size)
and − / % / + (session page zoom, 50–200 %).

Font, size, and colors for the preview itself (and the right half of Split)
live under Preview & Split: `preview_font`, `preview_font_size`,
`preview_bg`, `preview_fg`. Empty / `0` means theme tokens.

JetBrains Mono is not bundled: if it is missing, you get SF Mono.
