# SerialRUN Build Guide

[中文版](BUILD_CN.md) | [构建检查清单](BUILD_CHECKLIST_CN.md)

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- Platform-specific build tools (see below)

## Build Philosophy: Platform-Separated Outputs

Each platform builds to its own `target/<triple>/release/` directory, keeping Windows and macOS artifacts completely separate:

```
target/x86_64-pc-windows-msvc/release/    # Windows
target/aarch64-apple-darwin/release/       # macOS ARM
target/x86_64-apple-darwin/release/        # macOS Intel
```

## GitHub Actions Release (Recommended)

The release workflow builds every platform in the cloud:

1. Open **Actions → Release → Run workflow**.
2. Enter the version (e.g. `0.4.5`) and run it.
3. The workflow bumps all version numbers, commits, tags `vX.Y.Z`, builds
   Windows x64 / macOS arm64 / macOS x64 / Linux x64, and publishes the
   archives to GitHub Releases.

See [RELEASE_SOP.md](RELEASE_SOP.md) for the full checklist.

## Windows

### Requirements

- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++"

### Build

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui
```

Output: `target/x86_64-pc-windows-msvc/release/serialrun.exe`

### Release Package

```bash
# Automated (recommended): bumps versions, builds and publishes all platforms
# See docs/RELEASE_SOP.md

# Local packaging (after cargo build --release -p serialrun-gui)
pwsh ./scripts/package-windows.ps1 -Version 0.4.4
```

ZIP contents: `serialrun.exe`, install notes, help and manual documents.

## macOS

### Requirements

```bash
xcode-select --install
```

### Build

```bash
# Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-darwin
cargo build --target aarch64-apple-darwin --release -p serialrun-gui

# Intel Mac
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin --release -p serialrun-gui
```

Output: `target/<target>/release/serialrun`

### Bundle as .app

```bash
# Builds SerialRUN.app, ad-hoc signs it, and creates both ZIP and DMG
bash scripts/package-macos.sh aarch64-apple-darwin 0.4.4    # Apple Silicon
bash scripts/package-macos.sh x86_64-apple-darwin 0.4.4     # Intel
```

Output: `dist/serialrun-0.4.4-macos-arm64.dmg` / `.zip`

The bundle uses `crates/serialrun-gui/Info.plist` and `icon.icns`. Set
`CODESIGN_IDENTITY` to a Developer ID to sign with your certificate instead of
ad-hoc signing. Gatekeeper first-launch instructions are included in `INSTALL.txt`.

### Release Package (via Makefile)

```bash
make release    # Build + codesign .app
make install    # Copy to /Applications
```

## Linux

### Requirements (Ubuntu/Debian)

```bash
sudo apt install libudev-dev pkg-config libx11-dev libxcursor-dev libxrandr-dev \
  libxi-dev libgl1-mesa-dev libxkbcommon-dev libwayland-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Build

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release -p serialrun-gui

# Package as tar.gz
bash scripts/package-linux.sh 0.4.4
```

## Android

```bash
cargo install cargo-mobile2
cargo android init
cargo android build --release -p serialrun-gui
```

## iOS

```bash
cargo install cargo-mobile2
cargo ios init
cargo ios build --release -p serialrun-gui
```

## Cross-Compilation Reference

| Target | Command | Output |
|--------|---------|--------|
| Windows x64 | `--target x86_64-pc-windows-msvc` | `serialrun.exe` |
| macOS ARM | `--target aarch64-apple-darwin` | `serialrun` |
| macOS x64 | `--target x86_64-apple-darwin` | `serialrun` |
| Linux x64 | `--target x86_64-unknown-linux-gnu` | `serialrun` |
| Linux ARM64 | `--target aarch64-unknown-linux-gnu` | `serialrun` |
| Android | `--target aarch64-linux-android` | `serialrun` |
| iOS | `--target aarch64-apple-ios` | `serialrun` |

```bash
rustup target add <target>
cargo build --target <target> --release -p serialrun-gui
```

## Output Paths

```
target/x86_64-pc-windows-msvc/release/serialrun.exe     # Windows
target/aarch64-apple-darwin/release/serialrun            # macOS ARM
target/x86_64-apple-darwin/release/serialrun             # macOS Intel
target/x86_64-unknown-linux-gnu/release/serialrun        # Linux
target/release/bundle/osx/SerialRUN.app                  # macOS .app bundle
```

## Quick Reference

| Task | Command |
|------|---------|
| Debug build | `cargo build -p serialrun-gui` |
| Release build (host) | `cargo build --release -p serialrun-gui` |
| Release build (Windows) | `cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui` |
| Run | `cargo run -p serialrun-gui` |
| Test | `cargo test --workspace` |
| Lint | `cargo clippy --workspace` |
| Format | `cargo fmt --all` |
