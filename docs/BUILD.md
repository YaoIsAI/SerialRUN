# SerialRUN Build Guide

[中文版](BUILD_CN.md)

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- Platform-specific build tools (see below)

## Windows

### Requirements

- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++"

### Build

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui
```

Output: `target/x86_64-pc-windows-msvc/release/serialrun-gui.exe`

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

| Target | Command |
|--------|---------|
| Windows x64 | `--target x86_64-pc-windows-msvc` |
| macOS ARM | `--target aarch64-apple-darwin` |
| macOS x64 | `--target x86_64-apple-darwin` |
| Linux x64 | `--target x86_64-unknown-linux-gnu` |
| Linux ARM64 | `--target aarch64-unknown-linux-gnu` |
| Android | `--target aarch64-linux-android` |
| iOS | `--target aarch64-apple-ios` |

```bash
rustup target add <target>
cargo build --target <target> --release -p serialrun-gui
```

## Output Paths

```
target/<target>/release/serialrun-gui.exe              # Windows
target/<target>/release/serialrun-gui                  # macOS/Linux
target/release/bundle/osx/SerialRUN.app                # macOS .app
target/release/bundle/deb/serialrun-gui_*.deb          # Debian package
target/release/bundle/rpm/serialrun-gui-*.rpm          # RPM package
```
