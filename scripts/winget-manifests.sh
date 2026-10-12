#!/bin/bash
# Genera los manifiestos de winget (Vasallo94.RedLights) para una versión ya publicada.
#
#     scripts/winget-manifests.sh 0.3.1 salida/
#
# Crea salida/manifests/v/Vasallo94/RedLights/<versión>/ con los tres YAML, listos para
# validar (`winget validate`) o para un PR a microsoft/winget-pkgs.
set -euo pipefail

VERSION="${1:?uso: $0 VERSIÓN DIRECTORIO}"
OUT="${2:?uso: $0 VERSIÓN DIRECTORIO}"
ID="Vasallo94.RedLights"
REPO="https://github.com/Vasallo94/redlights"
URL="$REPO/releases/download/v$VERSION/redlights-windows-x64.zip"
SCHEMA="1.12.0"

SHA="$(curl -fsSL "$URL.sha256" | cut -d' ' -f1 | tr '[:lower:]' '[:upper:]')"
[[ "$SHA" =~ ^[0-9A-F]{64}$ ]] || { echo "SHA-256 no válido para $URL" >&2; exit 1; }
DATE="$(gh release view "v$VERSION" --repo Vasallo94/redlights --json publishedAt -q '.publishedAt[:10]')"

DIR="$OUT/manifests/v/Vasallo94/RedLights/$VERSION"
mkdir -p "$DIR"

cat > "$DIR/$ID.yaml" <<EOF
PackageIdentifier: $ID
PackageVersion: $VERSION
DefaultLocale: en-US
ManifestType: version
ManifestVersion: $SCHEMA
EOF

cat > "$DIR/$ID.installer.yaml" <<EOF
PackageIdentifier: $ID
PackageVersion: $VERSION
InstallerType: zip
NestedInstallerType: portable
ReleaseDate: $DATE
Installers:
- Architecture: x64
  NestedInstallerFiles:
  - RelativeFilePath: redlights.exe
    PortableCommandAlias: redlights
  InstallerUrl: $URL
  InstallerSha256: $SHA
ManifestType: installer
ManifestVersion: $SCHEMA
EOF

cat > "$DIR/$ID.locale.en-US.yaml" <<EOF
PackageIdentifier: $ID
PackageVersion: $VERSION
PackageLocale: en-US
Publisher: Vasallo94
PublisherUrl: https://github.com/Vasallo94
PublisherSupportUrl: $REPO/issues
Author: Vasallo94
PackageName: RedLights
PackageUrl: $REPO
License: MIT
LicenseUrl: $REPO/blob/main/LICENSE
Copyright: Copyright (c) 2026 Vasallo94
ShortDescription: Turn every display pure red for astronomy without losing your night vision.
Description: |-
  RedLights changes the colors your displays output so that green and blue are
  zero and red is dimmed to the level you choose, keeping your eyes adapted to
  the dark while you use a computer next to the telescope. It uses the Windows
  Magnification API color matrix: no screen capture, no overlay windows, and
  normal colors come back when it exits.
Moniker: redlights
Tags:
- astronomy
- astrophotography
- dark-mode
- night-vision
- red-light
- screen-filter
ReleaseNotesUrl: $REPO/releases/tag/v$VERSION
ManifestType: defaultLocale
ManifestVersion: $SCHEMA
EOF

echo "$DIR"
