#!/bin/bash
# Release script — build, package, publish to GitHub + Gitea
#
# Usage:
#   ./scripts/release.sh v0.3.0              # Full release
#   ./scripts/release.sh v0.3.0 --dry-run   # Preview only
#   ./scripts/release.sh v0.3.0 --github    # GitHub only
#   ./scripts/release.sh v0.3.0 --gitea     # Gitea only
#
# Environment variables:
#   GITHUB_TOKEN  — GitHub personal access token
#   GITEA_TOKEN   — Gitea personal access token (from Gitea → Settings → Applications)
#   GITEA_URL     — Gitea base URL (default: http://192.168.31.85:38633)
#   GITEA_USER    — Gitea username (default: yao)
#   GITEA_REPO    — Gitea repo path (default: yao/serialrun)

set -euo pipefail

# ── Config ──
GITHUB_REPO="YaoIsAI/SerialRUN"
GITEA_URL="${GITEA_URL:-http://192.168.31.85:38633}"
GITEA_USER="${GITEA_USER:-yao}"
GITEA_REPO="${GITEA_REPO:-yao/serialrun}"
GITEA_TOKEN="${GITEA_TOKEN:-}"

VERSION="${1:-}"
DRY_RUN=false
PUSH_GITHUB=true
PUSH_GITEA=true

# Parse args
shift || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run) DRY_RUN=true; shift ;;
        --github) PUSH_GITEA=false; shift ;;
        --gitea) PUSH_GITHUB=false; shift ;;
        *) shift ;;
    esac
done

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version> [--dry-run] [--github] [--gitea]"
    echo "Example: $0 v0.3.0"
    exit 1
fi

log() { echo "[$(date '+%H:%M:%S')] $*"; }

# ── Platform target ──
TARGET="x86_64-pc-windows-msvc"
BUILD_DIR="target/$TARGET/release"

# ── Step 1: Build ──
log "Building release (target: $TARGET)..."
taskkill //F //IM serialrun.exe 2>/dev/null || true
cargo build --target "$TARGET" --release -p serialrun-gui
log "Build complete."

# ── Step 2: Package ──
log "Packaging release..."
RELEASE_DIR="/tmp/serialrun-release-$$"
mkdir -p "$RELEASE_DIR"

# Copy exe + help files
cp "$BUILD_DIR/serialrun.exe" "$RELEASE_DIR/"
for f in docs/help_en.md docs/help_zh.md docs/MANUAL.md docs/MANUAL_CN.md; do
    [[ -f "$f" ]] && cp "$f" "$RELEASE_DIR/"
done

# Create ZIP
ZIP_NAME="serialrun-${VERSION#v}-windows-x64.zip"
cd "$RELEASE_DIR"
zip -r "/tmp/$ZIP_NAME" .
cd -
log "Package created: /tmp/$ZIP_NAME ($(du -h /tmp/$ZIP_NAME | cut -f1))"

if [[ "$DRY_RUN" == "true" ]]; then
    log "DRY RUN: Would publish to:"
    [[ "$PUSH_GITHUB" == "true" ]] && log "  - GitHub: $GITHUB_REPO"
    [[ "$PUSH_GITEA" == "true" ]] && log "  - Gitea: $GITEA_REPO"
    log "ZIP: /tmp/$ZIP_NAME"
    rm -rf "$RELEASE_DIR"
    exit 0
fi

# ── Step 3: Tag ──
log "Creating git tag..."
git tag -a "$VERSION" -m "Release $VERSION" 2>/dev/null || log "Tag $VERSION already exists"

# ── Step 4: Push to GitHub ──
if [[ "$PUSH_GITHUB" == "true" ]]; then
    log "Publishing to GitHub..."
    GH_TOKEN="${GITHUB_TOKEN:?Please set GITHUB_TOKEN}"

    # Create release
    RELEASE_RESP=$(curl -s -X POST "https://api.github.com/repos/$GITHUB_REPO/releases" \
        -H "Authorization: token $GH_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        -d "{\"tag_name\":\"$VERSION\",\"name\":\"$VERSION\",\"body\":\"SerialRUN $VERSION\"}")

    RELEASE_ID=$(echo "$RELEASE_RESP" | jq -r '.id // empty')
    if [[ -z "$RELEASE_ID" ]]; then
        log "WARNING: GitHub release may already exist, trying to upload asset..."
        RELEASE_ID=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases" \
            -H "Authorization: token $GH_TOKEN" | jq -r ".[] | select(.tag_name==\"$VERSION\") | .id")
    fi

    if [[ -n "$RELEASE_ID" ]]; then
        # Delete existing asset if any
        OLD_ASSETS=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/$RELEASE_ID/assets" \
            -H "Authorization: token $GH_TOKEN" | jq -r '.[].id')
        for aid in $OLD_ASSETS; do
            curl -s -X DELETE "https://api.github.com/repos/$GITHUB_REPO/releases/assets/$aid" \
                -H "Authorization: token $GH_TOKEN" > /dev/null
        done

        # Upload
        curl -s -X POST "https://uploads.github.com/repos/$GITHUB_REPO/releases/$RELEASE_ID/assets?name=$ZIP_NAME" \
            -H "Authorization: token $GH_TOKEN" \
            -H "Content-Type: application/zip" \
            --data-binary @"/tmp/$ZIP_NAME" | jq -r '.browser_download_url'
        log "GitHub release published."
    else
        log "ERROR: Failed to create GitHub release."
    fi

    # Push tag
    git push github "$VERSION" 2>/dev/null || true
fi

# ── Step 5: Push to Gitea ──
if [[ "$PUSH_GITEA" == "true" ]]; then
    log "Publishing to Gitea..."
    GT_TOKEN="${GITEA_TOKEN:?Please set GITEA_TOKEN}"

    # Create release
    RELEASE_RESP=$(curl -s -X POST "$GITEA_URL/api/v1/repos/$GITEA_REPO/releases" \
        -H "Authorization: token $GT_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"tag_name\":\"$VERSION\",\"name\":\"$VERSION\",\"body\":\"SerialRUN $VERSION\"}")

    RELEASE_ID=$(echo "$RELEASE_RESP" | jq -r '.id // empty')

    if [[ -n "$RELEASE_ID" ]]; then
        # Upload
        curl -s -X POST "$GITEA_URL/api/v1/repos/$GITEA_REPO/releases/$RELEASE_ID/assets?name=$ZIP_NAME" \
            -H "Authorization: token $GT_TOKEN" \
            -H "Content-Type: application/zip" \
            --data-binary @"/tmp/$ZIP_NAME" | jq -r '.browser_download_url // "upload ok"'
        log "Gitea release published."
    else
        log "ERROR: Failed to create Gitea release."
    fi

    # Push tag
    git push origin "$VERSION" 2>/dev/null || true
fi

# ── Cleanup ──
rm -rf "$RELEASE_DIR" "/tmp/$ZIP_NAME"
log "Release $VERSION complete!"
