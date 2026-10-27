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
rm -rf "$STAGE"

echo "Created: $TARBALL"
