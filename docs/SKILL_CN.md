# SerialRUN 技能参考

[English](SKILL.md)

---

## 概述

SerialRUN 为嵌入式开发工作流提供串口通信能力，支持 CLI 自动化和交互式 GUI 使用。

## 核心能力

### 1. 端口管理

- 枚举所有可用串口
- 配置波特率、数据位、停止位、校验位、流控
- 连接/断开/自动重连

### 2. 数据通信

- **文本模式**: UTF-8 编码收发
- **十六进制模式**: 十六进制数据展示和传输
- **混合模式**: 同时显示文本和十六进制

### 3. 协议分析

- 内置 Modbus RTU 解析器（CRC-16、帧解码）
- 自定义协议模式（正则匹配）
- 协议日志和流量分析

### 4. 脚本操作

- 录制串口会话为 JSON 或文本脚本
- 按时序回放脚本
- 脚本编辑和参数化

### 5. 文件传输

- XMODEM（128字节块）
- YMODEM（1024字节块，支持批量）
- ZMODEM（支持断点续传）

### 6. 插件系统

- 动态加载（.so / .dll / .dylib）
- C FFI 接口
- 自定义命令执行

## 集成方式

### CLI 管道

```bash
serialrun list --format json | jq '.ports[0].name'
serialrun send COM1 "test" && serialrun monitor COM1 -t -l output.log
```

### Agent JSON 接口

```bash
serialrun agent list-ports           # 列出端口
serialrun agent COM1 send "data"     # 发送
serialrun agent COM1 read            # 读取
serialrun agent COM1 run-script.txt  # 执行脚本
```

### 插件 API

```rust
#[no_mangle]
pub extern "C" fn plugin_get_info() -> *mut c_char;

#[no_mangle]
pub extern "C" fn plugin_execute(
    command: *const c_char,
    params: *const c_char
) -> *mut c_char;
```

## 最佳实践

### 端口配置

- 始终确保设备和助手的波特率一致
- 默认使用 8N1（8数据位、无校验、1停止位）
- 高吞吐量应用启用流控

### 数据处理

- 二进制协议（Modbus、自定义）使用十六进制模式
- 调试会话启用时间戳
- 记录重要会话供后续分析

### 脚本开发

- 从手动录制开始
- 在命令之间添加适当延迟
- 自动化前充分测试脚本
