#!/usr/bin/env python3
"""Test all MCP set_config parameters — verify GUI sync."""
import socket, json, time, sys

MCP_HOST, MCP_PORT = "127.0.0.1", 9527
DELAY = 1.5  # seconds between each change for GUI observation

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

    def call(self, tool_name, args=None):
        self.request_id += 1
        req = {"jsonrpc": "2.0", "id": self.request_id, "method": "tools/call",
               "params": {"name": tool_name, "arguments": args or {}}}
        self.sock.sendall((json.dumps(req) + "\n").encode())
        self.sock.settimeout(8.0)
        deadline = time.time() + 8.0
        while time.time() < deadline:
            try:
                chunk = self.sock.recv(4096)
                if not chunk: return None
                self._buf += chunk
                while b"\n" in self._buf:
                    line, self._buf = self._buf.split(b"\n", 1)
                    if line.strip():
                        resp = json.loads(line.decode())
                        if "result" in resp:
                            return resp["result"].get("content", [{}])[0].get("text", "")
            except socket.timeout: continue
        return None

    def get_config(self, key=None):
        args = {"key": key} if key else {}
        return self.call("get_config", args)

    def set_config(self, key, value):
        return self.call("set_config", {"key": key, "value": value})

def main():
    print("═══ MCP Config Sync Test ═══\n")
    print("Observe the GUI while each setting changes!\n")

    c = McpClient(MCP_HOST, MCP_PORT)
    print("[OK] Connected\n")

    # First, get current config
    print("── Current Config ──")
    cfg = c.get_config()
    print(cfg[:300] if cfg else "ERROR")
    print()

    # Connect to COM4 first
    print("── Connect ──")
    r = c.call("connect", {"port": "COM4", "baud_rate": 9600})
    print(r[:80] if r else "ERROR")
    time.sleep(DELAY)

    # Test each config parameter
    tests = [
        # (key, value, description, check_ui)
        ("hex_mode", True, "Enable HEX mode — terminal should switch to hex display", "hex_mode checkbox"),
        ("hex_mode", False, "Disable HEX mode — terminal back to text", "hex_mode checkbox"),

        ("show_timestamp", False, "Hide timestamps — timestamps disappear from terminal", "显示时间戳 checkbox"),
        ("show_timestamp", True, "Show timestamps — timestamps reappear", "显示时间戳 checkbox"),

        ("auto_scroll", False, "Disable auto-scroll", "自动滚动 checkbox"),
        ("auto_scroll", True, "Enable auto-scroll", "自动滚动 checkbox"),

        ("line_ending", "CR", "Set line ending to CR", "行尾 dropdown"),
        ("line_ending", "LF", "Set line ending to LF", "行尾 dropdown"),
        ("line_ending", "CRLF", "Set line ending to CRLF", "行尾 dropdown"),
        ("line_ending", "None", "Set line ending to None", "行尾 dropdown"),

        ("dtr", True, "Set DTR ON — DTR checkbox should check", "DTR checkbox"),
        ("dtr", False, "Set DTR OFF — DTR checkbox should uncheck", "DTR checkbox"),

        ("rts", True, "Set RTS ON — RTS checkbox should check", "RTS checkbox"),
        ("rts", False, "Set RTS OFF — RTS checkbox should uncheck", "RTS checkbox"),

        ("auto_send_enabled", True, "Enable auto-send — auto-send button should activate", "自动发送 button"),
        ("auto_send_enabled", False, "Disable auto-send", "自动发送 button"),

        ("auto_send_interval_ms", 2000, "Set auto-send interval to 2000ms", "auto-send DragValue"),

        ("rx_auto_aggregate", False, "Disable RX auto-aggregate — manual T/O mode", "T/O checkbox"),
        ("rx_auto_aggregate", True, "Enable RX auto-aggregate — auto T/O", "T/O checkbox"),

        ("auto_reply_enabled", True, "Enable auto-reply", "自动回复 checkbox"),
        ("auto_reply_pattern", "HELLO", "Set auto-reply pattern to HELLO", "自动回复 pattern"),
        ("auto_reply_response", "WORLD", "Set auto-reply response to WORLD", "自动回复 response"),
        ("auto_reply_enabled", False, "Disable auto-reply", "自动回复 checkbox"),

        ("keep_input", True, "Keep input after send", "保留输入 checkbox"),
        ("keep_input", False, "Clear input after send", "保留输入 checkbox"),
    ]

    for key, value, desc, ui_elem in tests:
        print(f"── set_config({key} = {value}) ──")
        print(f"    UI should change: {ui_elem}")
        print(f"    Description: {desc}")
        r = c.set_config(key, value)
        if r:
            # Extract just the result message
            first_line = r.split('\n')[0] if r else ""
            print(f"    Result: {first_line}")
        else:
            print(f"    Result: ERROR")
        time.sleep(DELAY)

        # Read back to verify
        readback = c.get_config(key)
        if readback:
            print(f"    Readback: {readback.strip()[:80]}")
        print()

    # Final config snapshot
    print("── Final Config ──")
    cfg = c.get_config()
    print(cfg[:500] if cfg else "ERROR")

    # Disconnect
    print("\n── Disconnect ──")
    c.call("disconnect")
    c.close()
    print("═══ Config Test Complete ═══")

if __name__ == "__main__":
    main()
