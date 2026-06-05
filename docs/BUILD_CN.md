# SerialRUN 构建指南

[English](BUILD.md) | [构建检查清单](BUILD_CHECKLIST.md)

---

## 前提条件

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- 平台特定的构建工具（见下方）

## 构建理念：平台分离输出

每个平台构建到各自的 `target/<triple>/release/` 目录，Windows 和 macOS 的产物完全分离：

```
target/x86_64-pc-windows-msvc/release/    # Windows
target/aarch64-apple-darwin/release/       # macOS ARM
target/x86_64-apple-darwin/release/        # macOS Intel
```

## Windows

### 要求

安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选「使用 C++ 的桌面开发」。

### 构建

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui
```

输出: `target/x86_64-pc-windows-msvc/release/serialrun.exe`

### 发布打包

```bash
# 自动化（通过发布脚本）
./scripts/release.sh v0.3.0 --dry-run     # 预览
./scripts/release.sh v0.3.0               # 构建 + 发布到 GitHub & Gitea

# 手动
mkdir -p /tmp/serialrun-win
cp target/x86_64-pc-windows-msvc/release/serialrun.exe /tmp/serialrun-win/
cp docs/help_en.md docs/help_zh.md /tmp/serialrun-win/
cd /tmp/serialrun-win && zip -r serialrun-0.3.0-windows-x64.zip .
```

ZIP 内容: `serialrun.exe`, `help_en.md`, `help_zh.md`

## macOS

### 要求

```bash
xcode-select --install
```

### 构建

```bash
# Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-darwin
cargo build --target aarch64-apple-darwin --release -p serialrun-gui

# Intel Mac
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin --release -p serialrun-gui
```

输出: `target/<target>/release/serialrun`

### 打包为 .app

```bash
cargo install cargo-bundle
```

在 `crates/serialrun-gui/Cargo.toml` 中添加:

```toml
[package.metadata.bundle]
name = "SerialRUN"
identifier = "com.serialrun.app"
category = "DeveloperTool"
short_description = "串口助手 - 嵌入式开发者工具"
```

运行:

```bash
cargo bundle --target aarch64-apple-darwin --release -p serialrun-gui
```

输出: `target/release/bundle/osx/SerialRUN.app`

### 发布打包（通过 Makefile）

```bash
make release    # 构建 + 签名 .app
make install    # 复制到 /Applications
```

## Linux

### 要求 (Ubuntu/Debian)

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libgtk-3-dev libatk1.0-dev
```

### 构建

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

## 交叉编译参考

| 目标 | 命令 | 输出 |
|------|------|------|
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

## 输出路径

```
target/x86_64-pc-windows-msvc/release/serialrun.exe     # Windows
target/aarch64-apple-darwin/release/serialrun            # macOS ARM
target/x86_64-apple-darwin/release/serialrun             # macOS Intel
target/x86_64-unknown-linux-gnu/release/serialrun        # Linux
target/release/bundle/osx/SerialRUN.app                  # macOS .app 包
```

## 快速参考

| 任务 | 命令 |
|------|------|
| 调试构建 | `cargo build -p serialrun-gui` |
| 发布构建（本机） | `cargo build --release -p serialrun-gui` |
| 发布构建（Windows） | `cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui` |
| 运行 | `cargo run -p serialrun-gui` |
| 测试 | `cargo test --workspace` |
| 代码检查 | `cargo clippy --workspace` |
| 格式化 | `cargo fmt --all` |
