# SerialRUN MCP API Reference

SerialRUN 内置 MCP (Model Context Protocol) 服务器，允许 AI 助手通过 TCP 连接控制串口设备。

## 连接信息

- **协议**: JSON-RPC over TCP
- **地址**: 127.0.0.1:9527（默认）
- **编码**: UTF-8
- **分隔符**: 换行符 `\n`

## 基本格式

### 请求

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "tool_name",
    "arguments": { ... }
  }
}
```

### 响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "response text"
      }
    ]
  }
}
```

### 错误响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -1,
    "message": "error description"
  }
}
```

---

## 工具列表（12 个）

### 1. list_ports

列出所有可用的串口设备。

**参数**: 无

**响应示例**:
```json
{
  "content": [{
    "type": "text",
    "text": "[\n  {\n    \"name\": \"COM1\",\n    \"description\": null,\n    \"manufacturer\": null\n  }\n]"
  }]
}
```

### 2. connect

连接到指定串口。如果未连接，会自动创建 port_owner。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| port | string | 是 | - | 串口名称（如 COM1, /dev/ttyUSB0） |
| baud_rate | integer | 否 | 115200 | 波特率 |

**请求示例**:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200}}}
```

**响应示例**:
```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Connected to COM3 at 115200 baud"}]}}
```

### 3. disconnect

断开当前串口连接。

**参数**: 无

### 4. send

发送数据到串口。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| data | string | 是 | - | 要发送的数据 |
| hex | boolean | 否 | false | 是否为十六进制格式 |

**请求示例**:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send","arguments":{"data":"AT\r\n","hex":false}}}
```

**HEX 格式示例**:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send","arguments":{"data":"48 65 6C 6C 6F","hex":true}}}
```

### 5. read

从串口读取数据。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| timeout_ms | integer | 否 | 1000 | 读取超时（毫秒） |

**响应示例**:
```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Read 5 bytes\nHEX: 4F 4B 0D 0A\nText: OK\r\n"}]}}
```

### 6. send_command

发送命令并等待响应（写入后轮询读取）。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| command | string | 是 | - | 要发送的命令 |
| timeout_ms | integer | 否 | 1000 | 响应超时（毫秒） |

**请求示例**:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"send_command","arguments":{"command":"AT","timeout_ms":1000}}}
```

### 7. modbus_read

读取 Modbus RTU 保持寄存器。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| slave_id | integer | 否 | 1 | 从站地址（1-247） |
| address | integer | 是 | - | 起始寄存器地址 |
| quantity | integer | 否 | 1 | 读取数量 |

**请求示例**:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"modbus_read","arguments":{"slave_id":1,"address":0,"quantity":10}}}
```

**响应示例**:
```json
{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Read 10 registers from slave 1\nHEX: 01 03 14 00 01 00 02...\nValues: [1, 2, 3, ...]"}]}}
```

### 8. modbus_write

写入 Modbus RTU 保持寄存器。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| slave_id | integer | 否 | 1 | 从站地址（1-247） |
| address | integer | 是 | - | 寄存器地址 |
| value | integer | 是 | - | 写入值（u16） |

### 9. plc_read

读取 PLC 预设品牌的所有寄存器。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| brand | string | 否 | "Siemens" | PLC 品牌：Siemens, Mitsubishi, Delta, Omron |
| slave_id | integer | 否 | 1 | 从站地址（1-247） |

### 10. plc_write

写入 PLC 寄存器。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| brand | string | 否 | "Siemens" | PLC 品牌 |
| slave_id | integer | 否 | 1 | 从站地址 |
| address | integer | 是 | - | 寄存器地址 |
| value | number | 是 | - | 写入值 |

### 11. get_access_log

查看 MCP 访问日志。

**参数**:
| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| limit | integer | 否 | 50 | 返回的最大条目数 |

**响应示例**:
```json
{
  "content": [{
    "type": "text",
    "text": "Active clients: 1\n\nAccess Log (last 3):\n[\n  {\n    \"time\": \"2026-05-29 10:00:00.123\",\n    \"ip\": \"192.168.1.100\",\n    \"action\": \"CALL\",\n    \"detail\": \"list_ports\"\n  }\n]"
  }]
}
```

### 12. get_device_info

获取当前设备识别信息。

**参数**: 无

**响应示例**:
```json
{
  "content": [{
    "type": "text",
    "text": "Device: SerialRUN\nStatus: Connected\nActive clients: 1\nTotal access log entries: 10\nServer: MCP v0.2.0\nProtocol: JSON-RPC over TCP"
  }]
}
```

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| -1 | 通用错误 |
| -32601 | 未知方法 |
| -32602 | 参数错误 |
| -32700 | 解析错误 |

---

## 使用流程

### 1. 初始化连接

```bash
# 连接到 MCP 服务器
nc 127.0.0.1 9527

# 发送初始化请求
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

### 2. 列出工具

```bash
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

### 3. 连接串口

```bash
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"connect","arguments":{"port":"COM3","baud_rate":115200}}}
```

### 4. 发送数据

```bash
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send","arguments":{"data":"Hello\r\n"}}}
```

### 5. 读取响应

```bash
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read","arguments":{"timeout_ms":1000}}}
```

### 6. 断开连接

```bash
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"disconnect","arguments":{}}}
```

---

## 注意事项

1. **端口独占**: Windows 上串口是独占资源，同一时间只能有一个连接
2. **超时设置**: 建议 send_command 的 timeout_ms 设置为 500-2000ms
3. **HEX 格式**: 十六进制数据用空格分隔，如 "48 65 6C 6C 6F"
4. **文本换行**: 发送文本时会自动添加 \r\n 结束符
5. **访问日志**: 所有操作自动记录客户端 IP，可用于追溯
6. **并发限制**: 多个客户端可同时连接，但串口操作会排队执行

---

## Python 客户端示例

```python
import socket
import json
import time

def mcp_call(host, port, method, params=None, req_id=1):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(5)
    sock.connect((host, port))
    
    msg = {'jsonrpc': '2.0', 'id': req_id, 'method': method}
    if params:
        msg['params'] = params
    sock.sendall((json.dumps(msg) + '\n').encode())
    
    time.sleep(0.3)
    data = b''
    while b'\n' not in data:
        chunk = sock.recv(4096)
        if not chunk:
            break
        data += chunk
    
    sock.close()
    return json.loads(data.decode().strip()) if data else None

# 使用示例
result = mcp_call('127.0.0.1', 9527, 'tools/call', {
    'name': 'connect',
    'arguments': {'port': 'COM3', 'baud_rate': 115200}
})
print(result)
```

---

## JavaScript/Node.js 客户端示例

```javascript
const net = require('net');

function mcpCall(host, port, method, params = null, reqId = 1) {
    return new Promise((resolve, reject) => {
        const client = net.createConnection({ host, port }, () => {
            const msg = { jsonrpc: '2.0', id: reqId, method };
            if (params) msg.params = params;
            client.write(JSON.stringify(msg) + '\n');
        });
        
        let data = '';
        client.on('data', (chunk) => {
            data += chunk.toString();
            if (data.includes('\n')) {
                client.end();
                resolve(JSON.parse(data.trim()));
            }
        });
        
        client.on('error', reject);
        setTimeout(() => { client.end(); reject(new Error('Timeout')); }, 5000);
    });
}

// 使用示例
mcpCall('127.0.0.1', 9527, 'tools/call', {
    name: 'connect',
    arguments: { port: 'COM3', baud_rate: 115200 }
}).then(console.log);
```
