# SerialRUN 发布 SOP

本文档说明如何发布一个多平台版本。所有安装包由 GitHub Actions 在云端构建，
本地无需安装 macOS / Linux 交叉编译工具链。

## 版本规则

- 版本号格式：`X.Y.Z`，发布时填入的版本不能低于当前 `crates/serialrun-gui/Cargo.toml` 中的版本。
  - 若填入的版本更高（如当前 0.4.5，填入 0.4.6），流水线会自动递增所有版本号并提交。
  - 若代码里已经手动改成目标版本（如当前已是 0.4.5，填入 0.4.5），流水线只创建标签并发布，不重复提交。
- 当前版本：**0.4.5**（发布时以 `crates/serialrun-gui/Cargo.toml` 为准）。
- 发布标签：`vX.Y.Z`，例如 `v0.4.5`。
- 历史版本只增不减；不要删除或复用已发布的标签。

## 发布产物

| 平台 | 文件 |
|------|------|
| Windows x64 | `serialrun-X.Y.Z-windows-x64.zip` |
| macOS Apple Silicon (M1–M4) | `serialrun-X.Y.Z-macos-arm64.dmg` / `.zip` |
| macOS Intel | `serialrun-X.Y.Z-macos-x64.dmg` / `.zip` |
| Linux x86_64 | `serialrun-X.Y.Z-linux-x64.tar.gz` |

## 发布前检查（本地）

```bash
# 1. 工作区干净，且与 GitHub 同步
git status
git pull github master

# 2. 至少编译检查一次
cargo check -p serialrun-gui

# 3. 运行测试
cargo test --workspace

# 4. 确认最新版本号
grep '^version' crates/serialrun-gui/Cargo.toml
```

> 如果本次发布包含用户可见的改动，先更新 `CHANGELOG.md` 和对应文档
>（`README.md`、`README_CN.md`、`docs/help_*.md`），再提交。

## 方式 A：GitHub Actions 一键发布（推荐）

1. 打开仓库 **Actions → Release → Run workflow**。
2. 选择分支（通常是 `master`），输入版本号（例如 `0.4.5`）。
3. 点击 **Run workflow**。

流水线会自动完成：

1. **prepare**：校验版本号 → 运行 `scripts/bump-version.sh` 更新所有版本号 →
   提交 `chore: release vX.Y.Z` → 创建并推送标签 `vX.Y.Z`。
2. **build-windows / build-macos (arm64, x64) / build-linux**：并行构建并打包。
3. **release**：创建 GitHub Release，上传全部安装包，并设为 Latest。

版本号同步范围：

- 5 个 crate 的 `Cargo.toml` 与 `Cargo.lock`
- 状态栏 `SerialRUN vX.Y.Z`
- macOS `Info.plist`（`CFBundleVersion` / `CFBundleShortVersionString`）
- `README.md` / `README_CN.md` 下载表格

## 方式 B：本地打标签发布

适用于已手动更新版本号并提交的场景：

```bash
# 版本号必须与 Cargo.toml 完全一致
VERSION=0.4.5

git add -A
git commit -m "chore: release v$VERSION"
git tag -a "v$VERSION" -m "SerialRUN v$VERSION"
git push github master
git push github "v$VERSION"
```

标签推送会自动触发 Release 流水线（prepare 只做版本一致性校验，不再递增版本）。

## 发布后验证

1. 打开 **Releases**，确认 4 个平台、共 6 个文件全部上传：
   - Windows ZIP
   - macOS arm64 DMG + ZIP
   - macOS x64 DMG + ZIP
   - Linux tar.gz
2. 下载 Windows 包，解压运行 `serialrun.exe`，确认状态栏显示新版本号。
3. 在 macOS 上确认：
   - Apple Silicon 机器下载 `macos-arm64`，Intel 机器下载 `macos-x64`
   - 首次打开：右键 → 打开；若提示“已损坏”，执行
     `xattr -dr com.apple.quarantine /Applications/SerialRUN.app`
4. Linux 解压后按 `INSTALL.txt` 安装运行依赖并启动。
5. 更新 README/官网下载链接（README 已由流水线自动更新）。

## 回滚

- **安装包有问题**：删除 Release 中的对应资产，重新运行失败的任务或重新发布补丁版本。
- **标签打错**：不要对已发布标签 force push。发布一个更高的补丁版本（如 `0.4.5`）覆盖问题版本。
- **版本号已递增但构建失败**：在 Actions 中点击 **Re-run failed jobs**，标签和提交会复用。

## 安全要求

- 仓库、工作流和文档中不得出现任何 Token、证书、私钥或本机路径。
- GitHub Actions 仅使用内置 `GITHUB_TOKEN`（`permissions: contents: write`）。
- 如需 macOS 正式签名/公证，将证书保存为仓库 Secrets（例如
  `MACOS_CODESIGN_IDENTITY`），不要提交到代码库；未配置时使用 ad-hoc 签名。
- 本地 git remote 不要内嵌 Token，使用 `gh auth login` 或凭据管理器。
