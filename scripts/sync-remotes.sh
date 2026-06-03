#!/bin/bash
# Sync to both GitHub and Gitea with correct URLs per branch.
#
# Usage:
#   ./scripts/sync-remotes.sh              # Push master (GitHub URLs) + gitea branch (Gitea URLs)
#   ./scripts/sync-remotes.sh "commit msg" # Auto-commit then sync
#
# Branches:
#   master  → GitHub (public, open source only)
#   gitea   → Gitea (private, URLs swapped to Gitea)
#   full    → Gitea (private, includes GUI + website)

set -e

GITHUB_URL="https://github.com/YaoIsAI/SerialRUN.git"
GITEA_URL="http://192.168.31.85:38633/yao/serialrun.git"
GITEA_BASE="http://192.168.31.85:38633/yao/serialrun"
GITHUB_BASE="https://github.com/YaoIsAI/SerialRUN"

# Auto-commit if message provided
if [ -n "$1" ]; then
    git add -A
    git commit -m "$1"
fi

# --- Push master to GitHub ---
echo ">>> Pushing master to GitHub..."
git checkout master
git push github master

# --- Sync gitea branch ---
echo ">>> Syncing gitea branch..."
git checkout gitea
git merge master --no-edit -q

# Swap GitHub URLs to Gitea
sed -i "s|${GITHUB_BASE}/releases/download|${GITEA_BASE}/releases/download|g" README.md README_CN.md
sed -i "s|${GITHUB_BASE}/releases|${GITEA_BASE}/releases|g" README.md README_CN.md
sed -i "s|${GITHUB_BASE}.git|${GITEA_BASE}.git|g" README.md README_CN.md

# Only commit if there are changes
if ! git diff --quiet; then
    git add README.md README_CN.md
    git commit -m "chore: swap URLs to Gitea" -q
fi

echo ">>> Pushing gitea branch to Gitea..."
git push origin gitea

# --- Push master to Gitea (with GitHub URLs, for reference) ---
echo ">>> Pushing master to Gitea..."
git checkout master
git push origin master

# --- Push full branch to Gitea ---
echo ">>> Pushing full branch to Gitea..."
git push origin full 2>/dev/null || echo "(full branch not updated, push manually if needed)"

echo ">>> Done! All remotes synced."
git checkout master
