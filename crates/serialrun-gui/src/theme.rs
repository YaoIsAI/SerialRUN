use eframe::egui;

/// Centralized theme colors for Dark and Light modes.
/// All UI components should reference these instead of hardcoded colors.
#[derive(Clone)]
pub struct ThemeColors {
    // ── Base ──
    pub bg_primary: egui::Color32,
    pub bg_secondary: egui::Color32,
    pub bg_panel: egui::Color32,

    // ── Text ──
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_muted: egui::Color32,
    pub text_strong: egui::Color32,

    // ── Status ──
    pub success: egui::Color32,      // Connected, OK, green
    pub error: egui::Color32,        // Errors, disconnect, red
    pub warning: egui::Color32,      // Warnings, yellow
    pub info: egui::Color32,         // Info links, blue
    pub logo_green: egui::Color32,   // Brand color

    // ── Terminal ──
    pub tx_color: egui::Color32,     // TX direction
    pub rx_color: egui::Color32,     // RX direction
    pub sys_color: egui::Color32,    // SYS direction
    pub timestamp_color: egui::Color32,
    pub tx_badge_bg: egui::Color32,  // TX badge background
    pub rx_badge_bg: egui::Color32,  // RX badge background
    pub sys_badge_bg: egui::Color32, // SYS badge background

    // ── Buttons ──
    pub btn_send: egui::Color32,
    pub btn_send_text: egui::Color32,
    pub btn_danger: egui::Color32,
    pub btn_danger_text: egui::Color32,
    pub btn_success: egui::Color32,
    pub btn_success_text: egui::Color32,
    pub btn_mcp_copy: egui::Color32,
    pub btn_mcp_copied: egui::Color32,

    // ── UI Elements ──
    pub separator: egui::Color32,
    pub border: egui::Color32,
    pub hover_bg: egui::Color32,
    pub scrollbar: egui::Color32,
    pub code_bg: egui::Color32,
    pub code_text: egui::Color32,

    // ── Log Levels ──
    pub log_info: egui::Color32,
    pub log_warning: egui::Color32,
    pub log_error: egui::Color32,

    // ── MCP ──
    pub mcp_connect: egui::Color32,
    pub mcp_disconnect: egui::Color32,
    pub mcp_call: egui::Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            bg_primary: egui::Color32::from_gray(30),
            bg_secondary: egui::Color32::from_gray(40),
            bg_panel: egui::Color32::from_gray(35),

            text_primary: egui::Color32::WHITE,
            text_secondary: egui::Color32::from_rgb(200, 200, 200),
            text_muted: egui::Color32::from_rgb(150, 150, 150),
            text_strong: egui::Color32::WHITE,

            success: egui::Color32::from_rgb(80, 200, 120),
            error: egui::Color32::from_rgb(240, 80, 80),
            warning: egui::Color32::from_rgb(240, 200, 60),
            info: egui::Color32::from_rgb(100, 160, 240),
            logo_green: egui::Color32::from_rgb(76, 175, 80),

            tx_color: egui::Color32::from_rgb(100, 180, 255),
            rx_color: egui::Color32::from_rgb(80, 220, 140),
            sys_color: egui::Color32::from_rgb(200, 180, 80),
            timestamp_color: egui::Color32::from_rgb(150, 150, 150),
            tx_badge_bg: egui::Color32::from_rgb(20, 50, 80),
            rx_badge_bg: egui::Color32::from_rgb(20, 60, 30),
            sys_badge_bg: egui::Color32::from_rgb(55, 55, 55),

            btn_send: egui::Color32::from_rgb(40, 160, 80),
            btn_send_text: egui::Color32::WHITE,
            btn_danger: egui::Color32::from_rgb(200, 60, 60),
            btn_danger_text: egui::Color32::WHITE,
            btn_success: egui::Color32::from_rgb(40, 160, 80),
            btn_success_text: egui::Color32::WHITE,
            btn_mcp_copy: egui::Color32::from_rgb(50, 120, 210),
            btn_mcp_copied: egui::Color32::from_rgb(60, 180, 80),

            separator: egui::Color32::from_rgb(60, 60, 60),
            border: egui::Color32::from_rgb(80, 80, 80),
            hover_bg: egui::Color32::from_rgb(50, 50, 50),
            scrollbar: egui::Color32::from_rgb(80, 80, 80),
            code_bg: egui::Color32::from_rgb(30, 30, 30),
            code_text: egui::Color32::from_rgb(200, 200, 200),

            log_info: egui::Color32::WHITE,
            log_warning: egui::Color32::YELLOW,
            log_error: egui::Color32::from_rgb(255, 80, 80),

            mcp_connect: egui::Color32::from_rgb(80, 200, 120),
            mcp_disconnect: egui::Color32::from_rgb(240, 80, 80),
            mcp_call: egui::Color32::from_rgb(100, 160, 240),
        }
    }

    pub fn light() -> Self {
        Self {
            bg_primary: egui::Color32::from_rgb(245, 245, 248),
            bg_secondary: egui::Color32::from_rgb(235, 235, 240),
            bg_panel: egui::Color32::from_rgb(240, 240, 245),

            text_primary: egui::Color32::from_rgb(20, 20, 20),
            text_secondary: egui::Color32::from_rgb(60, 60, 60),
            text_muted: egui::Color32::from_rgb(120, 120, 120),
            text_strong: egui::Color32::from_rgb(0, 0, 0),

            success: egui::Color32::from_rgb(0, 140, 60),
            error: egui::Color32::from_rgb(200, 40, 40),
            warning: egui::Color32::from_rgb(180, 120, 0),
            info: egui::Color32::from_rgb(30, 100, 200),
            logo_green: egui::Color32::from_rgb(56, 142, 60),

            tx_color: egui::Color32::from_rgb(20, 100, 200),
            rx_color: egui::Color32::from_rgb(0, 140, 60),
            sys_color: egui::Color32::from_rgb(160, 120, 0),
            timestamp_color: egui::Color32::from_rgb(100, 100, 100),
            tx_badge_bg: egui::Color32::from_rgb(200, 220, 255),
            rx_badge_bg: egui::Color32::from_rgb(200, 240, 210),
            sys_badge_bg: egui::Color32::from_rgb(230, 230, 220),

            btn_send: egui::Color32::from_rgb(30, 140, 60),
            btn_send_text: egui::Color32::WHITE,
            btn_danger: egui::Color32::from_rgb(200, 40, 40),
            btn_danger_text: egui::Color32::WHITE,
            btn_success: egui::Color32::from_rgb(30, 140, 60),
            btn_success_text: egui::Color32::WHITE,
            btn_mcp_copy: egui::Color32::from_rgb(30, 100, 200),
            btn_mcp_copied: egui::Color32::from_rgb(0, 140, 60),

            separator: egui::Color32::from_rgb(200, 200, 205),
            border: egui::Color32::from_rgb(180, 180, 185),
            hover_bg: egui::Color32::from_rgb(220, 220, 225),
            scrollbar: egui::Color32::from_rgb(180, 180, 185),
            code_bg: egui::Color32::from_rgb(230, 230, 235),
            code_text: egui::Color32::from_rgb(40, 40, 40),

            log_info: egui::Color32::from_rgb(20, 20, 20),
            log_warning: egui::Color32::from_rgb(180, 120, 0),
            log_error: egui::Color32::from_rgb(200, 40, 40),

            mcp_connect: egui::Color32::from_rgb(0, 140, 60),
            mcp_disconnect: egui::Color32::from_rgb(200, 40, 40),
            mcp_call: egui::Color32::from_rgb(30, 100, 200),
        }
    }
}

/// Get the current theme colors based on the app theme.
pub fn get_colors(theme: crate::state::Theme) -> ThemeColors {
    match theme {
        crate::state::Theme::Dark => ThemeColors::dark(),
        crate::state::Theme::Light => ThemeColors::light(),
    }
}
