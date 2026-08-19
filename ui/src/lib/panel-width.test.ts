import { describe, expect, it } from 'vitest'

import { clampPanelWidth } from './panel-width'

describe('clampPanelWidth', () => {
  it('rounds fractional pixels so config_set can store u32 widths', () => {
    expect(clampPanelWidth('tree', 128.1653125)).toBe(160)
    expect(clampPanelWidth('editor', 512.4, 900)).toBe(512)
  })

  it('lets the split editor grow with the workspace instead of stopping at 720', () => {
    expect(clampPanelWidth('editor', 2000, 1600)).toBe(1320)
  })
})
