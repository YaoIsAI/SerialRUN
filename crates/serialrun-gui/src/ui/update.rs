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

/// Clickable version label in the status bar: doubles as the update entry
/// point and inline update status.
pub fn render_version_control(ui: &mut egui::Ui, state: &mut AppState) {
    let c = theme::get_colors(state.theme);
    let (status, color, hover): (Option<String>, egui::Color32, &'static str) =
        match &state.update.phase {
            UpdatePhase::Checking => (
                Some(tr(state, "checking...", "正在检查...").to_string()),
                c.info,
                tr(state, "Checking GitHub for updates", "正在检查 GitHub 更新"),
            ),
            UpdatePhase::UpToDate => (
                Some("\u{2714}".to_string()),
                c.success,
                tr(
                    state,
                    "You're up to date \u{2014} click to check again",
                    "已是最新版本，点击重新检查",
                ),
            ),
            UpdatePhase::Available(candidate) => {
                if state.dismissed_update_version == candidate.release.version {
                    (
                        None,
                        c.text_muted,
                        tr(state, "Click to check for updates", "点击检查更新"),
                    )
                } else {
                    (
                        Some(format!("\u{2B06} v{}", candidate.release.version)),
                        c.warning,
                        tr(
                            state,
                            "New version available \u{2014} click to update",
                            "发现新版本，点击升级",
                        ),
                    )
                }
            }
            UpdatePhase::Downloading {
                downloaded, total, ..
            } => {
                let pct = if *total > 0 {
                    *downloaded as f32 / *total as f32 * 100.0
                } else {
                    0.0
                };
                (
                    Some(format!("\u{2B07} {:.0}%", pct)),
                    c.info,
                    tr(state, "Downloading update", "正在下载更新"),
                )
            }
            UpdatePhase::Ready { version, .. } => (
                Some(format!("\u{2B06} v{}", version)),
                c.success,
                tr(
                    state,
                    "Update ready \u{2014} click to install",
                    "更新已就绪，点击安装",
                ),
            ),
            UpdatePhase::Installing { .. } => (
                Some(tr(state, "installing...", "正在安装...").to_string()),
                c.info,
                tr(state, "Installing update", "正在安装更新"),
            ),
            UpdatePhase::Error(_) => (
                Some("\u{26A0}".to_string()),
                c.error,
                tr(
                    state,
                    "Update check failed \u{2014} click for details",
                    "检查更新失败，点击查看详情",
                ),
            ),
            UpdatePhase::Idle => (
                None,
                c.text_muted,
                tr(state, "Click to check for updates", "点击检查更新"),
            ),
        };

    let font = egui::FontId::proportional(12.0);
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &format!("SerialRUN v{}", current_version()),
        0.0,
        egui::TextFormat {
            font_id: font.clone(),
            color: c.text_muted,
            ..Default::default()
        },
    );
    if let Some(status) = status {
        job.append(
            &format!("  {}", status),
            0.0,
            egui::TextFormat {
                font_id: font,
                color,
                ..Default::default()
            },
        );
    }

    let response = ui
        .add(egui::Label::new(job).sense(egui::Sense::click()))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(hover);
    if response.clicked() {
        update::open_dialog(state);
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
    Later,
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
                            action = DialogAction::Later;
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
                            action = DialogAction::Later;
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

            if !matches!(phase, UpdatePhase::Installing { .. }) {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let auto_label =
                        tr(state, "Check for updates on startup", "启动时自动检查更新");
                    ui.checkbox(&mut state.auto_update_check, auto_label);
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if state.last_update_check > 0 {
                                let ts = chrono::DateTime::from_timestamp_millis(
                                    state.last_update_check,
                                )
                                .map(|t| {
                                    t.with_timezone(&chrono::Local)
                                        .format("%Y-%m-%d %H:%M")
                                        .to_string()
                                })
                                .unwrap_or_default();
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {}",
                                        tr(state, "Last checked", "上次检查"),
                                        ts
                                    ))
                                    .size(11.0)
                                    .color(c.text_muted),
                                );
                            }
                        },
                    );
                });
            }
        });

    let mut dismiss = false;
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
        DialogAction::Later => {
            open = false;
            dismiss = true;
        }
        DialogAction::Close => open = false,
    }

    if open {
        state.update.show_dialog = true;
    } else if dismiss {
        update::dismiss_dialog(state);
    } else {
        state.update.show_dialog = false;
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
