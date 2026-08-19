#!/usr/bin/env bash
# Fail unless the Mach-O is a universal binary (arm64 + x86_64) targeting macOS 12+.
set -euo pipefail

bin="${1:?usage: verify-universal.sh path/to/binary}"

if [[ ! -f "$bin" ]]; then
  echo "not a file: $bin" >&2
  exit 1
fi

archs="$(lipo -archs "$bin")"
echo "lipo -archs: $archs"

has_arm=0
has_x86=0
for arch in $archs; do
  case "$arch" in
    arm64 | arm64e) has_arm=1 ;;
    x86_64) has_x86=1 ;;
  esac
done

if [[ "$has_arm" -ne 1 ]]; then
  echo "missing Apple Silicon slice (arm64)" >&2
  exit 1
fi
if [[ "$has_x86" -ne 1 ]]; then
  echo "missing Intel slice (x86_64)" >&2
  exit 1
fi

vtool -show-build "$bin"
vtool -show-build "$bin" | python3 -c '
import re, sys
text = sys.stdin.read()
minos = re.findall(r"\bminos\s+(\d+(?:\.\d+)*)", text)
if not minos:
    raise SystemExit("could not read minos from vtool -show-build")
print("minos:", ", ".join(minos))
for raw in minos:
    parts = raw.split(".")
    major = int(parts[0])
    minor = int(parts[1]) if len(parts) > 1 else 0
    if (major, minor) < (12, 0):
        raise SystemExit(f"minimum OS {raw} is below macOS 12.0")
'
