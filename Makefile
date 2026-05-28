.PHONY: build release app install clean test run mcp lint fmt

# Build all crates (debug)
build:
	cargo build

# Build all crates (release) and sync .app bundle
release:
	@mkdir -p target/release/SerialTap.app/Contents/Resources
	@python3 scripts/gen_icon.py target/release/SerialTap.app/Contents/Resources/icon.icns
	cargo build --release
	@if [ -d target/release/SerialTap.app ]; then \
		echo "Syncing .app bundle..."; \
		cp target/release/serialtap target/release/SerialTap.app/Contents/MacOS/serialtap; \
		cp crates/serialtap-gui/Info.plist target/release/SerialTap.app/Contents/Info.plist; \
		codesign --force --deep --sign - target/release/SerialTap.app 2>/dev/null; \
		echo ".app bundle updated."; \
	fi

# Build macOS .app bundle with icon
app:
	@echo "Step 1: Generate icons from master image..."
	@mkdir -p target/release/SerialTap.app/Contents/Resources
	@python3 scripts/gen_icon.py target/release/SerialTap.app/Contents/Resources/icon.icns
	@echo "Step 2: Build binary (embeds icon)..."
	@cargo build --release -p serialtap-gui
	@echo "Step 3: Create .app bundle..."
	@cp target/release/serialtap target/release/SerialTap.app/Contents/MacOS/serialtap
	@cp crates/serialtap-gui/Info.plist target/release/SerialTap.app/Contents/Info.plist
	@cp -r docs target/release/SerialTap.app/Contents/Resources/docs
	@codesign --force --deep --sign - target/release/SerialTap.app 2>/dev/null
	@echo ""
	@echo "Done! App bundle: target/release/SerialTap.app"
	@echo "Run:      open target/release/SerialTap.app"
	@echo "Install:  make install"

# Install to /Applications
install: app
	@rm -rf /Applications/SerialTap.app
	@cp -r target/release/SerialTap.app /Applications/
	@codesign --force --deep --sign - /Applications/SerialTap.app 2>/dev/null
	@killall Dock 2>/dev/null || true
	@echo "Installed to /Applications/SerialTap.app (signed, Dock refreshed)"

# Run (debug)
run:
	cargo run -p serialtap-gui

# Run release .app
run-app: app
	open target/release/SerialTap.app

# Run MCP server
mcp:
	cargo run -p serialtap-mcp

# Run tests
test:
	cargo test --workspace

# Lint
lint:
	cargo clippy --workspace

# Format
fmt:
	cargo fmt --all

# Clean
clean:
	cargo clean
	rm -rf target/release/SerialTap.app
