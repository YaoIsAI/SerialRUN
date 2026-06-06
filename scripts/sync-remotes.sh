#!/bin/bash
# Sync serialrun to GitHub (public) and Gitea (private)
#
# Strategy:
#   - master → GitHub  (public, .gitignore excludes GUI/internal/build artifacts)
#   - full   → Gitea   (private, everything)
#
# Usage:
#   ./scripts/sync-remotes.sh              # Push both
#   ./scripts/sync-remotes.sh --github     # GitHub only
#   ./scripts/sync-remotes.sh --gitea      # Gitea only
#   ./scripts/sync-remotes.sh --dry-run    # Preview only

set -euo pipefail

PUSH_GITHUB=true
PUSH_GITEA=true
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --github)  PUSH_GITEA=false; shift ;;
        --gitea)   PUSH_GITHUB=false; shift ;;
        --dry-run) DRY_RUN=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Pre-flight checks ──
echo "=== Pre-flight ==="

# Ensure working tree is clean (untracked files are OK)
if ! git diff --quiet HEAD 2>/dev/null; then
    echo "ERROR: Uncommitted changes. Commit or stash first."
    exit 1
fi

# Verify branches exist
for branch in master full; do
    if ! git rev-parse --verify "$branch" >/dev/null 2>&1; then
        echo "ERROR: Branch '$branch' not found locally."
        exit 1
    fi
done

# ── GitHub: push master ──
if $PUSH_GITHUB; then
    echo ""
    echo "=== GitHub (master → github/master) ==="
    echo "Files that WILL be pushed:"
    git diff --stat github/master..master 2>/dev/null || echo "(up to date)"

    # Check for sensitive files
    SENSITIVE=$(git diff --name-only github/master..master 2>/dev/null | grep -E '\.(env|key|pem|p12|jks)$' || true)
    if [[ -n "$SENSITIVE" ]]; then
        echo "WARNING: Sensitive files detected:"
        echo "$SENSITIVE"
        exit 1
    fi

    if $DRY_RUN; then
        echo "[DRY RUN] Would push master to github/master"
    else
        git push github master:master
        echo "✓ GitHub updated"
    fi
fi

# ── Gitea: push full ──
if $PUSH_GITEA; then
    echo ""
    echo "=== Gitea (full → origin/full) ==="
    echo "Ahead of Gitea:"
    git log --oneline origin/full..full 2>/dev/null || echo "(up to date)"

    if $DRY_RUN; then
        echo "[DRY RUN] Would push full to origin/full"
    else
        git push origin full
        echo "✓ Gitea updated"
    fi
fi

echo ""
echo "=== Done ==="
