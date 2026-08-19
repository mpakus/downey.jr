#!/usr/bin/env python3
"""Print the Keep a Changelog section for a version (for GitHub Releases)."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

SAMPLE = """# Changelog

## [Unreleased]

### Added

- not this

## [0.1.0] - 2026-08-19

### Added

- first release

## [0.0.1] - 2026-01-01

- older

[Unreleased]: https://example.com/compare/v0.1.0...HEAD
[0.1.0]: https://example.com/releases/tag/v0.1.0
"""


def normalize_version(raw: str) -> str:
    text = raw.strip()
    if text.startswith("v") or text.startswith("V"):
        return text[1:]
    return text


def section_for(changelog: str, version: str) -> str:
    version = normalize_version(version)
    heading = re.compile(
        rf"^## \[{re.escape(version)}\](?:\s+-.*)?\s*$",
        re.MULTILINE,
    )
    match = heading.search(changelog)
    if match is None:
        raise SystemExit(f"CHANGELOG.md has no section for [{version}]")

    start = match.end()
    next_heading = re.search(r"^## \[", changelog[start:], re.MULTILINE)
    next_link = re.search(r"^\[", changelog[start:], re.MULTILINE)
    end = len(changelog)
    if next_heading is not None:
        end = min(end, start + next_heading.start())
    if next_link is not None:
        end = min(end, start + next_link.start())

    title = match.group(0).strip()
    body = changelog[start:end].strip()
    if not body:
        raise SystemExit(f"CHANGELOG.md section [{version}] is empty")
    return f"{title}\n\n{body}\n"


UNSIGNED_FOOTER = """
---

Universal macOS build (Apple Silicon `arm64` + Intel `x86_64`), minimum macOS 12.0.

**Unsigned test build.** Gatekeeper will warn until Developer ID signing and notarization. First launch: Right-click → Open, or `xattr -cr /Applications/1537paperstreet.app` after copying.
"""


def self_test() -> None:
    got = section_for(SAMPLE, "v0.1.0")
    assert "## [0.1.0] - 2026-08-19" in got
    assert "first release" in got
    assert "not this" not in got
    assert "older" not in got
    try:
        section_for(SAMPLE, "9.9.9")
    except SystemExit:
        pass
    else:
        raise AssertionError("missing version should fail")
    notes = section_for(SAMPLE, "0.1.0") + UNSIGNED_FOOTER
    assert "Unsigned test build" in notes


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "version",
        nargs="?",
        help="Version or git tag, e.g. 0.1.0 or v0.1.0",
    )
    parser.add_argument(
        "--changelog",
        type=Path,
        default=Path("CHANGELOG.md"),
        help="Path to CHANGELOG.md",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run built-in assertions and exit",
    )
    parser.add_argument(
        "--unsigned-footer",
        action="store_true",
        help="Append the Gatekeeper warning used on GitHub Releases",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        self_test()
        return 0

    if not args.version:
        parser.error("version is required unless --self-test")

    text = args.changelog.read_text(encoding="utf-8")
    sys.stdout.write(section_for(text, args.version))
    if args.unsigned_footer:
        sys.stdout.write(UNSIGNED_FOOTER)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
