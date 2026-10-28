use crate::state::{AppState, Language};
use crate::theme;
use crate::update::{
    self, current_version, UpdatePhase,
};
use eframe::egui;

fn is_zh(state: &AppState) -> bool {
    state.language == Language::Chinese
}

fn tr(state: &AppState, en: &'static str, zh: &'static str) -> &'static str {
    if is_zh(state) { zh } else { en }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn format_speed(bytes_per_second: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_second.max(0.0) as u64))
}

fn format_date(raw: &str) -> String {
    raw.get(..10).unwrap_or(raw).to_string()
}

/// Compact entry shown in the settings side panel.
pub fn render_settings_section(ui: &mut egui::Ui, state: &mut AppState) {
    let c = theme::get_colors(state.theme);
    ui.add_space(2.0);
    ui.label(egui::RichText::new(tr(state, "Software Update", "软件更新")).strong());
    ui.add_space(3.0);

    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "{}: v{}",
                tr(state, "Current version", "当前版本"),
                current_version()
            ))
            .color(c.text_secondary)
            .size(12.0),
        );
    });

    let busy = matches!(
        &state.update.phase,
        UpdatePhase::Checking
            | UpdatePhase::Downloading { .. }
            | UpdatePhase::Installing { .. }
    );
    let button_text = match &state.update.phase {
        UpdatePhase::Checking => tr(state, "Checking...", "正在检查...").to_string(),
        UpdatePhase::Downloading { .. } => tr(state, "Downloading...", "正在下载...").to_string(),
        UpdatePhase::Installing { .. } => tr(state, "Installing...", "正在安装...").to_string(),
        UpdatePhase::Available(candidate) => {
            format!("{} v{}", tr(state, "Update to", "升级到"), candidate.release.version)
        }
        UpdatePhase::Ready { .. } => tr(state, "Restart to Update", "重启并更新").to_string(),
        _ => tr(state, "Check for Updates", "检查更新").to_string(),
    };

    if ui
        .add_enabled(!busy, egui::Button::new(button_text).min_size(egui::vec2(0.0, 24.0)))
        .clicked()
    {
        match &state.update.phase {
            UpdatePhase::Available(_) | UpdatePhase::Ready { .. } => {
                state.update.show_dialog = true;
            }
            _ => update::start_check(state, true),
        }
    }

    let auto_label = tr(state, "Check on startup", "启动时自动检查");
    ui.checkbox(&mut state.auto_update_check, auto_label);

    match &state.update.phase {
        UpdatePhase::Downloading {
            downloaded, total, ..
        } => {
            let pct = if *total > 0 {
                *downloaded as f32 / *total as f32
            } else {
                0.0
            };
            ui.add(
                egui::ProgressBar::new(pct)
                    .text(format!("{:.0}%", pct * 100.0))
                    .desired_height(10.0),
            );
        }
        UpdatePhase::Available(candidate) => {
            if state.dismissed_update_version != candidate.release.version {
                ui.label(
                    egui::RichText::new(format!(
                        "{} v{}",
                        tr(state, "Available:", "可升级:"),
                        candidate.release.version
                    ))
                    .color(c.success)
                    .size(12.0),
                );
            }
        }
        UpdatePhase::Ready { version, .. } => {
            ui.label(
                egui::RichText::new(format!(
                    "v{} {}",
                    version,
                    tr(state, "ready to install", "已下载，等待安装")
                ))
                .color(c.success)
                .size(12.0),
            );
        }
        UpdatePhase::Error(error) => {
            let short: String = error.chars().take(60).collect();
            ui.label(egui::RichText::new(short).color(c.error).size(11.0));
        }
        UpdatePhase::UpToDate => {
            ui.label(
                egui::RichText::new(tr(state, "Up to date", "已是最新版本"))
                    .color(c.success)
                    .size(12.0),
            );
        }
        _ => {}
    }

    if state.last_update_check > 0 {
        let ts = chrono::DateTime::from_timestamp_millis(state.last_update_check)
            .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        ui.label(
            egui::RichText::new(format!("{}: {}", tr(state, "Last checked", "上次检查"), ts))
                .color(c.text_muted)
                .size(11.0),
        );
    }
}

/// Update badge shown in the status bar.
pub fn render_status_indicator(ui: &mut egui::Ui, state: &mut AppState) {
    let c = theme::get_colors(state.theme);
    match &state.update.phase {
        UpdatePhase::Available(candidate) => {
            if state.dismissed_update_version == candidate.release.version {
                return;
            }
            let text = if is_zh(state) {
                format!("\u{2B06} 升级到 v{}", candidate.release.version)
            } else {
                format!("\u{2B06} Update to v{}", candidate.release.version)
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(text)
                            .color(egui::Color32::BLACK)
                            .strong()
                            .size(12.0),
                    )
                    .fill(egui::Color32::from_rgb(250, 200, 60))
                    .rounding(4.0),
                )
                .on_hover_text(tr(state, "Open the update dialog", "打开更新窗口"))
                .clicked()
            {
                state.update.show_dialog = true;
            }
            ui.separator();
        }
        UpdatePhase::Downloading {
            downloaded, total, ..
        } => {
            let pct = if *total > 0 {
                *downloaded as f32 / *total as f32 * 100.0
            } else {
                0.0
            };
            ui.label(
                egui::RichText::new(format!(
                    "{} {:.0}%",
                    tr(state, "Updating", "正在升级"),
                    pct
                ))
                .color(c.info)
                .size(12.0),
            );
            ui.separator();
        }
        UpdatePhase::Ready { version, .. } => {
            let text = if is_zh(state) {
                format!("\u{2B06} 重启以更新到 v{}", version)
            } else {
                format!("\u{2B06} Restart to update to v{}", version)
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(text)
                            .color(egui::Color32::BLACK)
                            .strong()
                            .size(12.0),
                    )
                    .fill(egui::Color32::from_rgb(120, 210, 120))
                    .rounding(4.0),
                )
                .clicked()
            {
                state.update.show_dialog = true;
            }
            ui.separator();
        }
        UpdatePhase::Installing { .. } => {
            ui.label(
                egui::RichText::new(tr(state, "Installing update...", "正在安装更新..."))
                    .color(c.info)
                    .size(12.0),
            );
            ui.separator();
        }
        _ => {}
    }
}

enum DialogAction {
    None,
    Check,
    Download,
    Cancel,
    Install,
    ReleasePage,
    Reveal(std::path::PathBuf),
    Close,
}

pub fn render_update_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.update.show_dialog {
        return;
    }
    let c = theme::get_colors(state.theme);
    let lang = state.language;
    let zh = lang == Language::Chinese;
    let phase = state.update.phase.clone();
    let mut open = true;
    let mut action = DialogAction::None;

    egui::Window::new(if zh { "软件更新" } else { "Software Update" })
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(480.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(460.0);
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("SerialRUN")
                        .size(18.0)
                        .strong()
                        .color(c.text_primary),
                );
                ui.label(
                    egui::RichText::new(format!("v{}", current_version()))
                        .size(13.0)
                        .color(c.text_muted),
                );
            });
            ui.add_space(8.0);

            match &phase {
                UpdatePhase::Idle => {
                    ui.label(
                        egui::RichText::new(if zh {
                            "检查 GitHub 上是否有新版本。"
                        } else {
                            "Check GitHub for a newer version of SerialRUN."
                        })
                        .color(c.text_secondary),
                    );
                    ui.add_space(10.0);
                    if primary_button(ui, if zh { "检查更新" } else { "Check for Updates" }, c.success)
                        .clicked()
                    {
                        action = DialogAction::Check;
                    }
                }
                UpdatePhase::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(if zh {
                                "正在连接 GitHub 检查更新..."
                            } else {
                                "Checking GitHub for updates..."
                            })
                            .color(c.text_secondary),
                        );
                    });
                }
                UpdatePhase::UpToDate => {
                    ui.label(
                        egui::RichText::new(if zh {
                            format!("\u{2714} 已是最新版本 (v{})", current_version())
                        } else {
                            format!("\u{2714} You're on the latest version (v{})", current_version())
                        })
                        .size(14.0)
                        .color(c.success)
                        .strong(),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(if zh { "重新检查" } else { "Check Again" })
                            .clicked()
                        {
                            action = DialogAction::Check;
                        }
                        if ui.button(if zh { "关闭" } else { "Close" }).clicked() {
                            action = DialogAction::Close;
                        }
                    });
                }
                UpdatePhase::Available(candidate) => {
                    ui.label(
                        egui::RichText::new(if zh {
                            format!("发现新版本 v{}", candidate.release.version)
                        } else {
                            format!("SerialRUN v{} is available", candidate.release.version)
                        })
                        .size(16.0)
                        .strong()
                        .color(c.success),
                    );
                    ui.add_space(2.0);
                    let mut meta = format!(
                        "{} v{}  \u{2192}  v{}",
                        if zh { "当前版本" } else { "Current" },
                        current_version(),
                        candidate.release.version
                    );
                    if !candidate.release.published_at.is_empty() {
                        meta.push_str(&format!(
                            "   \u{00B7}   {}",
                            format_date(&candidate.release.published_at)
                        ));
                    }
                    ui.label(egui::RichText::new(meta).color(c.text_muted).size(12.0));

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(if zh { "更新说明" } else { "Release Notes" })
                            .strong()
                            .color(c.text_primary),
                    );
                    ui.add_space(2.0);
                    let notes = if candidate.release.body.trim().is_empty() {
                        if zh {
                            "该版本没有提供更新说明。"
                        } else {
                            "No release notes were provided for this version."
                        }
                    } else {
                        candidate.release.body.trim()
                    };
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(notes)
                                    .monospace()
                                    .size(12.0)
                                    .color(c.text_secondary),
                            );
                        });
                    ui.add_space(6.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(&candidate.asset.name)
                                .monospace()
                                .size(11.0)
                                .color(c.text_muted),
                        );
                        if candidate.asset.size > 0 {
                            ui.label(
                                egui::RichText::new(format!(
                                    "\u{00B7} {}",
                                    format_bytes(candidate.asset.size)
                                ))
                                .size(11.0)
                                .color(c.text_muted),
                            );
                        }
                        if candidate.checksum_asset.is_some() {
                            ui.label(
                                egui::RichText::new(if zh {
                                    "\u{00B7} SHA-256 校验可用"
                                } else {
                                    "\u{00B7} SHA-256 verified"
                                })
                                .size(11.0)
                                .color(c.success),
                            );
                        }
                    });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if primary_button(
                            ui,
                            if zh { "下载并安装" } else { "Download & Install" },
                            c.success,
                        )
                        .clicked()
                        {
                            action = DialogAction::Download;
                        }
                        if ui.button(if zh { "稍后" } else { "Later" }).clicked() {
                            action = DialogAction::Close;
                        }
                        if ui
                            .button(if zh { "打开发布页" } else { "Release Page" })
                            .clicked()
                        {
                            action = DialogAction::ReleasePage;
                        }
                    });
                }
                UpdatePhase::Downloading {
                    version,
                    asset_name,
                    downloaded,
                    total,
                    speed_bps,
                } => {
                    ui.label(
                        egui::RichText::new(if zh {
                            format!("正在下载 v{}...", version)
                        } else {
                            format!("Downloading v{}...", version)
                        })
                        .size(14.0)
                        .strong()
                        .color(c.text_primary),
                    );
                    ui.add_space(6.0);
                    let pct = if *total > 0 {
                        *downloaded as f32 / *total as f32
                    } else {
                        0.0
                    };
                    ui.add(
                        egui::ProgressBar::new(pct)
                            .text(format!("{:.1}%", pct * 100.0))
                            .desired_height(18.0),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let size_text = if *total > 0 {
                            format!(
                                "{} / {}",
                                format_bytes(*downloaded),
                                format_bytes(*total)
                            )
                        } else {
                            format_bytes(*downloaded)
                        };
                        ui.label(
                            egui::RichText::new(size_text)
                                .size(12.0)
                                .color(c.text_secondary),
                        );
                        if *speed_bps > 0.0 {
                            ui.label(
                                egui::RichText::new(format_speed(*speed_bps))
                                    .size(12.0)
                                    .color(c.text_muted),
                            );
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(if zh { "取消" } else { "Cancel" }).clicked() {
                                action = DialogAction::Cancel;
                            }
                        });
                    });
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(asset_name)
                            .monospace()
                            .size(10.0)
                            .color(c.text_muted),
                    );
                }
                UpdatePhase::Ready {
                    version,
                    asset_name,
                    path,
                    checksum_verified,
                } => {
                    ui.label(
                        egui::RichText::new(if zh {
                            format!("\u{2714} v{} 已下载完成", version)
                        } else {
                            format!("\u{2714} v{} has been downloaded", version)
                        })
                        .size(15.0)
                        .strong()
                        .color(c.success),
                    );
                    ui.add_space(4.0);
                    if *checksum_verified {
                        ui.label(
                            egui::RichText::new(if zh {
                                "SHA-256 校验通过"
                            } else {
                                "SHA-256 checksum verified"
                            })
                            .size(12.0)
                            .color(c.success),
                        );
                    }
                    ui.label(
                        egui::RichText::new(if zh {
                            "点击“立即安装并重启”后，SerialRUN 会关闭并完成安装，然后自动重新打开。"
                        } else {
                            "SerialRUN will close to install the update and reopen automatically."
                        })
                        .color(c.text_secondary)
                        .size(12.0),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(asset_name)
                            .monospace()
                            .size(10.0)
                            .color(c.text_muted),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if primary_button(
                            ui,
                            if zh { "立即安装并重启" } else { "Install & Restart" },
                            c.success,
                        )
                        .clicked()
                        {
                            action = DialogAction::Install;
                        }
                        if ui.button(if zh { "稍后安装" } else { "Install Later" }).clicked() {
                            action = DialogAction::Close;
                        }
                        if ui
                            .button(if zh { "打开文件位置" } else { "Show File" })
                            .clicked()
                        {
                            action = DialogAction::Reveal(path.clone());
                        }
                    });
                }
                UpdatePhase::Installing { version } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(if zh {
                                format!("正在安装 v{} 并重启...", version)
                            } else {
                                format!("Installing v{} and restarting...", version)
                            })
                            .color(c.text_secondary),
                        );
                    });
                }
                UpdatePhase::Error(error) => {
                    ui.label(
                        egui::RichText::new(if zh { "更新失败" } else { "Update Failed" })
                            .size(15.0)
                            .strong()
                            .color(c.error),
                    );
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(error)
                                    .size(12.0)
                                    .color(c.text_secondary),
                            );
                        });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, if zh { "重试" } else { "Try Again" }, c.success)
                            .clicked()
                        {
                            action = DialogAction::Check;
                        }
                        if ui
                            .button(if zh { "打开发布页" } else { "Release Page" })
                            .clicked()
                        {
                            action = DialogAction::ReleasePage;
                        }
                        if ui.button(if zh { "关闭" } else { "Close" }).clicked() {
                            action = DialogAction::Close;
                        }
                    });
                }
            }
        });

    match action {
        DialogAction::None => {}
        DialogAction::Check => update::start_check(state, true),
        DialogAction::Download => update::start_download(state),
        DialogAction::Cancel => update::cancel_download(state),
        DialogAction::Install => {
            let _ = update::install_and_restart(state);
        }
        DialogAction::ReleasePage => update::open_release_page(state),
        DialogAction::Reveal(path) => update::reveal_in_folder(&path),
        DialogAction::Close => open = false,
    }

    if open {
        state.update.show_dialog = true;
    } else {
        update::dismiss_dialog(state);
    }
}

fn primary_button(ui: &mut egui::Ui, text: &str, color: egui::Color32) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(color)
        .min_size(egui::vec2(0.0, 26.0)),
    )
}
