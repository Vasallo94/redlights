#!/bin/bash
# Regenera assets/AppIcon.icns a partir de assets/icon.svg (solo macOS, sin dependencias).
set -euo pipefail
cd "$(dirname "$0")/.."

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
qlmanage -t -s 1024 -o "$TMP" assets/icon.svg >/dev/null
SET="$TMP/AppIcon.iconset"
mkdir "$SET"
for s in 16 32 128 256 512; do
    sips -z $s $s           "$TMP/icon.svg.png" --out "$SET/icon_${s}x${s}.png"    >/dev/null
    sips -z $((s*2)) $((s*2)) "$TMP/icon.svg.png" --out "$SET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$SET" -o assets/AppIcon.icns
echo "assets/AppIcon.icns actualizado"
