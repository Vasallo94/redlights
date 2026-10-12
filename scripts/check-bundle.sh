#!/bin/bash
# Comprueba que RedLights.app está bien empaquetado. Lo ejecuta el CI tras build.sh;
# también sirve en local. En el CI (CI=true) además arranca la app unos segundos.
set -euo pipefail
cd "$(dirname "$0")/.."

APP="RedLights.app"
BIN="$APP/Contents/MacOS/RedLights"
PLIST="$APP/Contents/Info.plist"
fail() { echo "FAIL: $*"; exit 1; }
ok() { echo "ok: $*"; }

# Binario universal y firma
archs="$(lipo -archs "$BIN")"
[[ "$archs" == *arm64* && "$archs" == *x86_64* ]] || fail "binario no universal: $archs"
ok "binario universal ($archs)"
codesign --verify --strict "$APP" || fail "firma no válida"
ok "firma válida"

# Info.plist
plutil -lint -s "$PLIST" || fail "Info.plist mal formado"
version="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$PLIST")"
if [[ "${GITHUB_REF_TYPE:-}" == "tag" ]]; then
    [[ "$version" == "${GITHUB_REF_NAME#v}" ]] || fail "versión $version ≠ tag $GITHUB_REF_NAME"
fi
ok "Info.plist (v$version)"
[[ -f "$APP/Contents/Resources/AppIcon.icns" ]] || fail "falta el icono"
ok "icono"

# Traducciones: cada clave usada en el código existe en cada idioma, y no sobran claves
code_keys="$(grep -oh 'NSLocalizedString("[^"]*"' Sources/*.swift | sed 's/^NSLocalizedString("//; s/"$//' | sort -u)"
for strings in "$APP"/Contents/Resources/*.lproj/Localizable.strings; do
    lang="$(basename "$(dirname "$strings")")"
    plutil -lint -s "$strings" || fail "$lang: Localizable.strings mal formado"
    file_keys="$(plutil -convert json -o - "$strings" | jq -r 'keys[]' | sort -u)"
    missing="$(comm -23 <(echo "$code_keys") <(echo "$file_keys"))"
    extra="$(comm -13 <(echo "$code_keys") <(echo "$file_keys"))"
    [[ -z "$missing" ]] || fail "$lang: faltan traducciones: $missing"
    [[ -z "$extra" ]] || fail "$lang: claves que el código ya no usa: $extra"
    ok "traducción $lang completa ($(echo "$file_keys" | wc -l | tr -d ' ') claves)"
done

# Arranque: que no se caiga al abrir (solo en el CI, para no tocar tu Mac)
if [[ -n "${CI:-}" ]]; then
    "$BIN" & pid=$!
    sleep 5
    kill -0 "$pid" 2>/dev/null || fail "la app se cerró al arrancar"
    kill "$pid"
    ok "arranca y sigue viva tras 5 s"
fi
