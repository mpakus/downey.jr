import type { TreeNode } from './generated/core'

/** One visible row in the flattened, lazily expanded file tree. */
export type TreeRow = {
  node: TreeNode
  depth: number
}

const MARKDOWN_NAME = /\.(md|markdown|mdown|mdwn)$/i

/** Returns whether a file name uses a recognized Markdown extension. */
export function isMarkdownPath(name: string): boolean {
  return MARKDOWN_NAME.test(name)
}

/** Icon bucket for a tree row: folder, Markdown, or anything else. */
export type FileIconKind = 'directory' | 'markdown' | 'file'

/** Distinguishes folders and Markdown files from other entries. */
export function fileIconKind(
  node: Pick<TreeNode, 'kind' | 'name'>,
): FileIconKind {
  if (node.kind === 'directory') {
    return 'directory'
  }
  if (isMarkdownPath(node.name)) {
    return 'markdown'
  }
  return 'file'
}

/** Directory prefixes that must be expanded to reveal `relPath`. */
export function ancestorDirs(relPath: string): string[] {
  const parts = relPath.split(/[/\\]/).filter(Boolean)
  parts.pop()
  const dirs: string[] = []
  let current = ''
  for (const part of parts) {
    current = current ? `${current}/${part}` : part
    dirs.push(current)
  }
  return dirs
}

/** Parent directory of a project-relative path, or `''` at the project root. */
export function parentDir(relPath: string): string {
  const parts = relPath.split(/[/\\]/).filter(Boolean)
  parts.pop()
  return parts.join('/')
}

/** Joins a project-relative directory and a file name. */
export function joinRel(dir: string, name: string): string {
  return dir ? `${dir}/${name}` : name
}

/** Directory that should receive a new file created from `node`. */
export function targetDir(node: TreeNode | null): string {
  if (!node) {
    return ''
  }
  if (node.kind === 'directory') {
    return node.relPath
  }
  return parentDir(node.relPath)
}

/** Parents first so lazy tree loads can expand from the root downward. */
export function sortDirsByDepth(dirs: Iterable<string>): string[] {
  return [...new Set(dirs)].sort((left, right) => {
    const leftDepth = left.split('/').filter(Boolean).length
    const rightDepth = right.split('/').filter(Boolean).length
    return leftDepth - rightDepth || left.localeCompare(right)
  })
}

/** Flattens loaded tree nodes according to the expanded directory set. */
export function flattenTree(
  rootNodes: TreeNode[],
  children: Record<string, TreeNode[]>,
  expanded: ReadonlySet<string>,
): TreeRow[] {
  const rows: TreeRow[] = []
  const walk = (nodes: TreeNode[], depth: number) => {
    for (const node of nodes) {
      rows.push({ node, depth })
      if (node.kind === 'directory' && expanded.has(node.relPath)) {
        const nested = children[node.relPath]
        if (nested) {
          walk(nested, depth + 1)
        }
      }
    }
  }
  walk(rootNodes, 0)
  return rows
}

/** Visible slice of a virtualized list, including a leading/trailing buffer. */
export function visibleWindow(
  count: number,
  scrollTop: number,
  viewportHeight: number,
  rowHeight: number,
  buffer: number,
): { start: number; end: number } {
  if (viewportHeight <= 0 || count === 0) {
    return { start: 0, end: count }
  }
  const start = Math.max(0, Math.floor(scrollTop / rowHeight) - buffer)
  const end = Math.min(
    count,
    Math.ceil((scrollTop + viewportHeight) / rowHeight) + buffer,
  )
  return { start, end }
}

/** Project-relative asset URL produced by the Markdown renderer. */
export type AssetRef = {
  projectId: string
  relPath: string
  hash: string
}

const ASSET_HREF =
  /^asset:\/\/localhost\/([^/]+)\/([^?#]*)(?:\?[^#]*)?(?:#(.*))?$/i

/** Parses `asset://localhost/<projectId>/<rel>` including an optional hash. */
export function parseAssetHref(href: string): AssetRef | null {
  const match = href.trim().match(ASSET_HREF)
  if (!match) {
    return null
  }
  const projectId = match[1]
  const encodedPath = match[2]
  if (!projectId || encodedPath == null) {
    return null
  }
  try {
    const relPath = decodeURIComponent(encodedPath)
    if (!relPath || relPath.includes('..')) {
      return null
    }
    return { projectId, relPath, hash: match[3] ?? '' }
  } catch {
    return null
  }
}

/** Returns whether a link should open in the system browser. */
export function isHttpHref(href: string): boolean {
  return /^https?:\/\//i.test(href.trim())
}

/** Parent directories that should reload after watched paths change. */
export function dirsToReload(paths: Iterable<string>): string[] {
  const dirs = new Set<string>([''])
  for (const path of paths) {
    dirs.add(parentDir(path))
  }
  return sortDirsByDepth(dirs)
}

/** Inclusive range of visible tree paths between an anchor and the clicked row. */
export function rangeRelPaths(
  rows: TreeRow[],
  anchor: string | null,
  target: string,
): string[] {
  const targetIndex = rows.findIndex((row) => row.node.relPath === target)
  if (targetIndex < 0) {
    return [target]
  }
  const anchorIndex = anchor
    ? rows.findIndex((row) => row.node.relPath === anchor)
    : targetIndex
  if (anchorIndex < 0) {
    return [target]
  }
  const start = Math.min(anchorIndex, targetIndex)
  const end = Math.max(anchorIndex, targetIndex)
  return rows.slice(start, end + 1).map((row) => row.node.relPath)
}

/** Destination folder under a pointer, or `null` when the pointer is not over the tree. */
export function dropDirAtPoint(x: number, y: number): string | null {
  if (typeof document === 'undefined') {
    return null
  }
  const el = document.elementFromPoint(x, y)
  if (!(el instanceof Element)) {
    return null
  }
  const row = el.closest('[data-rel]')
  if (row instanceof HTMLElement && row.dataset.rel != null) {
    return row.dataset.kind === 'directory'
      ? row.dataset.rel
      : parentDir(row.dataset.rel)
  }
  return null
}
