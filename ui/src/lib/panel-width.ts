export type PanelKind = 'sidebar' | 'tree' | 'toc' | 'editor'

const PREVIEW_MIN = 280

/** Integer panel width. The editor may grow with the Split workspace. */
export function clampPanelWidth(
  kind: PanelKind,
  requested: number,
  workspaceWidth = 0,
): number {
  const min = kind === 'toc' ? 140 : kind === 'editor' ? 240 : 160
  const max =
    kind === 'toc'
      ? 360
      : kind === 'editor'
        ? Math.max(min, Math.floor(workspaceWidth) - PREVIEW_MIN)
        : 480
  return Math.min(max, Math.max(min, Math.round(requested)))
}
