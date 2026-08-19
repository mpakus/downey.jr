# ADR-009: Document PDF via WKWebView

- Status: accepted
- Date: 2026-08-19

## Context

PLAN § 18 listed PDF export as v1.1. The product now needs **File → Export PDF…**
for the open Markdown document, with a native Save dialog. A Chromium or
`printpdf` HTML layout engine would be a large new dependency and would miss
Mermaid SVGs already drawn in the preview.

## Decision

Build a self-contained HTML snapshot in the UI (preview innerHTML, theme CSS,
typography variables, inlined images) and render it to PDF on macOS with an
offscreen `WKWebView.createPDFWithConfiguration`. Bytes are written through
`save_user_file` (absolute path, `.pdf` only, atomic temp → fsync → rename).

ZIP project export remains P12 and is unchanged.

## Alternatives

- System print dialog (“Save as PDF”): no default path from a Save panel.
- `html2canvas` / jsPDF: extra frontend payload, poor print quality.
- Headless Chrome: hundreds of megabytes, network surface.

## Consequences

PDF matches the preview theme, including rendered diagrams. Export is macOS-only,
same as v1. Offscreen WebKit must run on the main thread.
