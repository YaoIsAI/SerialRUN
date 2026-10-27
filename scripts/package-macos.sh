#!/usr/bin/env bash
# Build a macOS .app bundle, ZIP and DMG for SerialRUN.
#
# Usage: ./scripts/package-macos.sh <target-triple> <version>
#   target-triple: aarch64-apple-darwin (Apple Silicon) or x86_64-apple-darwin (Intel)
#
# Environment:
#   CODESIGN_IDENTITY  Optional Developer ID identity. When unset the bundle is
#                      ad-hoc signed, which is enough for local use but macOS
#                      Gatekeeper will still ask the user to confirm first launch.

set -euo pipefail

TARGET="${1:?usage: package-macos.sh <target-triple> <version>}"
VERSION="${2:?usage: package-macos.sh <target-triple> <version>}"
VERSION="${VERSION#v}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

case "$TARGET" in
    aarch64-apple-darwin) ARCH="arm64"; TARGET_ARCH="arm64" ;;
    x86_64-apple-darwin)  ARCH="x64";   TARGET_ARCH="x86_64" ;;
    *) echo "Error: unsupported macOS target '$TARGET'" >&2; exit 1 ;;
esac

BIN="target/$TARGET/release/serialrun"
if [[ ! -f "$BIN" ]]; then
    echo "Error: $BIN not found. Build it first:" >&2
    echo "  cargo build --release --target $TARGET -p serialrun-gui" >&2
    exit 1
fi

# Guard against accidentally shipping the wrong architecture (the v0.4.0
# "universal" download was actually arm64-only).
if ! file "$BIN" | grep -q "Mach-O 64-bit executable $TARGET_ARCH"; then
    echo "Error: $BIN does not match target architecture $TARGET_ARCH" >&2
    file "$BIN" >&2
    exit 1
fi

DIST="dist"
APP="$DIST/SerialRUN.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/docs"

cp "$BIN" "$APP/Contents/MacOS/serialrun"
cp crates/serialrun-gui/Info.plist "$APP/Contents/Info.plist"
cp crates/serialrun-gui/icon.icns "$APP/Contents/Resources/icon.icns"
cp docs/help_en.md docs/help_zh.md docs/MANUAL.md docs/MANUAL_CN.md "$APP/Contents/Resources/docs/" 2>/dev/null || true

# Guarantee the bundle advertises the release version even if the plist was
# not bumped beforehand.
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$APP/Contents/Info.plist" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$APP/Contents/Info.plist" 2>/dev/null || true

if [[ -n "${CODESIGN_IDENTITY:-}" ]]; then
    echo "Signing with identity: $CODESIGN_IDENTITY"
    codesign --force --deep --options runtime --timestamp --sign "$CODESIGN_IDENTITY" "$APP"
else
    echo "Ad-hoc signing (no Developer ID configured)"
    codesign --force --deep --sign - "$APP"
fi
codesign --verify --deep --strict "$APP"

STAGE="dist/SerialRUN-$VERSION-macos-$ARCH"
rm -rf "$STAGE"
mkdir -p "$STAGE/cli"
ditto "$APP" "$STAGE/SerialRUN.app"
for extra in serialrun-cli serialrun-mcp; do
    [[ -f "target/$TARGET/release/$extra" ]] && cp "target/$TARGET/release/$extra" "$STAGE/cli/"
done
cp LICENSE "$STAGE/" 2>/dev/null || true

cat > "$STAGE/INSTALL.txt" <<EOF
SerialRUN v$VERSION (macOS $ARCH)
=================================

1. Drag SerialRUN.app to the Applications folder.
2. First launch: right-click SerialRUN.app -> Open -> Open.
   (The app is not notarized, so macOS Gatekeeper asks for confirmation once.)

Optional CLI tools are in the cli/ folder:
    serialrun-cli   - command-line interface
    serialrun-mcp   - standalone MCP server

If macOS reports the app is "damaged" after downloading, run:

   xattr -dr com.apple.quarantine /Applications/SerialRUN.app

Then open it again.
EOF

ZIP="dist/serialrun-$VERSION-macos-$ARCH.zip"
DMG="dist/serialrun-$VERSION-macos-$ARCH.dmg"
rm -f "$ZIP" "$DMG"
ditto -c -k --sequesterRsrc --keepParent "$STAGE" "$ZIP"

ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "SerialRUN $VERSION" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null

rm -rf "$STAGE" "$APP"

echo "Created:"
echo "  $ZIP"
echo "  $DMG"
