import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import Editor from './Editor.svelte'

describe('Editor', () => {
  it('renders a host for the lazily loaded source editor', () => {
    const { body } = render(Editor, {
      props: {
        value: '# Title',
      },
    })

    expect(body).toContain('cm-host')
    expect(body).not.toContain('cm-editor')
    expect(body).not.toContain('codemirror')
  })
})
