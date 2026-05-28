.PHONY: build release app install clean test run mcp lint fmt

# Build all crates (debug)
build:
	cargo build

# Build all crates (release) and sync .app bundle
release:
	@mkdir -p target/release/SerialRUN.app/Contents/Resources
	@python3 scripts/gen_icon.py target/release/SerialRUN.app/Contents/Resources/icon.icns
	cargo build --release
	@if [ -d target/release/SerialRUN.app ]; then \
		echo "Syncing .app bundle..."; \
		cp target/release/serialrun target/release/SerialRUN.app/Contents/MacOS/serialrun; \
		cp crates/serialrun-gui/Info.plist target/release/SerialRUN.app/Contents/Info.plist; \
		codesign --force --deep --sign - target/release/SerialRUN.app 2>/dev/null; \
		echo ".app bundle updated."; \
	fi

# Build macOS .app bundle with icon
app:
	@echo "Step 1: Generate icons from master image..."
	@mkdir -p target/release/SerialRUN.app/Contents/Resources
	@python3 scripts/gen_icon.py target/release/SerialRUN.app/Contents/Resources/icon.icns
	@echo "Step 2: Build binary (embeds icon)..."
	@cargo build --release -p serialrun-gui
	@echo "Step 3: Create .app bundle..."
	@cp target/release/serialrun target/release/SerialRUN.app/Contents/MacOS/serialrun
	@cp crates/serialrun-gui/Info.plist target/release/SerialRUN.app/Contents/Info.plist
	@cp -r docs target/release/SerialRUN.app/Contents/Resources/docs
	@codesign --force --deep --sign - target/release/SerialRUN.app 2>/dev/null
	@echo ""
	@echo "Done! App bundle: target/release/SerialRUN.app"
	@echo "Run:      open target/release/SerialRUN.app"
	@echo "Install:  make install"

# Install to /Applications
install: app
	@rm -rf /Applications/SerialRUN.app
	@cp -r target/release/SerialRUN.app /Applications/
	@codesign --force --deep --sign - /Applications/SerialRUN.app 2>/dev/null
	@killall Dock 2>/dev/null || true
	@echo "Installed to /Applications/SerialRUN.app (signed, Dock refreshed)"

# Run (debug)
run:
	cargo run -p serialrun-gui

# Run release .app
run-app: app
	open target/release/SerialRUN.app

# Run MCP server
mcp:
	cargo run -p serialrun-mcp

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
	rm -rf target/release/SerialRUN.app
