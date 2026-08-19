import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import Conflict from './Conflict.svelte'

describe('Conflict', () => {
  it('lists colliding names and the three resolution actions', () => {
    const { body } = render(Conflict, {
      props: {
        names: ['notes.md', 'draft.md'],
        onchoose() {},
        oncancel() {},
      },
    })

    expect(body).toContain('Items already exist at the destination')
    expect(body).toContain('notes.md')
    expect(body).toContain('draft.md')
    expect(body).toContain('Apply to all')
    expect(body).toContain('Replace')
    expect(body).toContain('Keep Both')
    expect(body).toContain('Skip')
  })
})
