# ADR-007: Do not enable App Sandbox in version 1

- Status: accepted
- Date: 2026-08-13

## Context

The app must retain access to arbitrary user-selected project folders across launches. App Sandbox would require security-scoped bookmark storage and lifecycle handling for every project path.

## Decision

Ship version 1 outside the Mac App Store without App Sandbox. Sign with Developer ID, enable hardened runtime with minimal entitlements, notarize the build, and distribute it in a stapled DMG and Homebrew cask.

## Alternatives

- App Sandbox would improve OS-enforced isolation but substantially expands path persistence and filesystem-operation complexity.
- Mac App Store-only distribution would require the sandbox and constrain the planned filesystem workflow.

## Consequences

Arbitrary project folders work without bookmark infrastructure. Distribution requires Developer ID signing and notarization, and application code must strictly enforce project-root path boundaries because the sandbox does not provide that boundary.
