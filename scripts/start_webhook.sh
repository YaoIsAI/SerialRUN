#!/bin/bash
# Start webhook server + Cloudflare Tunnel + Monitor
#
# Usage:
#   ./scripts/start_webhook.sh              # Start all
#   ./scripts/start_webhook.sh --tunnel     # Start with tunnel
#   ./scripts/start_webhook.sh --local      # Local only (no tunnel)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$HOME/.serialrun"
CONFIG_FILE="$CONFIG_DIR/webhook_config.json"

# Default values
PORT=9876
SECRET=""
USE_TUNNEL=true

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --tunnel) USE_TUNNEL=true; shift ;;
        --local) USE_TUNNEL=false; shift ;;
        *) shift ;;
    esac
done

# Load or create config
if [[ -f "$CONFIG_FILE" ]]; then
    PORT=$(jq -r '.port' "$CONFIG_FILE")
    SECRET=$(jq -r '.secret' "$CONFIG_FILE")
    DOMAIN=$(jq -r '.domain' "$CONFIG_FILE")
else
    echo "No config found. Running setup..."
    "$SCRIPT_DIR/setup_webhook.sh"
    PORT=$(jq -r '.port' "$CONFIG_FILE")
    SECRET=$(jq -r '.secret' "$CONFIG_FILE")
    DOMAIN=$(jq -r '.domain' "$CONFIG_FILE")
fi

echo "╔══════════════════════════════════════════════════╗"
echo "║  SerialRUN Webhook System                       ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Port: $PORT"
echo "Domain: $DOMAIN"
echo "Tunnel: $USE_TUNNEL"
echo ""

# Start webhook server
echo "Starting webhook server on port $PORT..."
python3 "$SCRIPT_DIR/webhook_server.py" \
    --port "$PORT" \
    --secret "$SECRET" &
WEBHOOK_PID=$!
echo "Webhook server PID: $WEBHOOK_PID"

# Start tunnel if enabled
TUNNEL_PID=""
if [[ "$USE_TUNNEL" == "true" ]]; then
    echo "Starting Cloudflare Tunnel..."
    cloudflared tunnel --config "$CONFIG_DIR/config.yml" run &
    TUNNEL_PID=$!
    echo "Tunnel PID: $TUNNEL_PID"
    echo ""
    echo "🔗 Webhook URL: https://$DOMAIN/webhook"
else
    echo ""
    echo "🔗 Webhook URL: http://localhost:$PORT/webhook"
fi

echo ""
echo "📡 Monitor: ./scripts/monitor_plugins.sh --watch"
echo ""
echo "Press Ctrl+C to stop all services"
echo ""

# Cleanup on exit
cleanup() {
    echo ""
    echo "Shutting down..."
    kill $WEBHOOK_PID 2>/dev/null || true
    [[ -n "$TUNNEL_PID" ]] && kill $TUNNEL_PID 2>/dev/null || true
    echo "Done."
}
trap cleanup EXIT INT TERM

# Wait for any process to exit
wait
