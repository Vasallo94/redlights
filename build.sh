#!/bin/bash
# Compila RedLights como binario universal (Apple Silicon + Intel) y lo empaqueta
# en RedLights.app. Con --zip genera además RedLights-<versión>.zip para publicar.
#
# La versión sale del último tag de git (v0.2.0 → 0.2.0); se puede forzar con
# VERSION=1.2.3 ./build.sh
set -euo pipefail
cd "$(dirname "$0")"

NAME="RedLights"
APP="$NAME.app"
BUNDLE_ID="es.quique.redlights"
VERSION="${VERSION:-$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || true)}"
VERSION="${VERSION:-0.0.0}"
BUILD="$(git rev-list --count HEAD 2>/dev/null || echo 1)"

# Una compilación por arquitectura y lipo para unirlas: funciona con las
# Command Line Tools, sin necesidad de Xcode completo.
BINS=()
for arch in arm64 x86_64; do
    swift build -c release --arch "$arch"
    BINS+=("$(swift build -c release --arch "$arch" --show-bin-path)/$NAME")
done

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create "${BINS[@]}" -output "$APP/Contents/MacOS/$NAME"
cp assets/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>            <string>${NAME}</string>
    <key>CFBundleDisplayName</key>     <string>${NAME}</string>
    <key>CFBundleIdentifier</key>      <string>${BUNDLE_ID}</string>
    <key>CFBundleExecutable</key>      <string>${NAME}</string>
    <key>CFBundleIconFile</key>        <string>AppIcon</string>
    <key>CFBundlePackageType</key>     <string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${VERSION}</string>
    <key>CFBundleVersion</key>         <string>${BUILD}</string>
    <key>LSMinimumSystemVersion</key>  <string>13.0</string>
    <key>LSUIElement</key>             <true/>
    <key>NSHighResolutionCapable</key> <true/>
</dict>
</plist>
PLIST

# Firma ad-hoc: suficiente para tu propio Mac. Para distribuir sin avisos de
# Gatekeeper haría falta Developer ID + notarización.
codesign --force --sign - "$APP"

echo "Listo: $(pwd)/$APP  (v$VERSION, build $BUILD, $(lipo -archs "$APP/Contents/MacOS/$NAME"))"

if [[ "${1:-}" == "--zip" ]]; then
    ZIP="$NAME-$VERSION.zip"
    rm -f "$ZIP"
    # ditto conserva la firma y los atributos del bundle; zip a secas puede romperlos.
    ditto -c -k --keepParent "$APP" "$ZIP"
    echo "Zip: $(pwd)/$ZIP"
fi
