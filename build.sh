#!/bin/bash
# Compila RedScreen y lo empaqueta como RedScreen.app (solo macOS).
set -euo pipefail
cd "$(dirname "$0")"

APP="RedScreen.app"
BUNDLE_ID="es.quique.redscreen"
VERSION="0.1.0"

swift build -c release
BIN="$(swift build -c release --show-bin-path)/RedScreen"

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/RedScreen"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>            <string>RedScreen</string>
    <key>CFBundleDisplayName</key>     <string>RedScreen</string>
    <key>CFBundleIdentifier</key>      <string>${BUNDLE_ID}</string>
    <key>CFBundleExecutable</key>      <string>RedScreen</string>
    <key>CFBundlePackageType</key>     <string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${VERSION}</string>
    <key>CFBundleVersion</key>         <string>1</string>
    <key>LSMinimumSystemVersion</key>  <string>13.0</string>
    <key>LSUIElement</key>             <true/>
    <key>NSHighResolutionCapable</key> <true/>
</dict>
</plist>
PLIST

# Firma ad-hoc: suficiente para usarla en tu propio Mac.
codesign --force --sign - "$APP"

echo "Listo: $(pwd)/$APP"
echo "Ábrela con: open $APP   (o arrástrala a /Applications)"
