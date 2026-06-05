# SerialRUN Build Checklist

[中文版](BUILD_CHECKLIST_CN.md)

Before building, run through this checklist to avoid common mistakes.

---

## 1. Version Number Consistency

All version numbers must match. Run:

```bash
# Check all Cargo.toml versions
grep -r 'version = "' crates/*/Cargo.toml plugins/*/Cargo.toml

# Check status bar version
grep 'SerialRUN v' crates/serialrun-gui/src/ui/status.rs
```

Expected: All should show the same version (e.g. `0.3.0`).

| File | Expected |
|------|----------|
| `crates/serialrun-gui/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-core/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-plugin-api/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-cli/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-mcp/Cargo.toml` | `version = "X.Y.Z"` |
| `plugins/serialrun-example-plugin/Cargo.toml` | `version = "X.Y.Z"` |
| `plugins/serialrun-mpy-ide/Cargo.toml` | `version = "X.Y.Z"` |
| `plugins/serialrun-stc-isp/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-gui/src/ui/status.rs` | `SerialRUN vX.Y.Z` |

If any mismatch → **fix before building**.

## 2. Build Directory (Platform-Separated)

Each platform builds to its own directory. **Do NOT use `target/release/` directly**.

| Platform | Build Command | Output Path |
|----------|---------------|-------------|
| **Windows** | `cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui` | `target/x86_64-pc-windows-msvc/release/serialrun.exe` |
| **macOS ARM** | `cargo build --target aarch64-apple-darwin --release -p serialrun-gui` | `target/aarch64-apple-darwin/release/serialrun` |
| **macOS Intel** | `cargo build --target x86_64-apple-darwin --release -p serialrun-gui` | `target/x86_64-apple-darwin/release/serialrun` |
| **Linux** | `cargo build --target x86_64-unknown-linux-gnu --release -p serialrun-gui` | `target/x86_64-unknown-linux-gnu/release/serialrun` |

**Windows shortcut**: `make win`

**macOS shortcut**: `make app` (builds + creates .app bundle)

### Why platform separation?

- Avoids mixing Windows `.exe` and macOS binaries in the same directory
- Allows building for multiple platforms from the same machine (cross-compilation)
- Makes CI/CD cleaner — each platform's artifacts are self-contained

## 3. Pre-Build Checks

```bash
# 1. Close running instance (Windows)
taskkill //F //IM serialrun.exe 2>/dev/null || true

# 2. Close running instance (macOS)
pkill -f serialrun 2>/dev/null || true

# 3. Verify Rust toolchain
rustc --version    # Should be 1.70+
cargo --version

# 4. Verify target is installed
rustup target list --installed | grep x86_64-pc-windows-msvc   # Windows
rustup target list --installed | grep aarch64-apple-darwin      # macOS ARM

# 5. Clean previous build (optional, ensures fresh build)
cargo clean
```

## 4. Build

### Windows

```bash
cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui
```

Verify output exists:
```bash
ls -lh target/x86_64-pc-windows-msvc/release/serialrun.exe
```

### macOS

```bash
# Option A: Direct build
cargo build --target aarch64-apple-darwin --release -p serialrun-gui

# Option B: Full .app bundle (recommended)
make app
```

Verify output exists:
```bash
ls -lh target/aarch64-apple-darwin/release/serialrun           # Option A
ls -lh target/release/SerialRUN.app/Contents/MacOS/serialrun   # Option B
```

## 5. Package Contents

### Windows ZIP

```
serialrun-X.Y.Z-windows-x64.zip
  +-- serialrun.exe          (main binary)
  +-- help_en.md             (English help)
  +-- help_zh.md             (Chinese help)
```

Build command:
```bash
make win-package
# Or manual:
mkdir -p /tmp/serialrun-pkg
cp target/x86_64-pc-windows-msvc/release/serialrun.exe /tmp/serialrun-pkg/
cp docs/help_en.md docs/help_zh.md /tmp/serialrun-pkg/
cd /tmp/serialrun-pkg && zip -r serialrun-X.Y.Z-windows-x64.zip .
```

### macOS .app Bundle

```
SerialRUN.app/
  +-- Contents/
      +-- MacOS/serialrun       (binary)
      +-- Info.plist             (metadata)
      +-- Resources/
          +-- icon.icns          (app icon)
          +-- docs/              (all documentation)
```

Build command:
```bash
make app
```

## 6. Post-Build Verification

```bash
# Check binary runs
target/x86_64-pc-windows-msvc/release/serialrun.exe --version    # Windows
open target/release/SerialRUN.app                                 # macOS

# Check version in UI matches expected version
# (look at bottom-right corner of status bar)

# Check help files are loadable
# (app should find help_en.md / help_zh.md next to the binary)
```

## 7. Release Publish

```bash
# Dry run (preview what will happen)
./scripts/release.sh v0.3.0 --dry-run

# Full release (GitHub + Gitea)
./scripts/release.sh v0.3.0

# GitHub only
./scripts/release.sh v0.3.0 --github

# Gitea only
./scripts/release.sh v0.3.0 --gitea
```

Required environment variables:
- `GITHUB_TOKEN` — GitHub personal access token
- `GITEA_TOKEN` — Gitea personal access token

## 8. Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Version mismatch | Status bar shows wrong version | Update all Cargo.toml + status.rs |
| Wrong build directory | `target/release/serialrun.exe` has old binary | Use `--target` flag |
| Missing help files | App can't load help docs | Include help_en.md + help_zh.md in package |
| Running instance | Build fails or overwrites locked file | Kill serialrun.exe first |
| Wrong target | Build fails with linker errors | `rustup target add <target>` |
| Cargo.lock conflict | Build errors after pulling | `cargo update` or delete Cargo.lock |
