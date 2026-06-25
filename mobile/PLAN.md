# SerialRUN Mobile 实施计划

## Phase 1: 项目结构搭建 ✅
- [x] 创建 `mobile/` 目录
- [x] 创建 `mobile/README.md`
- [x] 创建 `mobile/PLAN.md`
- [ ] 创建 `crates/serialrun-mobile/` crate
- [ ] 在 `serialrun-core` 中添加串口 trait 抽象

## Phase 2: Android USB 串口适配
- [x] Java 层：UsbSerialPort 封装（usb-serial-for-android）
- [x] Java 层：设备枚举 + 权限检查
- [x] Java 层：读写数据 + 回调
- [x] Rust 层：JNI 桥接（jni_bridge.rs）
- [x] Rust 层：MobileSerialPort 封装
- [x] 构建配置：build.rs + cargo config
- [ ] 端到端测试：OTG 连接串口设备（需真机）

## Phase 3: 移动端 UI 适配
- [ ] 响应式布局（触摸友好按钮）
- [ ] 简化功能集（串口 + Modbus + 终端）
- [ ] 移除桌面专属组件
- [ ] 底部导航栏

## Phase 4: 打包发布
- [ ] APK 签名配置
- [ ] 版本号管理
- [ ] Google Play 准备（可选）

## 技术决策记录

### 为什么不用 Flutter/React Native？
- SerialRUN 核心是 Rust，用 egui 可以复用大部分代码
- Flutter 需要用 Dart 重写串口逻辑
- React Native 性能不适合实时串口通信

### 为什么不用 Tauri Mobile？
- Tauri Mobile 需要 WebView，性能不如原生 egui
- 串口通信需要低延迟，WebView 不适合

### 串口抽象方案
在 `serialrun-core` 中定义 `SerialPortImpl` trait，桌面用 `serialport` crate 实现，Android 用 JNI 调用 `usb-serial-for-android` 实现。
