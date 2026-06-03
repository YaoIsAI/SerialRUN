#!/bin/bash
# Auto-implement: reads GitHub Issues with "auto-implement" label,
# generates task file, runs Claude Code to implement, updates Issue.
#
# Usage:
#   ./scripts/auto-implement.sh              # Process all open issues
#   ./scripts/auto-implement.sh --dry-run    # Preview without executing
#   ./scripts/auto-implement.sh --issue 123  # Process specific issue

set -euo pipefail

REPO="YaoIsAI/SerialRUN"
TOKEN="${GITHUB_TOKEN:?Please set GITHUB_TOKEN environment variable}"
API="https://api.github.com"
TASK_FILE=".auto-task.md"
DRY_RUN=false
SPECIFIC_ISSUE=""

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run) DRY_RUN=true; shift ;;
        --issue) SPECIFIC_ISSUE="$2"; shift 2 ;;
        *) shift ;;
    esac
done

log() { echo "[$(date '+%H:%M:%S')] $*"; }

# Fetch open issues with auto-implement label
fetch_issues() {
    if [[ -n "$SPECIFIC_ISSUE" ]]; then
        curl -s -H "Authorization: token $TOKEN" \
            "$API/repos/$REPO/issues/$SPECIFIC_ISSUE" | \
            jq -r '[{number: .number, title: .title, body: (.body // ""), labels: [.labels[].name]}]'
    else
        curl -s -H "Authorization: token $TOKEN" \
            "$API/repos/$REPO/issues?labels=auto-implement&state=open&per_page=10" | \
            jq -r '[.[] | {number: .number, title: .title, body: (.body // ""), labels: [.labels[].name]}]'
    fi
}

# Generate task file from issue
generate_task() {
    local number="$1"
    local title="$2"
    local body="$3"

    cat > "$TASK_FILE" << EOF
# 自动任务

## 来源
- Issue: #$number
- 标题: $title

## 需求描述
$body

## 约束
- 只修改必要的文件
- 必须通过 \`cargo build\`
- 不修改 .gitignore 和 .git
- 不修改 GUI 代码（proprietary）
- 只修改 serialrun-core / serialrun-plugin-api / plugins / docs
- Commit message 格式: "auto(issue-$number): $title"
EOF

    log "Task file generated: $TASK_FILE"
}

# Comment on issue
comment_issue() {
    local number="$1"
    local body="$2"
    curl -s -X POST -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "$API/repos/$REPO/issues/$number/comments" \
        -d "{\"body\": \"$body\"}" > /dev/null
}

# Close issue
close_issue() {
    local number="$1"
    curl -s -X PATCH -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "$API/repos/$REPO/issues/$number" \
        -d '{"state": "closed"}' > /dev/null
}

# Main
log "Fetching issues..."
ISSUES=$(fetch_issues)
COUNT=$(echo "$ISSUES" | jq 'length')

if [[ "$COUNT" -eq 0 ]]; then
    log "No open issues with auto-implement label found."
    exit 0
fi

log "Found $COUNT issue(s) to process."

echo "$ISSUES" | jq -c '.[]' | while read -r issue; do
    NUMBER=$(echo "$issue" | jq -r '.number')
    TITLE=$(echo "$issue" | jq -r '.title')
    BODY=$(echo "$issue" | jq -r '.body')

    log "Processing Issue #$NUMBER: $TITLE"

    # Generate task
    generate_task "$NUMBER" "$TITLE" "$BODY"

    if [[ "$DRY_RUN" == "true" ]]; then
        log "DRY RUN: Would execute Claude Code on $TASK_FILE"
        cat "$TASK_FILE"
        continue
    fi

    # Comment: processing started
    comment_issue "$NUMBER" "🤖 **Auto-implement started**\n\nClaude Code is now working on this issue.\n\nTask: $TITLE"

    # Run Claude Code (non-interactive)
    log "Running Claude Code..."
    PROMPT="Read and execute the task in $TASK_FILE. After completion, run \`cargo build\` to verify. Then commit with message 'auto(issue-$NUMBER): $TITLE'. Do NOT modify GUI code (serialrun-gui)."

    if claude -p "$PROMPT" --allowedTools "Bash,Read,Write,Edit,Glob,Grep" 2>&1; then
        log "Issue #$NUMBER completed successfully."

        # Comment: done
        comment_issue "$NUMBER" "✅ **Auto-implement completed**\n\nChanges committed. Build verified.\n\nTask: $TITLE"

        # Close issue
        close_issue "$NUMBER"
        log "Issue #$NUMBER closed."
    else
        log "Issue #$NUMBER failed."

        # Comment: failed
        comment_issue "$NUMBER" "❌ **Auto-implement failed**\n\nClaude Code encountered an error. Manual intervention needed.\n\nTask: $TITLE"
    fi

    # Clean up
    rm -f "$TASK_FILE"
done

log "All issues processed."
