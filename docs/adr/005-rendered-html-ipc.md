# ADR-005: Send rendered HTML across IPC

- Status: accepted
- Date: 2026-08-13

## Context

Large documents must open incrementally without shipping a second Markdown implementation or a large parser bundle to the WebView.

## Decision

Render and sanitize Markdown in Rust, then stream ready-to-insert HTML chunks to the WebView.

## Alternatives

- Sending Markdown would require parsing in JavaScript and duplicate renderer behavior.
- Sending a custom syntax tree would add a large serialization contract and a second rendering layer.

## Consequences

Heavy parsing remains native and the WebView remains a simple renderer. The Rust sanitizer becomes a security boundary, chunk ordering needs an explicit protocol, and incremental patches must produce the same HTML as a full render.
