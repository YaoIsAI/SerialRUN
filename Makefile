.PHONY: build release app install clean test run mcp lint fmt win win-release win-package

WIN_TARGET = x86_64-pc-windows-msvc
WIN_BUILD  = target/$(WIN_TARGET)/release
MAC_BUILD  = target/release

# ── Common ──

# Build all crates (debug)
build:
	cargo build

# Run (debug)
run:
	cargo run -p serialrun-gui

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

# ── macOS ──

# Build release + sync .app bundle
release:
	@mkdir -p $(MAC_BUILD)/SerialRUN.app/Contents/Resources
	@python3 scripts/gen_icon.py $(MAC_BUILD)/SerialRUN.app/Contents/Resources/icon.icns
	cargo build --release -p serialrun-gui
	@if [ -d $(MAC_BUILD)/SerialRUN.app ]; then \
		echo "Syncing .app bundle..."; \
		cp $(MAC_BUILD)/serialrun $(MAC_BUILD)/SerialRUN.app/Contents/MacOS/serialrun; \
		cp crates/serialrun-gui/Info.plist $(MAC_BUILD)/SerialRUN.app/Contents/Info.plist; \
		codesign --force --deep --sign - $(MAC_BUILD)/SerialRUN.app 2>/dev/null; \
		echo ".app bundle updated."; \
	fi

# Build macOS .app bundle with icon
app:
	@echo "Step 1: Generate icons from master image..."
	@mkdir -p $(MAC_BUILD)/SerialRUN.app/Contents/MacOS
	@mkdir -p $(MAC_BUILD)/SerialRUN.app/Contents/Resources
	@python3 scripts/gen_icon.py $(MAC_BUILD)/SerialRUN.app/Contents/Resources/icon.icns
	@echo "Step 2: Build binary (embeds icon)..."
	@cargo build --release -p serialrun-gui
	@echo "Step 3: Create .app bundle..."
	@cp $(MAC_BUILD)/serialrun $(MAC_BUILD)/SerialRUN.app/Contents/MacOS/serialrun
	@cp crates/serialrun-gui/Info.plist $(MAC_BUILD)/SerialRUN.app/Contents/Info.plist
	@rm -rf $(MAC_BUILD)/SerialRUN.app/Contents/Resources/docs
	@cp -r docs $(MAC_BUILD)/SerialRUN.app/Contents/Resources/docs
	@codesign --force --deep --sign - $(MAC_BUILD)/SerialRUN.app 2>/dev/null
	@echo ""
	@echo "Done! App bundle: $(MAC_BUILD)/SerialRUN.app"
	@echo "Run:      open $(MAC_BUILD)/SerialRUN.app"
	@echo "Install:  make install"

# Install to /Applications
install: app
	@rm -rf /Applications/SerialRUN.app
	@cp -r $(MAC_BUILD)/SerialRUN.app /Applications/
	@codesign --force --deep --sign - /Applications/SerialRUN.app 2>/dev/null
	@killall Dock 2>/dev/null || true
	@echo "Installed to /Applications/SerialRUN.app (signed, Dock refreshed)"

# Run release .app
run-app: app
	open $(MAC_BUILD)/SerialRUN.app

# ── Windows ──

# Build Windows release (uses --target for platform separation)
win:
	@taskkill //F //IM serialrun.exe 2>/dev/null || true
	cargo build --target $(WIN_TARGET) --release -p serialrun-gui
	@echo "Build complete: $(WIN_BUILD)/serialrun.exe"

# Build + package Windows release ZIP
win-package: win
	@echo "Packaging Windows release..."
	@rm -rf /tmp/serialrun-win-pkg
	@mkdir -p /tmp/serialrun-win-pkg
	@cp $(WIN_BUILD)/serialrun.exe /tmp/serialrun-win-pkg/
	@cp docs/help_en.md docs/help_zh.md /tmp/serialrun-win-pkg/
	@cd /tmp/serialrun-win-pkg && zip -r $(CURDIR)/serialrun-windows-x64.zip .
	@rm -rf /tmp/serialrun-win-pkg
	@echo "Package created: serialrun-windows-x64.zip"

# ── Clean ──

clean:
	cargo clean
	rm -rf $(MAC_BUILD)/SerialRUN.app
