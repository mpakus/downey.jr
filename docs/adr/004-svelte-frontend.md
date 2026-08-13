# ADR-004: Build the frontend with Svelte 5

- Status: accepted
- Date: 2026-08-13

## Context

The WebView needs a compact, reactive UI with TypeScript support and fast startup. Business rules must have one implementation shared by every caller.

## Decision

Use Svelte 5, Vite, and TypeScript. Keep the frontend thin: Rust performs parsing, formatting, sorting, filtering, and filesystem work. The CodeMirror text buffer is the sole planned exception.

## Alternatives

- React carries a larger virtual-DOM runtime for this application.
- A framework-free frontend would require custom state and component infrastructure.
- Moving business rules into TypeScript would duplicate core behavior across the IPC boundary.

## Consequences

The shipped JavaScript stays small and most behavior is testable in `ps-core`. IPC types must be generated from Rust, and UI components cannot silently reimplement core rules for convenience.
