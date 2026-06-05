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
# Automated (via release script)
./scripts/release.sh v0.3.0 --dry-run     # Preview
./scripts/release.sh v0.3.0               # Build + publish to GitHub & Gitea

# Manual
mkdir -p /tmp/serialrun-win
cp target/x86_64-pc-windows-msvc/release/serialrun.exe /tmp/serialrun-win/
cp docs/help_en.md docs/help_zh.md /tmp/serialrun-win/
cd /tmp/serialrun-win && zip -r serialrun-0.3.0-windows-x64.zip .
```

ZIP contents: `serialrun.exe`, `help_en.md`, `help_zh.md`

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
cargo install cargo-bundle
```

Add to `crates/serialrun-gui/Cargo.toml`:

```toml
[package.metadata.bundle]
name = "SerialRUN"
identifier = "com.serialrun.app"
category = "DeveloperTool"
short_description = "Serial port assistant for embedded developers"
```

Run:

```bash
cargo bundle --target aarch64-apple-darwin --release -p serialrun-gui
```

Output: `target/release/bundle/osx/SerialRUN.app`

### Release Package (via Makefile)

```bash
make release    # Build + codesign .app
make install    # Copy to /Applications
```

## Linux

### Requirements (Ubuntu/Debian)

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libgtk-3-dev libatk1.0-dev
```

### Build

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release -p serialrun-gui
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
