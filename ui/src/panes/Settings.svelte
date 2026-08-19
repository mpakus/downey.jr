<script lang="ts">
  import { onMount } from 'svelte'

  import type { Config, ThemeInfo } from '../lib/ipc'

  const BODY_FONTS = [
    'New York',
    'Iowan Old Style',
    'Palatino',
    'Georgia',
    'system-ui',
  ]
  const MONO_FONTS = ['JetBrains Mono', 'SF Mono', 'Menlo', 'ui-monospace']

  let {
    config,
    themes,
    onsave,
    onclose,
    onlive,
  }: {
    config: Config
    themes: ThemeInfo[]
    onsave: (next: Config) => void
    onclose: () => void
    onlive?: (next: Config) => void
  } = $props()

  // svelte-ignore state_referenced_locally
  // $state proxies cannot be structuredClone'd; snapshot() is a plain Config.
  let draft = $state($state.snapshot(config))
  let pageEl = $state<HTMLDivElement | undefined>(undefined)
  // svelte-ignore state_referenced_locally
  let readingColorsOn = $state(
    config.viewer.preview_bg !== '' || config.viewer.preview_fg !== '',
  )

  const lightThemes = $derived(
    themes.filter((theme) => theme.appearance === 'light'),
  )
  const darkThemes = $derived(
    themes.filter((theme) => theme.appearance === 'dark'),
  )

  onMount(() => {
    pageEl?.focus()
  })

  function persist() {
    onlive?.($state.snapshot(draft))
  }

  function pickTheme(which: 'light' | 'dark', id: string) {
    if (which === 'light') {
      draft.appearance.theme = id
    } else {
      draft.appearance.theme_dark = id
    }
    persist()
  }

  function cssColorToHex(value: string): string {
    const trimmed = value.trim()
    if (/^#[0-9a-fA-F]{6}$/.test(trimmed)) {
      return trimmed.toLowerCase()
    }
    const rgb = trimmed.match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/)
    if (!rgb) {
      return ''
    }
    const hex = (part: string) => Number(part).toString(16).padStart(2, '0')
    return `#${hex(rgb[1] ?? '0')}${hex(rgb[2] ?? '0')}${hex(rgb[3] ?? '0')}`
  }

  function tokenHex(token: string): string {
    if (typeof document === 'undefined') {
      return ''
    }
    return cssColorToHex(
      getComputedStyle(document.documentElement).getPropertyValue(token),
    )
  }

  function setReadingColors(on: boolean) {
    readingColorsOn = on
    if (!on) {
      draft.viewer.preview_bg = ''
      draft.viewer.preview_fg = ''
      persist()
      return
    }
    draft.viewer.preview_bg = tokenHex('--bg')
    draft.viewer.preview_fg = tokenHex('--fg')
    persist()
  }
</script>

<div
  class="page"
  role="dialog"
  aria-modal="true"
  aria-label="Settings"
  tabindex="-1"
  bind:this={pageEl}
  onkeydown={(event) => {
    if (event.key === 'Escape') {
      onclose()
    }
  }}
>
  <header>
    <div>
      <h2>Settings</h2>
      <p class="lede">
        Color themes follow popular editor palettes (Solarized, Nord, Gruvbox,
        Catppuccin, Tokyo Night, GitHub). ⌘⌥T flips light and dark for this
        session.
      </p>
    </div>
    <button
      type="button"
      class="done"
      onclick={() => {
        onsave($state.snapshot(draft))
      }}>Done</button
    >
  </header>

  <div class="body">
    <section>
      <h3>Themes</h3>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.appearance.follow_system}
          onchange={persist}
        />
        Follow system appearance
      </label>

      <p class="legend">Light</p>
      <div class="gallery" role="group" aria-label="Light theme">
        {#each lightThemes as theme (theme.id)}
          <button
            type="button"
            class="theme-card"
            data-theme={theme.id}
            aria-pressed={draft.appearance.theme === theme.id}
            onclick={() => pickTheme('light', theme.id)}
          >
            <span class="preview" aria-hidden="true">
              <span class="chip elev"></span>
              <span class="chip fg"></span>
              <span class="chip accent"></span>
            </span>
            <span class="theme-name">{theme.name}</span>
          </button>
        {/each}
      </div>

      <p class="legend">Dark</p>
      <div class="gallery" role="group" aria-label="Dark theme">
        {#each darkThemes as theme (theme.id)}
          <button
            type="button"
            class="theme-card"
            data-theme={theme.id}
            aria-pressed={draft.appearance.theme_dark === theme.id}
            onclick={() => pickTheme('dark', theme.id)}
          >
            <span class="preview" aria-hidden="true">
              <span class="chip elev"></span>
              <span class="chip fg"></span>
              <span class="chip accent"></span>
            </span>
            <span class="theme-name">{theme.name}</span>
          </button>
        {/each}
      </div>
    </section>

    <section>
      <h3>Typography</h3>
      <div class="grid">
        <label>
          Body font
          <select bind:value={draft.typography.body_font} onchange={persist}>
            {#each BODY_FONTS as font (font)}
              <option value={font}>{font}</option>
            {/each}
          </select>
        </label>
        <label>
          Mono font
          <select bind:value={draft.typography.mono_font} onchange={persist}>
            {#each MONO_FONTS as font (font)}
              <option value={font}>{font}</option>
            {/each}
          </select>
        </label>
        <label>
          Font size
          <input
            type="number"
            min="10"
            max="32"
            bind:value={draft.typography.font_size}
            onchange={persist}
          />
        </label>
        <label>
          Line height
          <input
            type="number"
            min="1.2"
            max="2"
            step="0.05"
            bind:value={draft.typography.line_height}
            onchange={persist}
          />
        </label>
        <label>
          Measure (ch)
          <input
            type="number"
            min="40"
            max="120"
            bind:value={draft.typography.measure_ch}
            onchange={persist}
          />
        </label>
      </div>
    </section>

    <section>
      <h3>Preview & Split</h3>
      <p class="hint">
        These apply to rendered Markdown only — not the editor, tree, or chrome.
        Leave them on the theme to follow the palette above.
      </p>
      <div class="grid">
        <label>
          Reading font
          <select bind:value={draft.viewer.preview_font} onchange={persist}>
            <option value="">Same as body font</option>
            {#each BODY_FONTS as font (font)}
              <option value={font}>{font}</option>
            {/each}
          </select>
        </label>
        <label>
          Reading size
          <input
            type="number"
            min="10"
            max="32"
            placeholder="Theme"
            value={draft.viewer.preview_font_size || ''}
            onchange={(event) => {
              const raw = event.currentTarget.value
              const size = Number(raw)
              draft.viewer.preview_font_size =
                raw === '' || Number.isNaN(size) ? 0 : size
              persist()
            }}
          />
        </label>
      </div>
      <label class="check">
        <input
          type="checkbox"
          checked={readingColorsOn}
          onchange={(event) => setReadingColors(event.currentTarget.checked)}
        />
        Custom reading colors
      </label>
      {#if readingColorsOn}
        <div class="colors">
          <label>
            Background
            <input
              type="color"
              value={draft.viewer.preview_bg || tokenHex('--bg')}
              oninput={(event) => {
                draft.viewer.preview_bg = event.currentTarget.value
                persist()
              }}
            />
          </label>
          <label>
            Text
            <input
              type="color"
              value={draft.viewer.preview_fg || tokenHex('--fg')}
              oninput={(event) => {
                draft.viewer.preview_fg = event.currentTarget.value
                persist()
              }}
            />
          </label>
        </div>
      {/if}
    </section>

    <section>
      <h3>Window</h3>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.window.show_in_dock}
          onchange={persist}
        />
        Keep icon in Dock when the window is hidden
      </label>
      <p class="hint">
        Uncheck to remove the Dock icon after the red traffic light hides the
        window. The menu-bar icon still brings the app back.
      </p>
    </section>

    <section>
      <h3>Files and preview</h3>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.viewer.show_toc}
          onchange={persist}
        />
        Show table of contents
      </label>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.files.confirm_delete}
          onchange={persist}
        />
        Confirm moving items to Trash
      </label>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.files.show_hidden}
          onchange={persist}
        />
        Show hidden files
      </label>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.viewer.mermaid_enabled}
          onchange={persist}
        />
        Render Mermaid diagrams
      </label>
      <label class="check">
        <input
          type="checkbox"
          bind:checked={draft.viewer.math_enabled}
          onchange={persist}
        />
        Render mathematics
      </label>
    </section>
  </div>
</div>

<style>
  .page {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding-top: 38px;
    background: var(--bg);
    color: var(--fg);
  }

  header {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: var(--space-4);
    flex: none;
    padding: var(--space-5) var(--space-6);
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev);
  }

  h2 {
    margin: 0;
    font-size: 1.25rem;
  }

  .lede {
    max-width: 40rem;
    margin: var(--space-2) 0 0;
    color: var(--fg-muted);
    font-size: 0.875rem;
    line-height: 1.45;
  }

  .done {
    min-height: 32px;
    padding: 0 var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    font-weight: 600;
  }

  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--space-5) var(--space-6) var(--space-6);
  }

  section + section {
    margin-top: var(--space-6);
  }

  h3 {
    margin: 0 0 var(--space-3);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--fg-muted);
  }

  .legend {
    margin: var(--space-3) 0 var(--space-2);
    color: var(--fg-muted);
    font-size: 0.75rem;
  }

  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9.5rem, 1fr));
    gap: var(--space-2);
  }

  .theme-card {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-elev);
    color: var(--fg);
    text-align: start;
  }

  .theme-card[aria-pressed='true'] {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .preview {
    display: flex;
    height: 40px;
    overflow: hidden;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
  }

  .chip {
    flex: 1;
  }

  .chip.elev {
    flex: 3;
    background: var(--bg-elev);
  }

  .chip.fg {
    background: var(--fg);
  }

  .chip.accent {
    background: var(--accent);
  }

  .theme-name {
    font-size: 0.8125rem;
    font-weight: 600;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr));
    gap: var(--space-3);
  }

  label {
    display: grid;
    gap: var(--space-1);
    color: var(--fg);
    font-size: 0.875rem;
  }

  label.check {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .hint {
    margin: 0 0 var(--space-2);
    color: var(--fg-muted);
    font-size: 0.8125rem;
    line-height: 1.45;
  }

  .colors {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
    margin: var(--space-2) 0 var(--space-3);
  }

  .colors label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  input[type='color'] {
    width: 2.5rem;
    height: 2rem;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--bg);
  }

  input,
  select {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    font: inherit;
  }
</style>
