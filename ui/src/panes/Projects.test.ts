import { render } from 'svelte/server'
import { describe, expect, it } from 'vitest'

import Projects from './Projects.svelte'

describe('Projects', () => {
  it('renders the search field and empty hint', () => {
    const { body } = render(Projects, {
      props: {
        onopen() {},
        onerror() {},
        onadd() {},
        onremoved() {},
        oncollapse() {},
      },
    })

    expect(body).toContain('aria-label="Hide projects"')
    expect(body).toContain('Projects')
    expect(body).toContain('aria-label="Search projects"')
    expect(body).toContain('Open Folder…')
    expect(body).toContain('Your Markdown projects will appear here.')
  })
})
