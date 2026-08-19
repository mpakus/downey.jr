import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import EditorToolbar from './EditorToolbar.svelte'

describe('EditorToolbar', () => {
  it('shows text, links, and media controls', () => {
    const { body } = render(EditorToolbar, {
      props: {
        oncommand() {},
      },
    })

    expect(body).toContain('aria-label="Editor"')
    expect(body).toContain('aria-label="Text"')
    expect(body).toContain('aria-label="Links"')
    expect(body).toContain('aria-label="Media"')
    expect(body).toContain('aria-label="Bold"')
    expect(body).toContain('aria-label="Link"')
    expect(body).toContain('aria-label="Image"')
  })
})
