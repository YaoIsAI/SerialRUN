# 插件开发自动化

## 概述

通过 GitHub Issue 自动触发插件开发流程。

## 工作流程

```
用户提 Issue (标签: plugin-request)
    ↓
Webhook 服务器接收
    ↓
自动创建插件请求
    ↓
监控脚本检测
    ↓
自动开发插件
    ↓
构建 + 测试
    ↓
完成通知
```

## 快速开始

### 1. 启动 Webhook 服务器

```bash
# 生成 webhook secret
WEBHOOK_SECRET=$(openssl rand -hex 20)
echo "Your webhook secret: $WEBHOOK_SECRET"

# 启动服务器
python3 scripts/webhook_server.py --port 9876 --secret $WEBHOOK_SECRET
```

### 2. 配置 GitHub Webhook

1. 打开 GitHub 仓库 → Settings → Webhooks → Add webhook
2. Payload URL: `http://YOUR_IP:9876/webhook`
3. Content type: `application/json`
4. Secret: 你生成的 secret
5. Events: 选择 "Issues"

### 3. 启动监控

```bash
# 单次检查
./scripts/monitor_plugins.sh

# 持续监控
./scripts/monitor_plugins.sh --watch --interval 30
```

### 4. 用户提交插件需求

在 GitHub 创建 Issue，使用 "Plugin Request" 模板，添加标签 `plugin-request`。

## 文件说明

| 文件 | 说明 |
|------|------|
| `scripts/webhook_server.py` | Webhook 服务器，接收 GitHub 事件 |
| `scripts/auto_develop_plugin.sh` | 自动开发插件脚本 |
| `scripts/monitor_plugins.sh` | 监控新请求脚本 |
| `.github/ISSUE_TEMPLATE/plugin_request.md` | Issue 模板 |
| `~/.serialrun/pending_plugins/` | 待处理的插件请求 |
| `~/.serialrun/processed_plugins/` | 已处理的插件请求 |

## Issue 模板

用户提交 Issue 时需要填写：

- **Plugin Name**: 插件名称
- **Description**: 描述
- **Features**: 功能列表
- **Serial Protocol**: 串口协议（可选）
- **Example Commands**: 示例命令（可选）

## 手动测试

```bash
# 创建测试请求
cat > ~/.serialrun/pending_plugins/issue-999.json << 'EOF'
{
  "issue_number": 999,
  "issue_url": "https://github.com/YaoIsAI/SerialRUN/issues/999",
  "title": "Test Plugin",
  "created_at": "2026-06-09T12:00:00Z",
  "plugin_name": "serialrun-test-plugin",
  "description": "A test plugin",
  "features": ["echo", "add"],
  "status": "pending"
}
EOF

# 运行监控
./scripts/monitor_plugins.sh
```

## 与 Claude Code 集成

你也可以手动触发：

```bash
# 检查新请求
./scripts/monitor_plugins.sh

# 或者直接处理某个请求
./scripts/auto_develop_plugin.sh ~/.serialrun/pending_plugins/issue-123.json
```
