import { describe, expect, it } from 'vitest'

import { hasMath, renderMathSpan } from './math'

describe('math', () => {
  it('detects math spans', () => {
    const empty = { querySelector: () => null } as unknown as ParentNode
    const withMath = {
      querySelector: (selector: string) => (selector === '.math' ? {} : null),
    } as unknown as ParentNode
    expect(hasMath(empty)).toBe(false)
    expect(hasMath(withMath)).toBe(true)
  })

  it('keeps an error message when KaTeX throws', () => {
    const node = {
      textContent: 'x + y',
      classList: { contains: () => false },
      dataset: {} as Record<string, string>,
    } as unknown as HTMLElement
    renderMathSpan(node, {
      render() {
        throw new Error('bad formula')
      },
    } as unknown as typeof import('katex').default)
    expect(node.textContent).toBe('bad formula')
    expect(node.dataset.rendered).toBe('error')
  })
})
