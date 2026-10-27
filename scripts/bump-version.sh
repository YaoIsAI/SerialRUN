#!/usr/bin/env bash
# Bump the SerialRUN version everywhere it is user-visible.
#
# Usage: ./scripts/bump-version.sh 0.4.4
#
# Updates:
#   - crates/{core,cli,gui,mcp,plugin-api}/Cargo.toml
#   - Cargo.lock (via cargo metadata)
#   - status bar label in crates/serialrun-gui/src/ui/status.rs
#   - macOS Info.plist (CFBundleVersion / CFBundleShortVersionString)
#   - README.md / README_CN.md download tables

set -euo pipefail

VERSION="${1:-}"
VERSION="${VERSION#v}"

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>   (example: $0 0.4.4)" >&2
    exit 1
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: version must be X.Y.Z (got '$VERSION')" >&2
    exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OLD_VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/serialrun-gui/Cargo.toml | head -1)"
if [[ -z "$OLD_VERSION" ]]; then
    echo "Error: could not read current version from crates/serialrun-gui/Cargo.toml" >&2
    exit 1
fi

if [[ "$OLD_VERSION" == "$VERSION" ]]; then
    echo "Version is already $VERSION — nothing to do."
    exit 0
fi

if [[ "$(printf '%s\n%s\n' "$OLD_VERSION" "$VERSION" | sort -V | tail -1)" != "$VERSION" ]]; then
    echo "Error: new version $VERSION must be greater than current $OLD_VERSION" >&2
    exit 1
fi

replace() {
    local file="$1" expr="$2"
    local tmp
    tmp="$(mktemp)"
    sed "$expr" "$file" > "$tmp"
    mv "$tmp" "$file"
}

echo "Bumping SerialRUN $OLD_VERSION -> $VERSION"

for manifest in \
    crates/serialrun-core/Cargo.toml \
    crates/serialrun-cli/Cargo.toml \
    crates/serialrun-gui/Cargo.toml \
    crates/serialrun-mcp/Cargo.toml \
    crates/serialrun-plugin-api/Cargo.toml; do
    replace "$manifest" "s/^version = \"$OLD_VERSION\"/version = \"$VERSION\"/"
done

replace crates/serialrun-gui/src/ui/status.rs "s/SerialRUN v$OLD_VERSION/SerialRUN v$VERSION/"

# Keep both bundle-version keys in sync so Finder / About show the right version.
replace crates/serialrun-gui/Info.plist \
    "/<key>CFBundleVersion<\/key>/{n;s/<string>.*<\/string>/<string>$VERSION<\/string>/;}"
replace crates/serialrun-gui/Info.plist \
    "/<key>CFBundleShortVersionString<\/key>/{n;s/<string>.*<\/string>/<string>$VERSION<\/string>/;}"

# Only touch the download tables, never changelog history.
for readme in README.md README_CN.md; do
    [[ -f "$readme" ]] || continue
    replace "$readme" \
        "/^## Download\$/,/^### Build from Source\$/ s/v$OLD_VERSION/v$VERSION/g; /^## 下载\$/,/^### 从源码编译\$/ s/v$OLD_VERSION/v$VERSION/g; /^## Download\$/,/^### Build from Source\$/ s/serialrun-$OLD_VERSION-/serialrun-$VERSION-/g; /^## 下载\$/,/^### 从源码编译\$/ s/serialrun-$OLD_VERSION-/serialrun-$VERSION-/g"
done

# Refresh workspace package versions in Cargo.lock without upgrading dependencies.
if ! cargo metadata --format-version 1 > /dev/null 2>&1; then
    echo "Warning: cargo metadata failed; Cargo.lock may need a manual refresh." >&2
fi

echo "Updated:"
echo "  - workspace manifests + Cargo.lock"
echo "  - status bar label"
echo "  - Info.plist ($VERSION)"
echo "  - README download tables"
echo ""
echo "Next: review the diff, then commit and tag v$VERSION."
