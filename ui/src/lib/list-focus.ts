/** Keyboard index for a project listbox: follow the active row, otherwise keep a valid index. */
export function listboxFocusIndex(
  length: number,
  activeIndex: number,
  previous: number,
): number {
  if (length <= 0) {
    return 0
  }
  if (activeIndex >= 0 && activeIndex < length) {
    return activeIndex
  }
  return Math.min(Math.max(previous, 0), length - 1)
}
