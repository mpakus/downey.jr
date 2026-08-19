import { describe, expect, it } from 'vitest'

import { cacheKey, diagramSource, mermaidThemeVariables } from './diagrams'

describe('diagrams', () => {
  it('reads the template source and builds a cache key', () => {
    const figure = {
      querySelector(selector: string) {
        if (selector === 'template') {
          return { textContent: 'graph TD\n  A --> B' }
        }
        return null
      },
    } as unknown as Element
    expect(diagramSource(figure)).toContain('graph TD')
    expect(cacheKey('abc', 'paper-light')).toBe('abc:paper-light')
  })

  it('reads WebKit template content when textContent is empty', () => {
    const figure = {
      querySelector(selector: string) {
        if (selector === 'template') {
          return {
            textContent: '',
            content: { textContent: 'flowchart LR\n  A --> B' },
          }
        }
        return null
      },
    } as unknown as Element
    expect(diagramSource(figure)).toContain('flowchart LR')
  })

  it('maps theme tokens onto Mermaid variables', () => {
    const style = {
      getPropertyValue(name: string) {
        const tokens: Record<string, string> = {
          '--bg': ' #111111 ',
          '--fg': '#eeeeee',
          '--border': '#333333',
          '--selection': '#222222',
          '--fg-muted': '#999999',
          '--bg-elev': '#1a1a1a',
          '--code-bg': '#181818',
          '--font-ui': 'system-ui',
        }
        return tokens[name] ?? ''
      },
    } as unknown as CSSStyleDeclaration
    expect(mermaidThemeVariables(style)).toMatchObject({
      background: '#111111',
      primaryTextColor: '#eeeeee',
      fontFamily: 'system-ui',
    })
  })

  it('falls back to paper-light tokens when CSS variables are empty', () => {
    const style = {
      getPropertyValue() {
        return '  '
      },
    } as unknown as CSSStyleDeclaration
    expect(mermaidThemeVariables(style).background).toBe('#fbfaf7')
    expect(mermaidThemeVariables(style).primaryTextColor).toBe('#1e1c1a')
  })
})
