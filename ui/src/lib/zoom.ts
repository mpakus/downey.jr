/** Smallest Preview/Split zoom. */
export const PREVIEW_ZOOM_MIN = 0.5
/** Largest Preview/Split zoom. */
export const PREVIEW_ZOOM_MAX = 2

/** Next zoom after a ±0.1 step, clamped to the supported range. */
export function nextPreviewZoom(current: number, delta: number): number {
  const stepped = Math.round((current + delta) * 10) / 10
  return Math.min(PREVIEW_ZOOM_MAX, Math.max(PREVIEW_ZOOM_MIN, stepped))
}

/** Formats a zoom factor for the toolbar, e.g. `100%`. */
export function previewZoomPercent(zoom: number): string {
  return `${Math.round(zoom * 100)}%`
}
