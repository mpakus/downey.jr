import { describe, expect, it } from 'vitest'

import { markdownHighlightStyle, markdownTokenClasses } from './highlight'

describe('markdown editor highlighting', () => {
  it('maps Markdown tokens onto theme CSS classes', () => {
    expect(markdownTokenClasses).toEqual([
      'cm-md-heading',
      'cm-md-em',
      'cm-md-strong',
      'cm-md-strike',
      'cm-md-link',
      'cm-md-code',
      'cm-md-quote',
      'cm-md-comment',
      'cm-md-mark',
    ])
    const classes = markdownHighlightStyle.specs.map((spec) => spec.class)
    for (const name of markdownTokenClasses) {
      expect(classes).toContain(name)
    }
  })
})
