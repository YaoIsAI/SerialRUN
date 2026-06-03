# SerialRUN 自动实现系统 — 使用手册

> 本地文档，不推送到 GitHub

---

## 概述

自动读取 GitHub Issue → Claude Code 自动实现 → 构建验证 → 关闭 Issue。

## 快速开始

### 1. 配置 Token（一次性）

```bash
# 添加到 ~/.bashrc 或 ~/.zshrc
export GITHUB_TOKEN="ghp_你的token"

# 生效
source ~/.bashrc
```

### 2. 创建 Issue

在 GitHub 创建 Issue，打上 `auto-implement` 标签：

```
标题: [Auto] 添加 XXX 功能
标签: auto-implement
内容: 具体需求描述...
```

### 3. 运行脚本

```bash
cd /path/to/serialrun
./scripts/auto-implement.sh
```

## 命令参考

| 命令 | 说明 |
|------|------|
| `./scripts/auto-implement.sh` | 处理所有 auto-implement 标签的 Issue |
| `./scripts/auto-implement.sh --issue 5` | 只处理 Issue #5 |
| `./scripts/auto-implement.sh --dry-run` | 预览模式，不执行 |

## 工作流程

```
GitHub Issue (标签: auto-implement)
    ↓
脚本读取 Issue 内容
    ↓
生成 .auto-task.md 任务文件
    ↓
调用 claude -p 非交互模式
    ↓
Claude Code 自动分析 → 编码 → 构建
    ↓
成功: 评论"完成" + 关闭 Issue
失败: 评论错误信息
```

## 标签约定

| 标签 | 行为 |
|------|------|
| `auto-implement` | 脚本自动处理 |
| `auto-fix` | 可扩展为自动修复 |
| `enhancement` | 需人工确认，脚本忽略 |

## 跨设备使用

在新设备（Mac/Linux/Windows）上使用：

```bash
# 1. Clone 项目
git clone -b full http://192.168.31.85:38633/yao/serialrun.git
cd serialrun

# 2. 安装 Claude Code CLI
# macOS/Linux:
curl -fsSL https://cli.anthropic.com/install.sh | sh
# 或参考 https://docs.anthropic.com/claude-code

# 3. 配置 Token
export GITHUB_TOKEN="ghp_你的token"

# 4. 运行
./scripts/auto-implement.sh
```

**前提条件：**
- 项目已从 Gitea clone（`full` 分支）
- 已安装 Claude Code CLI
- 已配置 `GITHUB_TOKEN` 环境变量
- 设备可访问 GitHub API

## 安全规则

- Token 只通过环境变量 `GITHUB_TOKEN` 使用
- 脚本不推送到 GitHub，只保留本地
- 脚本中不能硬编码任何 Token
- `.auto-task.md` 已加入 .gitignore

## 限制

- 只修改 `serialrun-core` / `serialrun-plugin-api` / `plugins` / `docs`
- 不修改 GUI 代码（proprietary）
- 不修改 `.gitignore` 和 `.git`
- 必须通过 `cargo build`

## 定时自动化（可选）

```bash
# 每 30 分钟自动检查
crontab -e
*/30 * * * * cd /path/to/serialrun && ./scripts/auto-implement.sh >> /tmp/auto-implement.log 2>&1
```

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| `GITHUB_TOKEN: unSet` | 运行 `export GITHUB_TOKEN=ghp_xxx` |
| `No issues found` | 没有 auto-implement 标签的 Issue |
| `Claude Code failed` | 检查 claude CLI 是否安装 |
| `cargo build failed` | Issue 需求可能超出自动实现范围 |
