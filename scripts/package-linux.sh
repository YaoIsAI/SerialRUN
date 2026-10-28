#!/usr/bin/env bash
# Package the Linux release as a self-contained tarball.
#
# Usage: ./scripts/package-linux.sh <version>
# Expects target/release/serialrun to be built already.

set -euo pipefail

VERSION="${1:?usage: package-linux.sh <version>}"
VERSION="${VERSION#v}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BIN="target/release/serialrun"
if [[ ! -f "$BIN" ]]; then
    echo "Error: $BIN not found. Build it first:" >&2
    echo "  cargo build --release -p serialrun-gui" >&2
    exit 1
fi

STAGE="dist/SerialRUN-$VERSION-linux-x64"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp "$BIN" "$STAGE/serialrun"
for extra in serialrun-cli serialrun-mcp; do
    [[ -f "target/release/$extra" ]] && cp "target/release/$extra" "$STAGE/"
done
cp docs/help_en.md docs/help_zh.md docs/MANUAL.md docs/MANUAL_CN.md "$STAGE/" 2>/dev/null || true
cp LICENSE "$STAGE/" 2>/dev/null || true

cat > "$STAGE/INSTALL.txt" <<EOF
SerialRUN v$VERSION (Linux x86_64)
==================================

Package contents:
  serialrun       - GUI application
  serialrun-cli   - command-line interface (list, monitor, record, replay, agent)
  serialrun-mcp   - standalone MCP server

Runtime dependencies (Ubuntu/Debian):
  sudo apt install libudev1 libxkbcommon0 libx11-6 libxcursor1 \\
                   libxrandr2 libxi6 libgl1 libwayland-client0

Run:
  chmod +x serialrun
  ./serialrun

To install system-wide:
  sudo install -m 755 serialrun serialrun-cli serialrun-mcp /usr/local/bin/
EOF

cat > "$STAGE/serialrun.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=SerialRUN
Comment=Serial port debugging assistant
Exec=serialrun
Icon=serialrun
Terminal=false
Categories=Development;Utility;
EOF

TARBALL="dist/serialrun-$VERSION-linux-x64.tar.gz"
rm -f "$TARBALL"
tar -C dist -czf "$TARBALL" "$(basename "$STAGE")"
echo "Created: $TARBALL"

# Build the Debian package (.deb) for Debian/Ubuntu users.
if command -v dpkg-deb >/dev/null 2>&1; then
    PKG="dist/serialrun_${VERSION}_amd64"
    rm -rf "$PKG"
    mkdir -p "$PKG/DEBIAN" \
             "$PKG/usr/bin" \
             "$PKG/usr/share/applications" \
             "$PKG/usr/share/icons/hicolor/512x512/apps" \
             "$PKG/usr/share/doc/serialrun"

    install -m 755 "$BIN" "$PKG/usr/bin/serialrun"
    for extra in serialrun-cli serialrun-mcp; do
        [[ -f "target/release/$extra" ]] && install -m 755 "target/release/$extra" "$PKG/usr/bin/"
    done
    install -m 644 assets/logo_square.png "$PKG/usr/share/icons/hicolor/512x512/apps/serialrun.png"
    install -m 644 "$STAGE/serialrun.desktop" "$PKG/usr/share/applications/serialrun.desktop"
    for doc in help_en.md help_zh.md MANUAL.md MANUAL_CN.md; do
        [[ -f "docs/$doc" ]] && install -m 644 "docs/$doc" "$PKG/usr/share/doc/serialrun/"
    done
    [[ -f LICENSE ]] && install -m 644 LICENSE "$PKG/usr/share/doc/serialrun/"

    INSTALLED_SIZE=$(du -sk "$PKG/usr" | cut -f1)
    cat > "$PKG/DEBIAN/control" <<EOF
Package: serialrun
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Installed-Size: $INSTALLED_SIZE
Maintainer: YaoIsAI <noreply@serialrun.com>
Homepage: https://www.serialrun.com
Depends: libudev1, libxkbcommon0, libx11-6, libxcursor1, libxrandr2, libxi6, libgl1, libwayland-client0, libxcb-render0, libxcb-shape0, libxcb-xfixes0
Description: Professional serial port debugging assistant
 SerialRUN is a cross-platform serial terminal with Modbus RTU/TCP,
 PLC control, CAN bus analysis, I2C/SPI tools, firmware flashing and a
 built-in MCP server for AI integration.
EOF

    DEB="dist/serialrun_${VERSION}_amd64.deb"
    rm -f "$DEB"
    dpkg-deb --build --root-owner-group "$PKG" "$DEB" >/dev/null
    rm -rf "$PKG"
    echo "Created: $DEB"
else
    echo "Warning: dpkg-deb not found; skipping .deb package" >&2
fi

rm -rf "$STAGE"
