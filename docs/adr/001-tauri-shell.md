# ADR-001: Use Tauri 2 for the application shell

- Status: accepted
- Date: 2026-08-13

## Context

The macOS application needs native filesystem services, a small distribution, Mermaid rendering, and a mature editor with IME, composition, virtualization, and spellchecking support.

## Decision

Use Tauri 2 with the system WKWebView. Keep application and filesystem logic in Rust; expose only thin typed commands to the Svelte UI.

## Alternatives

- `egui`, `iced`, or GPUI would still require a JavaScript engine and an SVG renderer for Mermaid, plus a custom editor.
- Electron provides the required web platform but has a much larger memory and distribution footprint.
- Swift and AppKit would be native, but would duplicate the Rust core and still need a web surface for Mermaid.

## Consequences

The app reuses macOS web rendering and can use Mermaid and CodeMirror without bundling a browser. The IPC boundary must remain explicit, WKWebView has limited automated E2E support, and all rendered user Markdown must be protected against XSS.
