# SerialRUN 构建检查清单

[English](BUILD_CHECKLIST.md)

构建前按此清单逐项检查，避免常见错误。

---

## 1. 版本号一致性

所有版本号必须一致。运行以下命令检查：

```bash
# 检查所有 Cargo.toml 版本
grep -r 'version = "' crates/*/Cargo.toml plugins/*/Cargo.toml

# 检查状态栏版本号
grep 'SerialRUN v' crates/serialrun-gui/src/ui/status.rs
```

期望结果：所有版本号一致（例如 `0.3.0`）。

| 文件 | 期望值 |
|------|--------|
| `crates/serialrun-gui/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-core/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-plugin-api/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-cli/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-mcp/Cargo.toml` | `version = "X.Y.Z"` |
| `plugins/serialrun-example-plugin/Cargo.toml` | `version = "X.Y.Z"` |
| `plugins/serialrun-mpy-ide/Cargo.toml` | `version = "X.Y.Z"` |
| `plugins/serialrun-stc-isp/Cargo.toml` | `version = "X.Y.Z"` |
| `crates/serialrun-gui/src/ui/status.rs` | `SerialRUN vX.Y.Z` |

如有不一致 → **先修复再构建**。

## 2. 构建目录（平台分离）

每个平台构建到各自目录。**不要直接使用 `target/release/`**。

| 平台 | 构建命令 | 输出路径 |
|------|----------|----------|
| **Windows** | `cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui` | `target/x86_64-pc-windows-msvc/release/serialrun.exe` |
| **macOS ARM** | `cargo build --target aarch64-apple-darwin --release -p serialrun-gui` | `target/aarch64-apple-darwin/release/serialrun` |
| **macOS Intel** | `cargo build --target x86_64-apple-darwin --release -p serialrun-gui` | `target/x86_64-apple-darwin/release/serialrun` |
| **Linux** | `cargo build --target x86_64-unknown-linux-gnu --release -p serialrun-gui` | `target/x86_64-unknown-linux-gnu/release/serialrun` |

**Windows 快捷方式**: `make win`

**macOS 快捷方式**: `make app`（构建 + 创建 .app 包）

### 为什么要平台分离？

- 避免 Windows `.exe` 和 macOS 二进制文件混在同一目录
- 支持从同一台机器交叉编译多个平台
- CI/CD 更清晰 — 每个平台的产物独立存放

## 3. 构建前检查

```bash
# 1. 关闭正在运行的实例 (Windows)
taskkill //F //IM serialrun.exe 2>/dev/null || true

# 2. 关闭正在运行的实例 (macOS)
pkill -f serialrun 2>/dev/null || true

# 3. 检查 Rust 工具链
rustc --version    # 应 >= 1.70+
cargo --version

# 4. 检查目标是否已安装
rustup target list --installed | grep x86_64-pc-windows-msvc   # Windows
rustup target list --installed | grep aarch64-apple-darwin      # macOS ARM

# 5. 清理旧构建（可选，确保干净构建）
cargo clean
```

## 4. 构建

### Windows

```bash
cargo build --target x86_64-pc-windows-msvc --release -p serialrun-gui
```

验证输出：
```bash
ls -lh target/x86_64-pc-windows-msvc/release/serialrun.exe
```

### macOS

```bash
# 方式 A：直接构建
cargo build --target aarch64-apple-darwin --release -p serialrun-gui

# 方式 B：完整 .app 包（推荐）
make app
```

验证输出：
```bash
ls -lh target/aarch64-apple-darwin/release/serialrun           # 方式 A
ls -lh target/release/SerialRUN.app/Contents/MacOS/serialrun   # 方式 B
```

## 5. 打包内容

### Windows ZIP

```
serialrun-X.Y.Z-windows-x64.zip
  +-- serialrun.exe          (主程序)
  +-- help_en.md             (英文帮助)
  +-- help_zh.md             (中文帮助)
```

构建命令：
```bash
make win-package
# 或手动：
mkdir -p /tmp/serialrun-pkg
cp target/x86_64-pc-windows-msvc/release/serialrun.exe /tmp/serialrun-pkg/
cp docs/help_en.md docs/help_zh.md /tmp/serialrun-pkg/
cd /tmp/serialrun-pkg && zip -r serialrun-X.Y.Z-windows-x64.zip .
```

### macOS .app 包

```
SerialRUN.app/
  +-- Contents/
      +-- MacOS/serialrun       (二进制文件)
      +-- Info.plist             (元数据)
      +-- Resources/
          +-- icon.icns          (应用图标)
          +-- docs/              (所有文档)
```

构建命令：
```bash
make app
```

## 6. 构建后验证

```bash
# 检查二进制文件能否运行
target/x86_64-pc-windows-msvc/release/serialrun.exe --version    # Windows
open target/release/SerialRUN.app                                 # macOS

# 检查 UI 中的版本号是否正确
# （查看状态栏右下角）

# 检查帮助文件是否可加载
# （应用应能找到二进制文件旁边的 help_en.md / help_zh.md）
```

## 7. 发布

```bash
# 预览（不实际发布）
./scripts/release.sh v0.3.0 --dry-run

# 完整发布（GitHub + Gitea）
./scripts/release.sh v0.3.0

# 仅 GitHub
./scripts/release.sh v0.3.0 --github

# 仅 Gitea
./scripts/release.sh v0.3.0 --gitea
```

需要的环境变量：
- `GITHUB_TOKEN` — GitHub 个人访问令牌
- `GITEA_TOKEN` — Gitea 个人访问令牌

## 8. 常见错误

| 错误 | 症状 | 解决方法 |
|------|------|----------|
| 版本号不一致 | 状态栏显示错误版本 | 更新所有 Cargo.toml + status.rs |
| 构建目录错误 | `target/release/serialrun.exe` 是旧文件 | 使用 `--target` 参数 |
| 缺少帮助文件 | 应用无法加载帮助文档 | 打包时包含 help_en.md + help_zh.md |
| 应用未关闭 | 构建失败或覆盖被锁定的文件 | 先关闭 serialrun.exe |
| 目标未安装 | 链接器报错 | `rustup target add <target>` |
| Cargo.lock 冲突 | 拉取代码后构建报错 | `cargo update` 或删除 Cargo.lock |
