import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import ReloadDisk from './ReloadDisk.svelte'

describe('ReloadDisk', () => {
  it('asks to reload a file that changed in another program', () => {
    const { body } = render(ReloadDisk, {
      props: {
        name: 'notes.md',
        onreload() {},
        onkeep() {},
      },
    })

    expect(body).toContain('File changed on disk')
    expect(body).toContain('notes.md was updated in another program.')
    expect(body).toContain('Reload')
    expect(body).toContain('Keep this version')
    expect(body).not.toContain('discards unsaved edits')
  })

  it('warns when reload would discard unsaved edits', () => {
    const { body } = render(ReloadDisk, {
      props: {
        name: 'notes.md',
        dirty: true,
        onreload() {},
        onkeep() {},
      },
    })

    expect(body).toContain('Reloading discards unsaved edits.')
  })

  it('offers to close a file that was removed', () => {
    const { body } = render(ReloadDisk, {
      props: {
        name: 'gone.md',
        missing: true,
        onreload() {},
        onkeep() {},
      },
    })

    expect(body).toContain('File removed from disk')
    expect(body).toContain('gone.md is no longer in this folder.')
    expect(body).toContain('Close')
    expect(body).toContain('Keep open')
  })
})
