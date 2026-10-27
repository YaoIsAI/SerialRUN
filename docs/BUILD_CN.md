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

## GitHub Actions 发布（推荐）

发布流水线在云端构建全部平台：

1. 打开 **Actions → Release → Run workflow**。
2. 输入版本号（如 `0.4.5`）并运行。
3. 流水线会自动递增版本号、提交并打标签 `vX.Y.Z`，构建 Windows x64 /
   macOS arm64 / macOS x64 / Linux x64，并上传安装包到 GitHub Releases。

完整检查清单见 [RELEASE_SOP.md](RELEASE_SOP.md)。

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
# 推荐：使用 GitHub Actions 自动递增版本并发布全部平台
# 详见 docs/RELEASE_SOP.md

# 本地打包（先执行 cargo build --release -p serialrun-gui）
pwsh ./scripts/package-windows.ps1 -Version 0.4.5
```

ZIP 内容：`serialrun.exe`、安装说明、帮助和手册文档。

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
# 生成 SerialRUN.app，ad-hoc 签名，并同时输出 ZIP 和 DMG
bash scripts/package-macos.sh aarch64-apple-darwin 0.4.5    # Apple Silicon
bash scripts/package-macos.sh x86_64-apple-darwin 0.4.5     # Intel
```

输出：`dist/serialrun-0.4.5-macos-arm64.dmg` / `.zip`

应用包使用 `crates/serialrun-gui/Info.plist` 和 `icon.icns`。设置
`CODESIGN_IDENTITY` 可使用 Developer ID 正式签名；未设置时使用 ad-hoc 签名。
首次打开说明会写入包内 `INSTALL.txt`。

### 发布打包（通过 Makefile）

```bash
make release    # 构建 + 签名 .app
make install    # 复制到 /Applications
```

## Linux

### 要求 (Ubuntu/Debian)

```bash
sudo apt install libudev-dev pkg-config libx11-dev libxcursor-dev libxrandr-dev \
  libxi-dev libgl1-mesa-dev libxkbcommon-dev libwayland-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### 构建

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release -p serialrun-gui

# 打包为 tar.gz
bash scripts/package-linux.sh 0.4.5
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
