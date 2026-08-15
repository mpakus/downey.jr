# ADR-003: Emit classed syntax highlighting

- Status: accepted
- Date: 2026-08-13

## Context

Code blocks need broad syntax coverage while theme changes must stay within the 16 ms interaction budget.

## Decision

Use `syntect` with the pure-Rust `regex-fancy` backend and `ClassedHTMLGenerator`. Emit semantic highlight classes and resolve their colors through theme CSS variables. Load the shared `SyntaxSet` lazily through `OnceLock`.
Prefix generated scope classes with `syntax-` to keep the renderer-to-theme contract isolated from application classes.

## Alternatives

- Inline colors would require rerendering every highlighted block when the theme changes.
- A JavaScript highlighter would increase the frontend bundle and move document processing out of Rust.
- The `onig` backend introduces a native C dependency.

## Consequences

Theme changes update CSS instead of document HTML, and syntax data is loaded only when needed. Themes must define every supported highlight token, and class naming becomes a stable renderer-to-theme contract.
