# ADR-006: Move deleted files to the macOS Trash

- Status: accepted
- Date: 2026-08-13

## Context

Project files contain user-authored text. An ordinary delete action must be recoverable through familiar system behavior and through application history.

## Decision

Use the `trash` crate for ordinary deletion. Create the required `pre_*` history snapshot before the operation. Permanent deletion is available only through an explicit Shift-modified confirmation flow.

## Alternatives

- Calling `unlink` is immediate and cannot be undone through Finder.
- Implementing a private trash folder would duplicate macOS behavior and complicate cross-volume moves and recovery.

## Consequences

Users can recover files through the system Trash and the application history. Deletion remains an asynchronous operation that can fail, and permanent deletion requires a separate, clearly confirmed path with loss-of-data tests.
