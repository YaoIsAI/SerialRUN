#!/bin/bash
# Auto-develop plugin from GitHub Issue request
#
# Usage:
#   ./scripts/auto_develop_plugin.sh issue-123.json
#
# Reads the pending request and generates a plugin.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
PENDING_DIR="$HOME/.serialrun/pending_plugins"
PLUGINS_DIR="$REPO_ROOT/plugins"

REQUEST_FILE="${1:-}"

if [[ -z "$REQUEST_FILE" ]]; then
    echo "Usage: $0 <issue-XXX.json>"
    echo ""
    echo "Available pending requests:"
    ls -1 "$PENDING_DIR"/issue-*.json 2>/dev/null || echo "  (none)"
    exit 1
fi

# Read request
if [[ ! -f "$REQUEST_FILE" ]]; then
    echo "Error: Request file not found: $REQUEST_FILE"
    exit 1
fi

PLUGIN_NAME=$(jq -r '.plugin_name' "$REQUEST_FILE")
DESCRIPTION=$(jq -r '.description' "$REQUEST_FILE")
FEATURES=$(jq -r '.features[]' "$REQUEST_FILE")
ISSUE_URL=$(jq -r '.issue_url' "$REQUEST_FILE")
ISSUE_NUMBER=$(jq -r '.issue_number' "$REQUEST_FILE")

echo "╔══════════════════════════════════════════════════╗"
echo "║  SerialRUN Plugin Auto-Development               ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Issue: #$ISSUE_NUMBER"
echo "Plugin: $PLUGIN_NAME"
echo "Description: $DESCRIPTION"
echo "Features:"
echo "$FEATURES" | sed 's/^/  - /'
echo ""

# Step 1: Copy template
PLUGIN_DIR="$PLUGINS_DIR/$PLUGIN_NAME"
if [[ -d "$PLUGIN_DIR" ]]; then
    echo "⚠️  Plugin directory already exists: $PLUGIN_DIR"
    echo "   Skipping copy, will update existing plugin."
else
    echo "📁 Creating plugin from template..."
    cp -r "$PLUGINS_DIR/serialrun-example-plugin" "$PLUGIN_DIR"
fi

# Step 2: Update plugin.json
echo "📝 Updating plugin.json..."
jq --arg name "$PLUGIN_NAME" \
   --arg desc "$DESCRIPTION" \
   '.name = $name | .description = $desc | .version = "1.0.0"' \
   "$PLUGIN_DIR/plugin.json" > "$PLUGIN_DIR/plugin.json.tmp"
mv "$PLUGIN_DIR/plugin.json.tmp" "$PLUGIN_DIR/plugin.json"

# Step 3: Update Cargo.toml
echo "📝 Updating Cargo.toml..."
sed -i '' "s/name = \"serialrun-example-plugin\"/name = \"$PLUGIN_NAME\"/" \
    "$PLUGIN_DIR/Cargo.toml" 2>/dev/null || \
sed -i "s/name = \"serialrun-example-plugin\"/name = \"$PLUGIN_NAME\"/" \
    "$PLUGIN_DIR/Cargo.toml"

# Add to workspace members if not already present
WORKSPACE_CARGO="$REPO_ROOT/Cargo.toml"
if ! grep -q "\"plugins/$PLUGIN_NAME\"" "$WORKSPACE_CARGO"; then
    echo "📝 Adding to workspace members..."
    sed -i '' "/members =/,/]/{
        s|]|\    \"plugins/$PLUGIN_NAME\",\n]|;
    }" "$WORKSPACE_CARGO" 2>/dev/null || \
    sed -i "/members =/,/]/{
        s|]|\    \"plugins/$PLUGIN_NAME\",\n]|;
    }" "$WORKSPACE_CARGO"
fi

# Step 4: Generate plugin commands from features
echo "📝 Generating plugin commands..."
COMMANDS=""
IFS=$'\n' read -rd '' -a FEATURE_ARRAY <<< "$FEATURES"

for i in "${!FEATURE_ARRAY[@]}"; do
    feature="${FEATURE_ARRAY[$i]}"
    cmd_name=$(echo "$feature" | tr '[:upper:]' '[:lower:]' | tr ' ' '_' | tr -cd 'a-z0-9_')
    COMMANDS="$COMMANDS
        PluginCommand {
            name: \"$cmd_name\".to_string(),
            description: \"$feature\".to_string(),
            parameters: vec![],
        },"
done

# Step 5: Build and test
echo ""
echo "🔨 Building plugin..."
cd "$REPO_ROOT"
if cargo build --release -p "$PLUGIN_NAME" 2>&1; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    # Update request status
    jq '.status = "build_failed"' "$REQUEST_FILE" > "$REQUEST_FILE.tmp"
    mv "$REQUEST_FILE.tmp" "$REQUEST_FILE"
    exit 1
fi

echo ""
echo "🧪 Running tests..."
if cargo test -p "$PLUGIN_NAME" 2>&1; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed"
    jq '.status = "test_failed"' "$REQUEST_FILE" > "$REQUEST_FILE.tmp"
    mv "$REQUEST_FILE.tmp" "$REQUEST_FILE"
    exit 1
fi

# Step 6: Package
echo ""
echo "📦 Packaging plugin..."
cd "$REPO_ROOT"
"$REPO_ROOT/scripts/package_plugin.sh" "$PLUGIN_DIR" 2>&1 || true

# Update request status
jq '.status = "completed"' "$REQUEST_FILE" > "$REQUEST_FILE.tmp"
mv "$REQUEST_FILE.tmp" "$REQUEST_FILE"

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║  ✅ Plugin Development Complete                   ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Plugin: $PLUGIN_NAME"
echo "Directory: $PLUGIN_DIR"
echo "Issue: $ISSUE_URL"
echo ""
echo "Next steps:"
echo "  1. Review the generated code in $PLUGIN_DIR"
echo "  2. Add specific logic to plugin_execute()"
echo "  3. Push to serialrun-plugins repo"
echo "  4. Comment on Issue #$ISSUE_NUMBER with install instructions"
