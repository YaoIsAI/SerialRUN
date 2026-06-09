#!/bin/bash
# Setup Cloudflare Tunnel for SerialRUN webhook
#
# Usage:
#   ./scripts/setup_webhook.sh              # Interactive setup
#   ./scripts/setup_webhook.sh --port 9876  # Custom port
#   ./scripts/setup_webhook.sh --domain webhook.serialrun.com

set -euo pipefail

PORT=9876
DOMAIN="webhook.serialrun.com"
SECRET=""

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --port) PORT="$2"; shift 2 ;;
        --domain) DOMAIN="$2"; shift 2 ;;
        --secret) SECRET="$2"; shift 2 ;;
        *) shift ;;
    esac
done

# Generate secret if not provided
if [[ -z "$SECRET" ]]; then
    SECRET=$(openssl rand -hex 20)
    echo "Generated webhook secret: $SECRET"
    echo "Save this for GitHub webhook configuration!"
    echo ""
fi

echo "╔══════════════════════════════════════════════════╗"
echo "║  SerialRUN Webhook Setup (Cloudflare Tunnel)    ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Domain: $DOMAIN"
echo "Port: $PORT"
echo "Secret: $SECRET"
echo ""

# Create config directory
CONFIG_DIR="$HOME/.serialrun"
mkdir -p "$CONFIG_DIR"

# Save config
cat > "$CONFIG_DIR/webhook_config.json" << EOF
{
  "domain": "$DOMAIN",
  "port": $PORT,
  "secret": "$SECRET",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "Config saved to: $CONFIG_DIR/webhook_config.json"
echo ""

# Check if already logged in
if ! cloudflared tunnel list 2>/dev/null | grep -q "serialrun-webhook"; then
    echo "Creating tunnel..."
    cloudflared tunnel create serialrun-webhook
    echo ""
fi

# Get tunnel ID
TUNNEL_ID=$(cloudflared tunnel list | grep serialrun-webhook | awk '{print $1}')

if [[ -z "$TUNNEL_ID" ]]; then
    echo "❌ Failed to create tunnel"
    exit 1
fi

echo "Tunnel ID: $TUNNEL_ID"
echo ""

# Create DNS route
echo "Setting up DNS route..."
cloudflared tunnel route dns serialrun-webhook "$DOMAIN" 2>/dev/null || true
echo ""

# Create config file
cat > "$CONFIG_DIR/config.yml" << EOF
tunnel: $TUNNEL_ID
credentials-file: $HOME/.cloudflared/$TUNNEL_ID.json

ingress:
  - hostname: $DOMAIN
    service: http://localhost:$PORT
  - service: http_status:404
EOF

echo "Config written to: $CONFIG_DIR/config.yml"
echo ""

# Start tunnel script
cat > "$CONFIG_DIR/start_webhook.sh" << 'SCRIPT'
#!/bin/bash
# Start webhook tunnel + server

CONFIG_DIR="$HOME/.serialrun"
CONFIG="$CONFIG_DIR/config.yml"

if [[ ! -f "$CONFIG" ]]; then
    echo "❌ Config not found. Run setup_webhook.sh first."
    exit 1
fi

# Start webhook server in background
echo "Starting webhook server..."
python3 "$(dirname "$0")/webhook_server.py" \
    --port $(jq -r '.port' "$CONFIG_DIR/webhook_config.json") \
    --secret $(jq -r '.secret' "$CONFIG_DIR/webhook_config.json") &

WEBHOOK_PID=$!
echo "Webhook server PID: $WEBHOOK_PID"

# Start tunnel
echo "Starting Cloudflare Tunnel..."
cloudflared tunnel --config "$CONFIG" run

# Cleanup
kill $WEBHOOK_PID 2>/dev/null
SCRIPT

chmod +x "$CONFIG_DIR/start_webhook.sh"

echo "╔══════════════════════════════════════════════════╗"
echo "║  ✅ Setup Complete                               ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo ""
echo "1. Login to Cloudflare (if not already):"
echo "   cloudflared tunnel login"
echo ""
echo "2. Start the webhook server:"
echo "   $CONFIG_DIR/start_webhook.sh"
echo ""
echo "3. Configure GitHub Webhook:"
echo "   URL: https://$DOMAIN/webhook"
echo "   Secret: $SECRET"
echo "   Content type: application/json"
echo "   Events: Issues"
echo ""
echo "4. Start monitor:"
echo "   ./scripts/monitor_plugins.sh --watch"
echo ""
