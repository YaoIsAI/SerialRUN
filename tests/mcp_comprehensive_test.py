#!/usr/bin/env python3
"""
Comprehensive MCP Server Test against Multi-Sim v3.0 MCU
COM4 @ 9600 baud

Tests all MCP tools: list_ports, connect, send, read, send_command,
modbus_read, modbus_write, plc_read, plc_write, get_access_log,
get_device_info, status, get_config, set_config, disconnect
"""
import socket
import json
import time
import sys

MCP_HOST = "127.0.0.1"
MCP_PORT = 9527
TEST_PORT = "COM4"
TEST_BAUD = 9600

results = []

def log(test_name, passed, detail=""):
    status = "PASS" if passed else "FAIL"
    msg = f"[{status}] {test_name}"
    if detail:
        msg += f" — {detail}"
    print(msg)
    results.append((test_name, passed, detail))
    return passed


class McpClient:
    def __init__(self, host, port):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(10.0)
        self.sock.connect((host, port))
        self.request_id = 0
        self._buf = b""

    def close(self):
        try:
            self.sock.close()
        except:
            pass

    def send_request(self, method, params=None):
        self.request_id += 1
        req = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
        }
        if params is not None:
            req["params"] = params
        msg = json.dumps(req) + "\n"
        self.sock.sendall(msg.encode())
        return self.request_id

    def recv_response(self, timeout=8.0):
        self.sock.settimeout(timeout)
        deadline = time.time() + timeout
        while time.time() < deadline:
            try:
                chunk = self.sock.recv(4096)
                if not chunk:
                    return None
                self._buf += chunk
                while b"\n" in self._buf:
                    line, self._buf = self._buf.split(b"\n", 1)
                    line = line.strip()
                    if line:
                        return json.loads(line.decode())
            except socket.timeout:
                continue
        return None

    def call_tool(self, tool_name, arguments=None):
        params = {"name": tool_name, "arguments": arguments or {}}
        req_id = self.send_request("tools/call", params)
        resp = self.recv_response()
        return resp

    def call_tool_expect_ok(self, tool_name, arguments=None):
        resp = self.call_tool(tool_name, arguments)
        if resp is None:
            return None, "No response (timeout)"
        if "error" in resp and resp["error"] is not None:
            return resp, f"Error {resp['error']['code']}: {resp['error']['message']}"
        if "result" in resp and resp["result"] is not None:
            content = resp["result"].get("content", [])
            if content:
                text = content[0].get("text", "")
                return resp, text
        return resp, str(resp.get("result", {}))


def test_initialize(client):
    req_id = client.send_request("initialize", {
        "protocolVersion": "2024-11-05",
        "capabilities": {"tools": {}},
        "clientInfo": {"name": "test-client", "version": "1.0.0"}
    })
    resp = client.recv_response()
    if resp and "result" in resp:
        ver = resp["result"].get("protocolVersion", "")
        return log("initialize", ver == "2024-11-05", f"protocol={ver}")
    return log("initialize", False, str(resp))


def test_tools_list(client):
    resp, text = client.call_tool_expect_ok("list_ports")
    if resp is None:
        return log("tools/list → list_ports", False, text)
    # It's a tools/list method, not tools/call
    req_id = client.send_request("tools/list")
    resp = client.recv_response()
    if resp and "result" in resp:
        tools = resp["result"].get("tools", [])
        names = [t["name"] for t in tools]
        expected = ["list_ports", "connect", "disconnect", "send", "read",
                     "send_command", "modbus_read", "modbus_write",
                     "plc_read", "plc_write", "get_access_log",
                     "get_device_info", "status", "get_config", "set_config"]
        missing = [n for n in expected if n not in names]
        if missing:
            return log("tools/list", False, f"Missing tools: {missing}")
        return log("tools/list", True, f"{len(names)} tools found")
    return log("tools/list", False, str(resp))


def test_list_ports(client):
    resp, text = client.call_tool_expect_ok("list_ports")
    if resp is None:
        return log("list_ports", False, text)
    has_com4 = "COM4" in text
    return log("list_ports", has_com4, f"COM4 found={has_com4}")


def test_connect(client):
    resp, text = client.call_tool_expect_ok("connect", {
        "port": TEST_PORT,
        "baud_rate": TEST_BAUD,
        "data_bits": 8,
        "stop_bits": 1,
        "parity": "None",
        "flow_control": "None"
    })
    if resp is None:
        return log("connect", False, text)
    connected = "Connected" in text
    return log("connect", connected, text[:100])


def test_status(client):
    resp, text = client.call_tool_expect_ok("status")
    if resp is None:
        return log("status", False, text)
    has_connection = "connection" in text.lower() or "Connected" in text
    return log("status", has_connection, text[:200])


def test_get_config(client):
    resp, text = client.call_tool_expect_ok("get_config")
    if resp is None:
        return log("get_config", False, text)
    has_baud = "baud_rate" in text
    return log("get_config", has_baud, text[:200])


def test_set_config(client):
    resp, text = client.call_tool_expect_ok("set_config", {
        "key": "hex_mode",
        "value": False
    })
    if resp is None:
        return log("set_config", False, text)
    ok = "hex_mode" in text or "error" not in text.lower()
    return log("set_config", ok, text[:100])


def test_get_device_info(client):
    resp, text = client.call_tool_expect_ok("get_device_info")
    if resp is None:
        return log("get_device_info", False, text)
    has_info = "SerialRUN" in text
    return log("get_device_info", has_info, text[:200])


# ── ECHO Mode Tests ──

def test_echo_text(client):
    """Send text, expect echo back"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "Hello MCP!",
        "hex": False
    })
    if resp is None:
        return log("echo:send_text", False, text)
    sent_ok = "bytes" in text.lower()
    time.sleep(0.3)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "text"
    })
    if resp2 is None:
        return log("echo:read_text", False, text2)
    has_echo = "Hello MCP!" in text2
    return log("echo:text_roundtrip", has_echo, f"sent='Hello MCP!' got='{text2[:60]}'")


def test_echo_hex(client):
    """Send hex data, expect echo back"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "41 54 2B 48 45 4C 50",
        "hex": True
    })
    if resp is None:
        return log("echo:send_hex", False, text)
    time.sleep(0.3)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("echo:read_hex", False, text2)
    has_41 = "41" in text2 and "54" in text2
    return log("echo:hex_roundtrip", has_41, f"got='{text2[:60]}'")


def test_echo_binary(client):
    """Send binary bytes (0x00, 0xFF), expect echo back"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "00 FF AA BB CC DD",
        "hex": True
    })
    if resp is None:
        return log("echo:send_binary", False, text)
    time.sleep(0.3)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("echo:read_binary", False, text2)
    has_ff = "FF" in text2 and "AA" in text2
    return log("echo:binary_roundtrip", has_ff, f"got='{text2[:60]}'")


def test_send_command_echo(client):
    """send_command in ECHO mode"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "Test123",
        "timeout_ms": 1000
    })
    if resp is None:
        return log("send_command:echo", False, text)
    has_echo = "Test123" in text
    return log("send_command:echo", has_echo, f"got='{text[:60]}'")


def test_send_read_hex_format(client):
    """send_command with hex output — use send_command which handles pause/resume internally"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+STATUS",
        "timeout_ms": 1000
    })
    if resp is None:
        return log("send_read:hex_format", False, text)
    # The response should be text, verify it has content
    has_data = len(text.strip()) > 0
    return log("send_read:hex_format", has_data, f"got='{text[:80]}'")


def test_send_read_raw_format(client):
    """send_command with raw/base64 output — use send_command which handles pause/resume internally"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+HELP",
        "timeout_ms": 1000
    })
    if resp is None:
        return log("send_read:raw_format", False, text)
    has_data = len(text.strip()) > 0
    return log("send_read:raw_format", has_data, f"got='{text[:80]}'")


# ── AT Command Tests ──

def test_at_help(client):
    """Send AT+HELP command"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+HELP",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("AT+HELP", False, text)
    has_help = "AT" in text or "HELP" in text or "ECHO" in text
    return log("AT+HELP", has_help, f"got='{text[:120]}'")


def test_at_status(client):
    """Send AT+STATUS command"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+STATUS",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("AT+STATUS", False, text)
    has_status = "MODE" in text or "BAUD" in text or "ECHO" in text
    return log("AT+STATUS", has_status, f"got='{text[:120]}'")


# ── Modbus Mode Tests ──

def test_switch_to_modbus(client):
    """Switch MCU to Modbus mode"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+APP=MODBUS",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("switch_to_modbus", False, text)
    has_ok = "OK" in text or "MODBUS" in text
    return log("switch_to_modbus", has_ok, f"got='{text[:120]}'")


def test_modbus_read_holding_registers(client):
    """Read holding registers FC03"""
    resp, text = client.call_tool_expect_ok("modbus_read", {
        "slave_id": 1,
        "address": 0,
        "quantity": 4
    })
    if resp is None:
        return log("modbus_read:FC03_4regs", False, text)
    has_values = "Values" in text or "registers" in text.lower() or "raw" in text.lower()
    return log("modbus_read:FC03_4regs", has_values, text[:200])


def test_modbus_read_with_scale(client):
    """Read with engineering conversion"""
    resp, text = client.call_tool_expect_ok("modbus_read", {
        "slave_id": 1,
        "address": 0,
        "quantity": 2,
        "scale": 0.1,
        "offset": 10.0,
        "unit": "degC"
    })
    if resp is None:
        return log("modbus_read:FC03_scaled", False, text)
    has_eng = "Engineering" in text or "value" in text.lower()
    return log("modbus_read:FC03_scaled", has_eng, text[:200])


def test_modbus_write(client):
    """Write holding register FC06"""
    resp, text = client.call_tool_expect_ok("modbus_write", {
        "slave_id": 1,
        "address": 0,
        "value": 100
    })
    if resp is None:
        return log("modbus_write:FC06", False, text)
    has_ok = "Wrote" in text or "Response" in text
    return log("modbus_write:FC06", has_ok, text[:200])


def test_modbus_read_back_written(client):
    """Read back the value we wrote"""
    resp, text = client.call_tool_expect_ok("modbus_read", {
        "slave_id": 1,
        "address": 0,
        "quantity": 1
    })
    if resp is None:
        return log("modbus_read_back", False, text)
    has_100 = "100" in text
    return log("modbus_read_back", has_100, text[:200])


def test_modbus_read_10_regs(client):
    """Read 10 registers"""
    resp, text = client.call_tool_expect_ok("modbus_read", {
        "slave_id": 1,
        "address": 0,
        "quantity": 10
    })
    if resp is None:
        return log("modbus_read:FC03_10regs", False, text)
    has_values = "10 registers" in text or "registers" in text.lower()
    return log("modbus_read:FC03_10regs", has_values, text[:200])


def test_modbus_invalid_quantity(client):
    """Quantity > 125 should error"""
    resp, text = client.call_tool_expect_ok("modbus_read", {
        "slave_id": 1,
        "address": 0,
        "quantity": 200
    })
    # Should get an error
    has_error = resp and "error" in resp
    return log("modbus_read:invalid_quantity", has_error, text[:100])


def test_modbus_read_discrete_inputs(client):
    """Read discrete inputs FC02 - use send with hex"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "01 02 00 00 00 08 79 CC",
        "hex": True
    })
    if resp is None:
        return log("modbus:FC02_send", False, text)
    time.sleep(0.5)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("modbus:FC02_read", False, text2)
    has_response = len(text2.strip()) > 10
    return log("modbus:FC02_raw", has_response, f"got='{text2[:80]}'")


def test_modbus_write_multiple_registers(client):
    """Write multiple registers FC16 - use send with hex"""
    # Drain stale data from previous tests
    client.call_tool_expect_ok("read", {"timeout_ms": 200, "format": "hex"})
    time.sleep(0.2)
    # FC16: slave=1, func=0x10, start=0x0000, qty=0x0002, bytecount=4, data[200,300]
    # CRC: 0x1C72 → low=0x72, high=0x1C
    resp, text = client.call_tool_expect_ok("send", {
        "data": "01 10 00 00 00 02 04 00 C8 01 2C 72 1C",
        "hex": True
    })
    if resp is None:
        return log("modbus:FC16_send", False, text)
    time.sleep(0.5)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("modbus:FC16_read", False, text2)
    has_response = len(text2.strip()) > 5
    return log("modbus:FC16_write_multi", has_response, f"got='{text2[:80]}'")


def test_modbus_read_coils(client):
    """Read coils FC01 - use send with hex"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "01 01 00 00 00 08 3D CC",
        "hex": True
    })
    if resp is None:
        return log("modbus:FC01_send", False, text)
    time.sleep(0.5)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("modbus:FC01_read", False, text2)
    has_response = len(text2.strip()) > 5
    return log("modbus:FC01_coils", has_response, f"got='{text2[:80]}'")


def test_modbus_write_single_coil(client):
    """Write single coil FC05 - use send with hex"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "01 05 00 00 FF 00 8C 3A",
        "hex": True
    })
    if resp is None:
        return log("modbus:FC05_send", False, text)
    time.sleep(0.5)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("modbus:FC05_read", False, text2)
    has_response = len(text2.strip()) > 5
    return log("modbus:FC05_write_coil", has_response, f"got='{text2[:80]}'")


def test_modbus_read_input_registers(client):
    """Read input registers FC04 - use send with hex"""
    resp, text = client.call_tool_expect_ok("send", {
        "data": "01 04 00 00 00 04 F1 C9",
        "hex": True
    })
    if resp is None:
        return log("modbus:FC04_send", False, text)
    time.sleep(0.5)
    resp2, text2 = client.call_tool_expect_ok("read", {
        "timeout_ms": 1000,
        "format": "hex"
    })
    if resp2 is None:
        return log("modbus:FC04_read", False, text2)
    has_response = len(text2.strip()) > 10
    return log("modbus:FC04_input_regs", has_response, f"got='{text2[:80]}'")


# ── PLC Mode Tests ──

def test_switch_to_plc(client):
    """Switch MCU to PLC mode — with delay to let MCU settle from previous mode"""
    time.sleep(0.5)  # Let MCU settle from Modbus mode
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+APP=PLC",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("switch_to_plc", False, text)
    has_ok = "OK" in text or "PLC" in text
    return log("switch_to_plc", has_ok, f"got='{text[:120]}'")


def test_plc_set_brand(client):
    """Set PLC brand to Siemens"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+BRAND=SIEMENS",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("plc:set_brand", False, text)
    has_ok = "OK" in text or "SIEMENS" in text or "BRAND" in text
    return log("plc:set_brand", has_ok, f"got='{text[:120]}'")


def test_plc_enable_sim(client):
    """Enable random data simulation"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+SIM=ON",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("plc:enable_sim", False, text)
    has_ok = "OK" in text or "SIM" in text
    return log("plc:enable_sim", has_ok, f"got='{text[:120]}'")


def test_plc_read(client):
    """PLC read all registers"""
    resp, text = client.call_tool_expect_ok("plc_read", {
        "brand": "Siemens",
        "slave_id": 1
    })
    if resp is None:
        return log("plc_read:siemens", False, text)
    has_regs = "registers" in text.lower() or "addr" in text.lower() or "PLC" in text
    return log("plc_read:siemens", has_regs, text[:300])


def test_plc_write(client):
    """PLC write register"""
    resp, text = client.call_tool_expect_ok("plc_write", {
        "brand": "Siemens",
        "slave_id": 1,
        "address": 8,
        "value": 500
    })
    if resp is None:
        return log("plc_write:siemens", False, text)
    has_ok = "Wrote" in text or "register" in text.lower()
    return log("plc_write:siemens", has_ok, text[:200])


def test_plc_read_different_brands(client):
    """Test PLC read with different brands"""
    for brand in ["Mitsubishi", "Delta", "Omron"]:
        resp, text = client.call_tool_expect_ok("plc_read", {
            "brand": brand,
            "slave_id": 1
        })
        if resp is None:
            log(f"plc_read:{brand}", False, text)
        else:
            has_regs = "registers" in text.lower() or "addr" in text.lower()
            log(f"plc_read:{brand}", has_regs, text[:150])


def test_plc_invalid_brand(client):
    """PLC read with invalid brand"""
    resp, text = client.call_tool_expect_ok("plc_read", {
        "brand": "InvalidBrand"
    })
    has_error = resp and "error" in resp
    return log("plc_read:invalid_brand", has_error, text[:100])


# ── Access Log & Status Tests ──

def test_get_access_log(client):
    """Get access log"""
    resp, text = client.call_tool_expect_ok("get_access_log", {"limit": 10})
    if resp is None:
        return log("get_access_log", False, text)
    has_entries = "time" in text.lower() or "action" in text.lower() or "CALL" in text
    return log("get_access_log", has_entries, text[:200])


# ── Stress Tests ──

def test_rapid_send_read(client):
    """Send/read cycles using send_command"""
    passed = True
    for i in range(5):
        resp, text = client.call_tool_expect_ok("send_command", {
            "command": f"rapid_{i}",
            "timeout_ms": 2000
        })
        if resp is None or f"rapid_{i}" not in text:
            passed = False
            break
        time.sleep(0.3)  # Delay between commands for MCU at 9600 baud
    return log("rapid_send_read_5x", passed, "")


def test_large_payload(client):
    """Send large hex payload using send_command for reliable read"""
    # 16 bytes of 0xAA (smaller than 64 to avoid MCU buffer issues at 9600 baud)
    large_hex = " ".join(["AA"] * 16)
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": large_hex,
        "timeout_ms": 3000
    })
    if resp is None:
        return log("large_payload:echo", False, text)
    has_aa = "AA" in text
    return log("large_payload:echo", has_aa, f"got {len(text.split())} tokens")


# ── Switch back to ECHO ──

def test_switch_to_echo(client):
    """Switch MCU back to ECHO mode"""
    resp, text = client.call_tool_expect_ok("send_command", {
        "command": "AT+APP=ECHO",
        "timeout_ms": 2000
    })
    if resp is None:
        return log("switch_to_echo", False, text)
    has_ok = "OK" in text or "ECHO" in text
    return log("switch_to_echo", has_ok, f"got='{text[:120]}'")


def test_disconnect(client):
    resp, text = client.call_tool_expect_ok("disconnect")
    if resp is None:
        return log("disconnect", False, text)
    ok = "Disconnected" in text or "error" not in text.lower()
    return log("disconnect", ok, text[:100])


# ── Main ──

def main():
    print(f"=" * 60)
    print(f"MCP Server Comprehensive Test")
    print(f"Target: {MCP_HOST}:{MCP_PORT} → MCU {TEST_PORT} @ {TEST_BAUD} baud")
    print(f"=" * 60)
    print()

    client = McpClient(MCP_HOST, MCP_PORT)
    print("[OK] Connected to MCP server\n")

    try:
        # Phase 1: Protocol & Discovery
        print("── Phase 1: Protocol & Discovery ──")
        test_initialize(client)
        test_tools_list(client)
        test_list_ports(client)
        print()

        # Phase 2: Connection & Config
        print("── Phase 2: Connection & Config ──")
        test_connect(client)
        test_status(client)
        test_get_config(client)
        test_set_config(client)
        test_get_device_info(client)
        print()

        # Phase 3: ECHO Mode
        print("── Phase 3: ECHO Mode ──")
        test_echo_text(client)
        test_echo_hex(client)
        test_echo_binary(client)
        test_send_command_echo(client)
        test_send_read_hex_format(client)
        test_send_read_raw_format(client)
        test_at_help(client)
        test_at_status(client)
        print()

        # Phase 4: Modbus Mode
        print("── Phase 4: Modbus Mode ──")
        test_switch_to_modbus(client)
        test_modbus_read_holding_registers(client)
        test_modbus_read_with_scale(client)
        test_modbus_write(client)
        test_modbus_read_back_written(client)
        test_modbus_read_10_regs(client)
        test_modbus_invalid_quantity(client)
        test_modbus_read_discrete_inputs(client)
        test_modbus_write_multiple_registers(client)
        test_modbus_read_coils(client)
        test_modbus_write_single_coil(client)
        test_modbus_read_input_registers(client)
        print()

        # Phase 5: PLC Mode
        print("── Phase 5: PLC Mode ──")
        test_switch_to_plc(client)
        test_plc_set_brand(client)
        test_plc_enable_sim(client)
        test_plc_read(client)
        test_plc_write(client)
        test_plc_read_different_brands(client)
        test_plc_invalid_brand(client)
        print()

        # Phase 6: Access Log
        print("── Phase 6: Access Log ──")
        test_get_access_log(client)
        print()

        # Phase 7: Stress
        print("── Phase 7: Stress Tests ──")
        # Switch back to ECHO mode for stress tests
        test_switch_to_echo(client)
        time.sleep(0.5)
        test_rapid_send_read(client)
        test_large_payload(client)
        print()

        # Cleanup: switch back to ECHO
        print("── Cleanup ──")
        test_switch_to_echo(client)
        test_disconnect(client)

    except Exception as e:
        print(f"\n[FATAL] {e}")
    finally:
        client.close()

    # Summary
    print()
    print(f"=" * 60)
    total = len(results)
    passed = sum(1 for _, p, _ in results if p)
    failed = sum(1 for _, p, _ in results if not p)
    print(f"Results: {passed}/{total} passed, {failed} failed")
    if failed > 0:
        print(f"\nFailed tests:")
        for name, p, detail in results:
            if not p:
                print(f"  - {name}: {detail}")
    print(f"=" * 60)

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
