import { setInvokeForTests } from './ipc'

type Handler = (args?: Record<string, unknown>) => unknown | Promise<unknown>

/** Installs in-memory handlers for IPC commands used by UI tests. */
export function mockIpc(handlers: Record<string, Handler>): void {
  setInvokeForTests(async (cmd, args) => {
    const handler = handlers[cmd]
    if (!handler) {
      throw new Error(`No mock for IPC command ${cmd}`)
    }
    return handler(args)
  })
}

/** Restores the default Tauri invoke implementation. */
export function resetIpc(): void {
  setInvokeForTests(null)
}
