# Forge Final Report: MCP Server Comprehensive Test & Fix

## Summary
- **Task**: Test all MCP tools against Multi-Sim v3.0 MCU (COM4 @ 9600 baud) and fix bugs
- **Result**: 43/43 tests pass (0 failures)
- **MCU Modes Tested**: ECHO, Modbus RTU, PLC (Siemens/Mitsubishi/Delta/Omron)

## Bugs Found & Fixed

### Bug 1: MCP Read drains GUI's rx_buffer (HIGH)
**Problem**: MCP `Read`/`SendRead` and GUI terminal both read from the same `rx_buffer`. Whichever reads first consumes the data exclusively — data loss for the other.
**Fix**: Added dedicated `mcp_rx_buffer` in `PortOwnerHandle`. Continuous reader writes to both buffers. MCP reads from its own buffer.
**Files**: `port_owner.rs`, `app.rs`

### Bug 2: SendRead doesn't use exclusive read (HIGH)
**Problem**: MCP `send_command` used `SendRead` which sent `Write` + polled `rx_buffer`. When `pause_after=true`, the continuous reader was paused and the MCP buffer was never populated.
**Fix**: Changed `SendRead` to use `write_read_exclusive()` which pauses the continuous reader, writes, reads directly from serial, and resumes.
**Files**: `app.rs`

### Bug 3: Read handler fallback to direct serial read (MEDIUM)
**Problem**: When continuous reader is paused (e.g., after CAN/Scope exclusive access), `Read` handler reads empty MCP buffer with no fallback.
**Fix**: Added fallback: if MCP buffer is empty after initial poll, send `ReadWait` command to read directly from serial port.
**Files**: `app.rs`

### Bug 4: WriteAndPause doesn't clear MCP buffer (LOW)
**Problem**: `WriteAndPause` clears serial input buffer but not MCP buffer, causing stale data to persist.
**Fix**: Added `mcp_rx_buffer.clear()` in `WriteAndPause` and `ReadExclusive` handlers.
**Files**: `port_owner.rs`

## Test Coverage

| Phase | Tests | Status |
|-------|-------|--------|
| Protocol & Discovery | 3 | ALL PASS |
| Connection & Config | 5 | ALL PASS |
| ECHO Mode (text/hex/binary/AT) | 8 | ALL PASS |
| Modbus RTU (FC01-FC06, FC16) | 12 | ALL PASS |
| PLC (Siemens/Mitsubishi/Delta/Omron) | 8 | ALL PASS |
| Access Log | 1 | ALL PASS |
| Stress (rapid send/read, large payload) | 3 | ALL PASS |
| Cleanup (mode switch, disconnect) | 3 | ALL PASS |
| **Total** | **43** | **ALL PASS** |

## MCP Tools Verified Working
- `list_ports` — finds COM4
- `connect` / `disconnect` — port lifecycle
- `send` — text and hex data
- `read` — hex, text, base64 formats
- `send_command` — write-read with response
- `modbus_read` — FC03 with engineering conversion
- `modbus_write` — FC06 single register
- `plc_read` — all 4 brands with data types
- `plc_write` — register write
- `get_config` / `set_config` — settings management
- `status` / `get_device_info` / `get_access_log` — monitoring
- Raw Modbus frames: FC01, FC02, FC04, FC05, FC16

## Files Changed (not in git — GUI crate is untracked)
- `crates/serialrun-gui/src/port_owner.rs` — MCP buffer, WriteAndPause clear
- `crates/serialrun-gui/src/app.rs` — MCP Read/SendRead handlers
- `tests/mcp_comprehensive_test.py` — comprehensive test script (new)
