#!/usr/bin/env python3
"""Slow MCP test — 2s delay between each step for GUI observation."""
import socket, json, time, sys

MCP_HOST, MCP_PORT = "127.0.0.1", 9527
TEST_PORT, TEST_BAUD = "COM4", 9600
DELAY = 2.0  # seconds between each operation

class McpClient:
    def __init__(self, host, port):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(10.0)
        self.sock.connect((host, port))
        self.request_id = 0
        self._buf = b""

    def close(self):
        try: self.sock.close()
        except: pass

    def call_tool(self, tool_name, arguments=None):
        self.request_id += 1
        req = {"jsonrpc": "2.0", "id": self.request_id, "method": "tools/call",
               "params": {"name": tool_name, "arguments": arguments or {}}}
        self.sock.sendall((json.dumps(req) + "\n").encode())
        self.sock.settimeout(8.0)
        deadline = time.time() + 8.0
        while time.time() < deadline:
            try:
                chunk = self.sock.recv(4096)
                if not chunk: return None, "Connection closed"
                self._buf += chunk
                while b"\n" in self._buf:
                    line, self._buf = self._buf.split(b"\n", 1)
                    if line.strip():
                        resp = json.loads(line.decode())
                        if "error" in resp and resp["error"]:
                            return resp, f"Error: {resp['error']['message']}"
                        if "result" in resp:
                            text = resp["result"].get("content", [{}])[0].get("text", "")
                            return resp, text
            except socket.timeout: continue
        return None, "Timeout"

def log(name, passed, detail=""):
    print(f"[{'PASS' if passed else 'FAIL'}] {name}" + (f" — {detail[:120]}" if detail else ""))

def main():
    print(f"═══ Slow MCP Test (delay={DELAY}s between steps) ═══")
    print(f"Target: {MCP_HOST}:{MCP_PORT} → MCU {TEST_PORT} @ {TEST_BAUD} baud")
    print(f"Observe the GUI terminal — try sending manual commands during the test!\n")

    c = McpClient(MCP_HOST, MCP_PORT)
    print("[OK] Connected to MCP server\n")

    steps = [
        # (name, tool, args)
        ("connect", "connect", {"port": TEST_PORT, "baud_rate": TEST_BAUD}),
        ("status", "status", {}),
        ("send: AT+HELP", "send_command", {"command": "AT+HELP", "timeout_ms": 1500}),
        ("send: AT+STATUS", "send_command", {"command": "AT+STATUS", "timeout_ms": 1500}),
        ("send: Hello World", "send_command", {"command": "Hello World", "timeout_ms": 1500}),
        ("switch to Modbus", "send_command", {"command": "AT+APP=MODBUS", "timeout_ms": 1500}),
        ("modbus_read FC03 (4 regs)", "modbus_read", {"slave_id": 1, "address": 0, "quantity": 4}),
        ("modbus_write FC06 (=200)", "modbus_write", {"slave_id": 1, "address": 0, "value": 200}),
        ("modbus_read back", "modbus_read", {"slave_id": 1, "address": 0, "quantity": 1}),
        ("switch to PLC", "send_command", {"command": "AT+APP=PLC", "timeout_ms": 1500}),
        ("plc_set_brand Siemens", "send_command", {"command": "AT+BRAND=SIEMENS", "timeout_ms": 1500}),
        ("plc_read Siemens", "plc_read", {"brand": "Siemens", "slave_id": 1}),
        ("switch to ECHO", "send_command", {"command": "AT+APP=ECHO", "timeout_ms": 1500}),
        ("disconnect", "disconnect", {}),
    ]

    for i, (name, tool, args) in enumerate(steps):
        print(f"── Step {i+1}/{len(steps)}: {name} ──")
        resp, text = c.call_tool(tool, args)
        if resp is None:
            log(name, False, text)
        else:
            log(name, True, text[:100])
        if i < len(steps) - 1:
            print(f"    (waiting {DELAY}s — try sending a manual command now!)\n")
            time.sleep(DELAY)

    c.close()
    print("\n═══ Test Complete ═══")

if __name__ == "__main__":
    main()
