#!/usr/bin/env python3
"""Fail CI when crate line coverage is below the PLAN § 14 thresholds."""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

# Tauri command wrappers and the process entry point cannot be unit-tested
# without a WebView. They stay in the uploaded report but are excluded from
# the ps-app gate. incremental/history modules are gated once those files exist.
APP_IGNORE = {"commands.rs", "main.rs", "fs_watch.rs"}

THRESHOLDS = {
    "ps-core": 85.0,
    "ps-render": 85.0,
    "ps-app": 60.0,
}

FILE_THRESHOLDS = {
    "docio.rs": 95.0,
    "incremental.rs": 95.0,
    "history.rs": 95.0,
}


def crate_of(filename: str) -> str | None:
    parts = Path(filename).parts
    if "ps-core" in parts:
        return "ps-core"
    if "ps-render" in parts:
        return "ps-render"
    if "ps-app" in parts:
        return "ps-app"
    return None


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-coverage.py coverage/rust/cov.json", file=sys.stderr)
        return 2
    payload = json.loads(Path(sys.argv[1]).read_text())
    files = payload["data"][0]["files"]

    totals: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    failures: list[str] = []

    for entry in files:
        filename = entry["filename"]
        crate = crate_of(filename)
        if crate is None:
            continue
        name = Path(filename).name
        if crate == "ps-app" and name in APP_IGNORE:
            continue
        if "/tests/" in filename.replace("\\", "/"):
            continue
        lines = entry["summary"]["lines"]
        count = int(lines["count"])
        covered = int(lines["covered"])
        totals[crate][0] += count
        totals[crate][1] += covered

        if name in FILE_THRESHOLDS and count:
            percent = 100.0 * covered / count
            need = FILE_THRESHOLDS[name]
            if percent + 1e-9 < need:
                failures.append(
                    f"{name}: {percent:.2f}% lines, need ≥ {need:.0f}%"
                )

    for crate, need in THRESHOLDS.items():
        count, covered = totals[crate]
        if count == 0:
            failures.append(f"{crate}: no instrumented lines")
            continue
        percent = 100.0 * covered / count
        print(f"{crate}: {percent:.2f}% lines ({covered}/{count})")
        if percent + 1e-9 < need:
            failures.append(
                f"{crate}: {percent:.2f}% lines, need ≥ {need:.0f}%"
            )

    if failures:
        print("coverage gate failed:", file=sys.stderr)
        for item in failures:
            print(f"  {item}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
