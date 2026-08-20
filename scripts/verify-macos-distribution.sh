#!/usr/bin/env bash

set -euo pipefail

if [[ "$#" -ne 2 ]]; then
  echo "usage: $0 /path/to/App.app /path/to/App.dmg" >&2
  exit 2
fi

app_path="$1"
dmg_path="$2"

if [[ ! -d "$app_path" ]]; then
  echo "app bundle not found: $app_path" >&2
  exit 1
fi

if [[ ! -f "$dmg_path" ]]; then
  echo "disk image not found: $dmg_path" >&2
  exit 1
fi

codesign --verify --deep --strict --verbose=2 "$app_path"
xcrun stapler validate "$app_path"
spctl --assess --type execute --verbose=4 "$app_path"

codesign --verify --strict --verbose=2 "$dmg_path"
xcrun stapler validate "$dmg_path"
spctl --assess --type open --context context:primary-signature --verbose=4 "$dmg_path"
