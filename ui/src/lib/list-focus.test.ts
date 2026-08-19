import { describe, expect, it } from 'vitest'

import { listboxFocusIndex } from './list-focus'

describe('listboxFocusIndex', () => {
  it('follows the active project instead of always using the first row', () => {
    expect(listboxFocusIndex(3, 2, 0)).toBe(2)
  })

  it('keeps a previous index when nothing is active', () => {
    expect(listboxFocusIndex(3, -1, 1)).toBe(1)
  })

  it('clamps when the list shrinks', () => {
    expect(listboxFocusIndex(1, -1, 4)).toBe(0)
  })
})
