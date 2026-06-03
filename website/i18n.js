const translations = {
    en: {
        nav_features: "Features",
        nav_download: "Download",
        nav_coffee: "Coffee",
        nav_community: "Community",
        nav_guide: "Guide",
        hero_sub: "Professional Serial Port Debugging Assistant",
        hero_desc: "All-in-one tool for serial communication, Modbus debugging, PLC control, CAN bus analysis, I2C/SPI, firmware flashing — with built-in MCP server for AI integration.",
        hero_dl: "Download",
        hero_github: "View Source Code",
        notice_banner: "Early release. Some features are not fully tested yet. Feedback and feature requests are welcome on <a href=\"https://github.com/YaoIsAI/SerialRUN/issues\" target=\"_blank\">GitHub Issues</a>!",
        feat_title: "Everything you need for serial debugging",
        feat_desc: "From simple AT command testing to complex Modbus/PLC protocol analysis",
        f1_title: "Serial Communication",
        f1_desc: "TX/RX with HEX/TEXT mode, timestamps, CRC checksums, auto-send, DTR/RTS control. Supports 300-921600 baud rates.",
        f2_title: "Modbus Debug",
        f2_desc: "Quick register read/write, 8 function codes, configurable response timeout, register monitoring with auto-polling.",
        f3_title: "PLC Control",
        f3_desc: "Siemens, Mitsubishi, Delta, Omron presets. Batch register read, inline editing, real-time data freshness indicators.",
        f4_title: "TCP/RTU Bridge",
        f4_desc: "Bridge Modbus TCP clients to serial RTU devices. Connect SCADA/HMI software to your serial devices over network.",
        f5_title: "MCP Server",
        f5_desc: "Built-in Model Context Protocol server. Let AI assistants control your serial devices via TCP. 15 tools available.",
        f6_title: "I2C/SPI & CAN",
        f6_desc: "Scan and read/write I2C/SPI devices. CAN bus frame capture and analysis with SLCAN protocol support.",
        f7_title: "Data Visualization",
        f7_desc: "Real-time data rate charts, oscilloscope waveform display, data logging to CSV with timestamps.",
        f8_title: "Firmware Flash",
        f8_desc: "STM32 ISP and ESP32 serial flashing. XMODEM/YMODEM/ZMODEM file transfer with progress tracking.",
        ai_badge: "AI-Powered",
        ai_title: "MCP Server for AI Integration",
        ai_desc: "SerialRUN includes a built-in MCP (Model Context Protocol) server that allows AI assistants to remotely control your serial devices.",
        ai_f1: "15 built-in tools: connect, send, read, modbus_read, modbus_write, plc_read, plc_write, and more",
        ai_f2: "Real-time TX/RX monitoring by AI assistants",
        ai_f3: "Support for local and LAN connections",
        ai_f4: "Full access logging with client IP tracking",
        diagram_ai: "AI Assistant",
        diagram_dev: "MCU Device",
        mw_title: "Multi-Window Interface",
        mw_desc: "All panels run as independent OS windows — drag, resize, and arrange freely",
        mw1_title: "Independent Windows",
        mw1_desc: "Each panel (Modbus, PLC, Log, etc.) is a separate OS window that can be moved anywhere on screen.",
        mw2_title: "Always On Top",
        mw2_desc: "Protocol panels stay above the main window — never lose your tools behind the terminal.",
        mw3_title: "Auto-Wrapping Toolbar",
        mw3_desc: "Terminal toolbar adapts to window size — controls wrap automatically, nothing gets clipped.",
        dl_title: "Download SerialRUN",
        dl_desc: "Source available under BSL 1.1. Build from source for any platform.",
        dl_win: "Windows 10/11 (x64)",
        dl_win_btn: "Download .zip",
        dl_mac: "macOS 12+ (Apple Silicon)",
        dl_mac_btn: "Download .zip",
        dl_linux: "x86_64 / aarch64",
        dl_build: "Build from Source",
        macos_note: "<strong>macOS:</strong> On first launch, go to <strong>System Settings → Privacy & Security</strong> and click \"Open Anyway\". Tested on MacBook Air M1.",
        build_title: "Build from source:",
        build_note: 'Requires Rust toolchain. See <a href="https://github.com/YaoIsAI/SerialRUN/blob/master/docs/BUILD.md" target="_blank">BUILD.md</a> for details.',
        footer_tagline: "Professional Serial Port Debugging Assistant",
        footer_guide: "User Guide",
        footer_license: "License",
        community_issues_title: "Feature Requests & Bug Reports",
        community_issues_desc: "Have an idea or found a bug? Open an issue on GitHub and let's make SerialRUN better together.",
        community_issues_btn: "Open Issue",
        community_discuss_title: "Discussion & Feedback",
        community_discuss_desc: "Questions, suggestions, or just want to say hi? Start a discussion on GitHub.",
        community_discuss_btn: "Join Discussion",
        coffee_title: "Buy the Author a Coffee",
        coffee_desc: "If SerialRUN helps your work, consider buying me a coffee.",
    },
    zh: {
        nav_features: "功能特性",
        nav_download: "下载",
        nav_coffee: "请喝咖啡",
        nav_community: "社区",
        nav_guide: "使用指南",
        hero_sub: "专业串口调试助手",
        hero_desc: "一站式串口调试工具，支持 Modbus 调试、PLC 控制、CAN 总线分析、I2C/SPI、固件烧录——内置 MCP 服务器，支持 AI 助手集成。",
        hero_dl: "下载",
        hero_github: "查看源码",
        notice_banner: "早期版本，部分功能尚未完整测试。欢迎在 <a href=\"https://github.com/YaoIsAI/SerialRUN/issues\" target=\"_blank\">GitHub Issues</a> 提交反馈和需求！",
        feat_title: "你需要的一切串口调试功能",
        feat_desc: "从简单的 AT 命令测试到复杂的 Modbus/PLC 协议分析",
        f1_title: "串口通信",
        f1_desc: "TX/RX 支持 HEX/TEXT 模式、时间戳、CRC 校验、自动发送、DTR/RTS 控制。支持 300-921600 波特率。",
        f2_title: "Modbus 调试",
        f2_desc: "快速读写寄存器，8 种功能码，可配置响应超时，寄存器监控与自动轮询。",
        f3_title: "PLC 控制",
        f3_desc: "西门子、三菱、台达、欧姆龙品牌预设。批量读取、内联编辑、实时数据新鲜度指示。",
        f4_title: "TCP/RTU 桥接",
        f4_desc: "将 Modbus TCP 客户端桥接到串口 RTU 设备。通过网络连接 SCADA/HMI 软件到串口设备。",
        f5_title: "MCP 服务器",
        f5_desc: "内置 MCP（模型上下文协议）服务器，让 AI 助手通过 TCP 远程控制串口设备。15 个内置工具。",
        f6_title: "I2C/SPI 与 CAN",
        f6_desc: "扫描和读写 I2C/SPI 设备。CAN 总线帧捕获与解析，支持 SLCAN 协议。",
        f7_title: "数据可视化",
        f7_desc: "实时数据速率图表、示波器波形显示、带时间戳的 CSV 数据记录。",
        f8_title: "固件烧录",
        f8_desc: "STM32 ISP 和 ESP32 串口烧录。XMODEM/YMODEM/ZMODEM 文件传输，带进度追踪。",
        ai_badge: "AI 驱动",
        ai_title: "MCP 服务器 — AI 集成",
        ai_desc: "SerialRUN 内置 MCP（模型上下文协议）服务器，允许 AI 助手通过 TCP 远程控制串口设备。",
        ai_f1: "15 个内置工具：connect、send、read、modbus_read、modbus_write、plc_read、plc_write 等",
        ai_f2: "AI 助手实时监控 TX/RX 数据",
        ai_f3: "支持本机和局域网连接",
        ai_f4: "完整的访问日志，记录客户端 IP",
        diagram_ai: "AI 助手",
        diagram_dev: "MCU 设备",
        mw_title: "多窗口界面",
        mw_desc: "所有面板作为独立 OS 窗口运行——自由拖拽、缩放和排列",
        mw1_title: "独立窗口",
        mw1_desc: "每个面板（Modbus、PLC、日志等）都是独立的 OS 窗口，可以移动到屏幕任意位置。",
        mw2_title: "始终在前",
        mw2_desc: "协议面板始终在主窗口前面——永远不会被终端遮挡。",
        mw3_title: "自动换行工具栏",
        mw3_desc: "终端工具栏自适应窗口大小——控件自动换行，不会被裁剪。",
        dl_title: "下载 SerialRUN",
        dl_desc: "BSL 1.1 源码可用。可从源码编译，支持全平台。",
        dl_win: "Windows 10/11 (x64)",
        dl_win_btn: "下载 .zip",
        dl_mac: "macOS 12+ (Apple Silicon)",
        dl_mac_btn: "下载 .zip",
        dl_linux: "x86_64 / aarch64",
        dl_build: "从源码编译",
        macos_note: "<strong>macOS:</strong> 首次运行需在 <strong>系统设置 → 隐私与安全性</strong> 中点击「仍要打开」。已测试于 MacBook Air M1。",
        build_title: "从源码编译：",
        build_note: '需要 Rust 工具链。详见 <a href="https://github.com/YaoIsAI/SerialRUN/blob/master/docs/BUILD_CN.md" target="_blank">BUILD_CN.md</a>。',
        footer_tagline: "专业串口调试助手",
        footer_guide: "使用指南",
        footer_license: "许可证",
        community_issues_title: "功能需求与 Bug 反馈",
        community_issues_desc: "有新想法或发现了 Bug？在 GitHub 上提交 Issue，一起让 SerialRUN 变得更好。",
        community_issues_btn: "提交 Issue",
        community_discuss_title: "讨论与建议",
        community_discuss_desc: "有问题、建议，或者只是想打个招呼？在 GitHub 上开启讨论。",
        community_discuss_btn: "参与讨论",
        coffee_title: "请作者喝杯咖啡",
        coffee_desc: "如果 SerialRUN 对你的工作有帮助，请考虑请作者喝杯咖啡。你的支持是项目持续发展的动力。",
    }
};

let currentLang = localStorage.getItem('lang') || 'en';

function applyLang(lang) {
    currentLang = lang;
    localStorage.setItem('lang', lang);
    document.documentElement.setAttribute('data-lang', lang);
    document.documentElement.setAttribute('lang', lang === 'zh' ? 'zh-CN' : 'en');

    // Toggle text elements
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (translations[lang][key]) {
            if (el.tagName === 'INPUT' && el.hasAttribute('placeholder')) {
                el.placeholder = translations[lang][key];
            } else {
                el.innerHTML = translations[lang][key];
            }
        }
    });

    // Toggle language-specific elements
    document.querySelectorAll('.lang-en').forEach(el => {
        el.style.display = lang === 'en' ? '' : 'none';
    });
    document.querySelectorAll('.lang-zh').forEach(el => {
        el.style.display = lang === 'zh' ? '' : 'none';
    });

    // Toggle screenshots
    const imgEn = document.getElementById('hero_screenshot_en');
    const imgZh = document.getElementById('hero_screenshot_zh');
    if (imgEn) imgEn.style.display = lang === 'en' ? '' : 'none';
    if (imgZh) imgZh.style.display = lang === 'zh' ? '' : 'none';

    // Update page title
    document.title = lang === 'zh'
        ? 'SerialRUN - 专业串口调试助手'
        : 'SerialRUN - Professional Serial Port Debugging Assistant';
}

function toggleLang() {
    applyLang(currentLang === 'en' ? 'zh' : 'en');
}

// Initialize on load
document.addEventListener('DOMContentLoaded', () => {
    applyLang(currentLang);
});
