import { describe, expect, it } from 'vitest'

import {
  PREVIEW_ZOOM_MAX,
  PREVIEW_ZOOM_MIN,
  nextPreviewZoom,
  previewZoomPercent,
} from './zoom'

describe('nextPreviewZoom', () => {
  it('steps by tenths and stays inside the reading range', () => {
    expect(nextPreviewZoom(1, 0.1)).toBe(1.1)
    expect(nextPreviewZoom(1, -0.1)).toBe(0.9)
    expect(nextPreviewZoom(PREVIEW_ZOOM_MIN, -0.1)).toBe(PREVIEW_ZOOM_MIN)
    expect(nextPreviewZoom(PREVIEW_ZOOM_MAX, 0.1)).toBe(PREVIEW_ZOOM_MAX)
  })

  it('formats a percent label', () => {
    expect(previewZoomPercent(1)).toBe('100%')
    expect(previewZoomPercent(1.2)).toBe('120%')
  })
})
