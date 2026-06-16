# Demo Title: <一句话描述这个 demo 在做什么>

## 📋 Basic Info / 基本信息

| Item / 项目 | Content / 内容 |
|-------------|----------------|
| **Scenario / 场景** | e.g. Arduino UNO upload + serial monitor |
| **Author / 作者** | @your-github-name |
| **SerialRUN Version / 版本** | v0.4.3+ |
| **AI Tool / AI 工具** | Claude Desktop / Cursor / Other |
| **Video Link / 视频链接** | B站 / YouTube / Feishu URL |
| **Submit Date / 提交日期** | YYYY-MM-DD |

## 🎯 Pain Point Solved / 解决的痛点

> EN: Briefly describe how painful it was before, and how SerialRUN MCP solves it.
> 
> CN: 用一段话描述：以前做这件事有多麻烦？现在用 SerialRUN MCP 怎么解决的？

## 🛠 Environment / 环境准备

- **Hardware / 硬件**: <列出用到的硬件>
- **Software / 软件**: <SerialRUN 版本、Claude Desktop 版本、Python 环境等>
- **Network / 网络**: <是否需要联网 / 局域网>

## 📝 Steps / 操作步骤

### Step 1: Configure SerialRUN MCP Server / 配置 SerialRUN MCP Server

![step1-config](./screenshots/step1-config.png)

```json
{
  "mcpServers": {
    "serialrun": {
      "command": "serialrun",
      "args": ["mcp", "serve"],
      "env": {}
    }
  }
}
```

### Step 2: Launch AI Tool and Load MCP / 启动 AI 工具并加载 MCP

![step2-launch](./screenshots/step2-launch.png)

### Step 3: Drive Debugging with Natural Language / 用自然语言驱动调试

![step3-prompt](./screenshots/step3-prompt.png)

Key prompt:
```
"Read COM3 serial port, filter out lines containing ERROR"
"读取 COM3 串口，过滤出包含 ERROR 的行"
```

### Step 4: Verify Result / 验证结果

![step4-result](./screenshots/step4-result.png)

## 💡 Prompt Library / 关键 prompt 模板

| Scenario / 场景 | Prompt | Notes / 备注 |
|-----------------|--------|--------------|
| Read serial / 串口读取 | `Read <port> serial data` | |
| Parse protocol / 协议解析 | `Parse <protocol> frame` | |
| Scan bus / 设备扫描 | `Scan I2C bus` | |

## ⚠️ Pitfalls / 踩坑记录

| Pitfall / 坑 | Symptom / 现象 | Fix / 解决 |
|--------------|----------------|------------|
| Port occupied / 端口占用 | AI can't open COM3 | Close serial monitor process |
| Encoding error / 编码错误 | Garbled Chinese | Switch to UTF-8 |

## 📁 Attachments / 附件说明

- `screenshots/` - 关键操作截图 / Key operation screenshots
- `prompts/` - 完整 prompt 文件 / Complete prompt files
- `configs/` - 配置文件（脱敏）/ Config files (sanitized)
- `video-link.md` - 视频链接 / Video link

## 🪪 License / 许可

This demo is released under MIT License. The SerialRUN team reserves the right to reference it in official documentation/videos.
