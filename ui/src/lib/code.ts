/** Wraps highlighted code blocks with a Copy button. */
export function enhanceCodeBlocks(
  root: HTMLElement,
  onerror: (message: string) => void,
): () => void {
  const blocks = [...root.querySelectorAll<HTMLElement>('pre.code')]
  const buttons: HTMLButtonElement[] = []
  for (const pre of blocks) {
    if (pre.parentElement?.classList.contains('code-block')) {
      continue
    }
    const wrap = document.createElement('div')
    wrap.className = 'code-block'
    pre.replaceWith(wrap)
    wrap.append(pre)
    const button = document.createElement('button')
    button.type = 'button'
    button.className = 'code-copy'
    button.textContent = 'Copy'
    button.addEventListener('click', () => {
      void navigator.clipboard.writeText(pre.textContent ?? '').then(
        () => {
          button.textContent = 'Copied'
          window.setTimeout(() => {
            button.textContent = 'Copy'
          }, 1200)
        },
        (cause: unknown) => {
          onerror(cause instanceof Error ? cause.message : String(cause))
        },
      )
    })
    wrap.append(button)
    buttons.push(button)
  }
  return () => {
    for (const button of buttons) {
      button.replaceWith()
    }
  }
}
