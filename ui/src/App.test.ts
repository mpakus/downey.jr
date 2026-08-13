import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import App from './App.svelte'

describe('App', () => {
  it('renders the initial project state', () => {
    const { body, head } = render(App)

    expect(head).toContain('<title>1537paperstreet</title>')
    expect(body).toContain('Your Markdown projects will appear here.')
  })
})
