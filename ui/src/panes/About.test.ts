import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import About from './About.svelte'

describe('About', () => {
  it('shows branding, version, and the Austin credit', () => {
    const { body } = render(About, {
      props: {
        version: '0.1.0',
        onclose() {},
        onopen() {},
      },
    })

    expect(body).toContain('aria-label="About 1537paperstreet"')
    expect(body).toContain('src="/logo.png"')
    expect(body).toContain('logo-stripe')
    expect(body).toContain('1537paperstreet')
    expect(body).toContain('0.1.0')
    expect(body).toContain('Made in Austin ✩ Texas')
    expect(body).toContain('https://aomega.co')
    expect(body).toContain('A local Markdown reader for macOS')
  })
})
