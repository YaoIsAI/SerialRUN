#!/bin/bash
# Monitor for new plugin development requests
#
# Usage:
#   ./scripts/monitor_plugins.sh              # Check once
#   ./scripts/monitor_plugins.sh --watch      # Watch continuously
#   ./scripts/monitor_plugins.sh --interval 60 # Check every 60 seconds

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PENDING_DIR="$HOME/.serialrun/pending_plugins"
PROCESSED_DIR="$HOME/.serialrun/processed_plugins"
LOG_FILE="$HOME/.serialrun/monitor.log"

WATCH=false
INTERVAL=30

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --watch) WATCH=true; shift ;;
        --interval) INTERVAL="$2"; shift 2 ;;
        *) shift ;;
    esac
done

log() {
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $*"
    echo "[$timestamp] $*" >> "$LOG_FILE"
}

process_request() {
    local request_file="$1"
    local filename=$(basename "$request_file")
    local issue_num=$(jq -r '.issue_number' "$request_file")

    log "Processing plugin request: $filename"

    # Mark as processing
    jq '.status = "processing"' "$request_file" > "$request_file.tmp"
    mv "$request_file.tmp" "$request_file"

    # Run auto-development
    if "$SCRIPT_DIR/auto_develop_plugin.sh" "$request_file" 2>&1 | tee -a "$LOG_FILE"; then
        log "✅ Plugin development completed for Issue #$issue_num"

        # Move to processed
        mkdir -p "$PROCESSED_DIR"
        mv "$request_file" "$PROCESSED_DIR/$filename"
    else
        log "❌ Plugin development failed for Issue #$issue_num"
        jq '.status = "failed"' "$request_file" > "$request_file.tmp"
        mv "$request_file.tmp" "$request_file"
    fi
}

check_pending() {
    if [[ ! -d "$PENDING_DIR" ]]; then
        return
    fi

    local count=0
    for request_file in "$PENDING_DIR"/issue-*.json; do
        [[ -f "$request_file" ]] || continue

        local status=$(jq -r '.status' "$request_file")
        if [[ "$status" == "pending" ]]; then
            process_request "$request_file"
            ((count++))
        fi
    done

    if [[ $count -eq 0 ]]; then
        log "No pending requests"
    fi
}

# Main
mkdir -p "$PENDING_DIR" "$PROCESSED_DIR"

log "Monitor started"
log "Watching: $PENDING_DIR"

if [[ "$WATCH" == "true" ]]; then
    log "Mode: continuous (interval: ${INTERVAL}s)"
    log "Press Ctrl+C to stop"
    echo ""

    while true; do
        check_pending
        sleep "$INTERVAL"
    done
else
    check_pending
fi
