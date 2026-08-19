# ADR-002: Parse Markdown with pulldown-cmark

- Status: accepted
- Date: 2026-08-13

## Context

Rendering must be fast, support GFM features, rewrite links, collect a table of contents, recognize special fenced blocks, and retain byte offsets for incremental rendering and synchronized scrolling.

## Decision

Use `pulldown-cmark` with tables, footnotes, strikethrough, task lists, smart punctuation, heading attributes, math, GFM alerts, YAML metadata blocks, definition lists, and wiki links enabled. Transform its event stream in Rust (front matter HTML, wiki resolution, task-list classes, classed syntect).

## Alternatives

`comrak` offers more extensions out of the box, but its tree-oriented API is less suitable for streaming transformations and the offset-based incremental pipeline.

## Consequences

The renderer can process documents as a pull-based event stream and retain source offsets. Wiki backlinks remain out of scope.
