import type { Project } from './generated/core'

/** Recent projects, most recently opened first. */
export function recentProjects(projects: Project[], limit = 12): Project[] {
  return [...projects]
    .sort((left, right) =>
      (right.last_opened_at ?? '').localeCompare(left.last_opened_at ?? ''),
    )
    .slice(0, limit)
}

/** Absolute paths carried by a Finder / HTML drop, if the webview exposes them. */
export function pathsFromDataTransfer(transfer: DataTransfer | null): string[] {
  if (!transfer) {
    return []
  }
  const fromFiles = [...transfer.files]
    .map((file) => {
      const path = (file as File & { path?: string }).path
      return typeof path === 'string' ? path : ''
    })
    .filter(Boolean)
  if (fromFiles.length > 0) {
    return fromFiles
  }
  const uriList = transfer.getData('text/uri-list')
  if (!uriList) {
    return []
  }
  return uriList
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && !line.startsWith('#'))
    .flatMap((line) => {
      if (!line.startsWith('file:')) {
        return []
      }
      try {
        const url = new URL(line)
        if (url.protocol !== 'file:') {
          return []
        }
        return [decodeURIComponent(url.pathname)]
      } catch {
        return []
      }
    })
}
