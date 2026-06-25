# SerialRUN Mobile

移动端版本，支持 Android USB OTG 串口调试。

## 平台支持

| 平台 | 状态 | 说明 |
|------|------|------|
| Android | 🚧 开发中 | USB OTG 串口，需 OTG 转接线 |
| iOS | ❌ 暂不支持 | 需要 MFi 认证，成本过高 |

## 快速开始

### 环境准备

```bash
# 安装 cargo-mobile2
cargo install cargo-mobile2

# 安装 Android SDK/NDK（通过 Android Studio）
```

### 编译 Android

```bash
cd mobile/android
cargo mobile2 build --release -p serialrun-mobile
```

### 安装到手机

```bash
adb install target/aarch64-linux-android/release/serialrun-mobile.apk
```

## 功能范围（MVP）

- ✅ 串口通信（HEX/TEXT）
- ✅ Modbus RTU/TCP 调试
- ✅ 波特率自动检测
- ⬜ PLC 控制（后续）
- ⬜ MCP AI 集成（后续）

## 硬件要求

- Android 6.0+ 手机
- USB OTG 转接线
- USB 转串口模块（CH340/CP2102/FT232）

## 架构

```
mobile/
├── android/          # Android 项目配置
├── README.md         # 本文件
└── PLAN.md           # 详细实施计划

crates/
├── serialrun-mobile/ # 移动端核心逻辑
└── serialrun-core/   # 共享核心（协议、校验、配置）
```
