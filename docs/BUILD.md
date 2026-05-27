# SerialTap Build Guide / SerialTap 构建指南

English | [中文](#中文构建指南)

---

## Prerequisites / 前提条件

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- Platform-specific build tools (see below)

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- 平台特定的构建工具（见下方）

---

## Windows

### Requirements / 要求

- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++"

- 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选「使用 C++ 的桌面开发」

### Build / 构建

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc --release -p serialtap-gui
```

Output / 输出: `target/x86_64-pc-windows-msvc/release/serialtap-gui.exe`

---

## macOS

### Requirements / 要求

```bash
xcode-select --install
```

### Build / 构建

```bash
# Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-darwin
cargo build --target aarch64-apple-darwin --release -p serialtap-gui

# Intel Mac
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin --release -p serialtap-gui
```

### Bundle as .app / 打包为 .app

```bash
cargo install cargo-bundle
```

Add to `crates/serialtap-gui/Cargo.toml` / 在 `crates/serialtap-gui/Cargo.toml` 中添加:

```toml
[package.metadata.bundle]
name = "SerialTap"
identifier = "com.serialtap.app"
category = "DeveloperTool"
short_description = "Serial port assistant for embedded developers"
```

Run / 运行:

```bash
cargo bundle --target aarch64-apple-darwin --release -p serialtap-gui
```

Output / 输出: `target/release/bundle/osx/SerialTap.app`

---

## Linux

### Requirements / 要求 (Ubuntu/Debian)

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libgtk-3-dev libatk1.0-dev
```

### Build / 构建

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release -p serialtap-gui
```

---

## Android

### Requirements / 要求

- Android NDK
- [cargo-mobile2](https://github.com/nickelc/cargo-mobile2)

### Build / 构建

```bash
cargo install cargo-mobile2
cargo android init
cargo android build --release -p serialtap-gui
```

---

## iOS

### Requirements / 要求

- Xcode
- Apple Developer Account
- [cargo-mobile2](https://github.com/nickelc/cargo-mobile2)

### Build / 构建

```bash
cargo install cargo-mobile2
cargo ios init
cargo ios build --release -p serialtap-gui
```

---

## Cross-Compilation Reference / 交叉编译参考

| Target / 目标 | Command / 命令 |
|---------------|----------------|
| Windows x64 | `--target x86_64-pc-windows-msvc` |
| macOS ARM | `--target aarch64-apple-darwin` |
| macOS x64 | `--target x86_64-apple-darwin` |
| Linux x64 | `--target x86_64-unknown-linux-gnu` |
| Linux ARM64 | `--target aarch64-unknown-linux-gnu` |
| Android | `--target aarch64-linux-android` |
| iOS | `--target aarch64-apple-ios` |

```bash
# Add target / 添加目标
rustup target add <target>

# Build / 构建
cargo build --target <target> --release -p serialtap-gui
```

---

## Output Paths / 输出路径

```
target/<target>/release/serialtap-gui.exe              # Windows
target/<target>/release/serialtap-gui                  # macOS/Linux
target/release/bundle/osx/SerialTap.app                # macOS .app
target/release/bundle/deb/serialtap-gui_*.deb          # Debian package
target/release/bundle/rpm/serialtap-gui-*.rpm          # RPM package
```

---

## 中文构建指南

### 前提条件

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- 平台特定的构建工具（见下方）

### Windows

安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选「使用 C++ 的桌面开发」。

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc --release -p serialtap-gui
```

输出: `target/x86_64-pc-windows-msvc/release/serialtap-gui.exe`

### macOS

```bash
xcode-select --install

# Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-darwin
cargo build --target aarch64-apple-darwin --release -p serialtap-gui

# Intel Mac
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin --release -p serialtap-gui
```

#### 打包为 .app

```bash
cargo install cargo-bundle
```

在 `crates/serialtap-gui/Cargo.toml` 中添加:

```toml
[package.metadata.bundle]
name = "SerialTap"
identifier = "com.serialtap.app"
category = "DeveloperTool"
short_description = "串口助手 - 嵌入式开发者工具"
```

```bash
cargo bundle --target aarch64-apple-darwin --release -p serialtap-gui
```

输出: `target/release/bundle/osx/SerialTap.app`

### Linux

```bash
# Ubuntu/Debian 依赖
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libgtk-3-dev libatk1.0-dev

rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release -p serialtap-gui
```

### Android

```bash
cargo install cargo-mobile2
cargo android init
cargo android build --release -p serialtap-gui
```

### iOS

```bash
cargo install cargo-mobile2
cargo ios init
cargo ios build --release -p serialtap-gui
```

### 交叉编译参考

| 目标 | 命令 |
|------|------|
| Windows x64 | `--target x86_64-pc-windows-msvc` |
| macOS ARM | `--target aarch64-apple-darwin` |
| macOS x64 | `--target x86_64-apple-darwin` |
| Linux x64 | `--target x86_64-unknown-linux-gnu` |
| Linux ARM64 | `--target aarch64-unknown-linux-gnu` |
| Android | `--target aarch64-linux-android` |
| iOS | `--target aarch64-apple-ios` |
