use crate::state::{AppState, T};
use crate::app::get_loaded_plugins;
use eframe::egui;
use serialrun_core::plugin_install::PluginManager;
use serialrun_core::plugin_registry::{PluginRegistry, RegistryPlugin};
use serialrun_plugin_api::manifest::PluginManifest;
use super::stc_panel;
use super::spinner;
use super::plugin_ui;

pub fn render_plugin_panel(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Poll async import result
    if let Some(rx) = state.plugin_import_rx.take() {
        match rx.try_recv() {
            Ok(Ok(name)) => {
                state.plugin_importing = false;
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Installed: {}", name));
                // BUG 4 FIX: Verify plugin directory exists before re-discovering
                let mgr = PluginManager::new();
                if let Some(installed) = mgr.get(&name) {
                    let path = installed.install_path.clone();
                    drop(mgr);
                    if path.exists() {
                        discover_plugins(state);
                    } else {
                        state.add_log_entry(crate::state::LogLevel::Warning,
                            &format!("Plugin directory not found, retrying..."));
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        discover_plugins(state);
                    }
                } else {
                    discover_plugins(state);
                }
            }
            Ok(Err(e)) => {
                state.plugin_importing = false;
                state.add_log_entry(crate::state::LogLevel::Error, &format!("Install failed: {}", e));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                state.plugin_import_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                state.plugin_importing = false;
            }
        }
    }

    // Poll community search result
    if let Some(rx) = state.plugin_search_rx.take() {
        match rx.try_recv() {
            Ok(Ok(results)) => {
                state.plugin_search_loading = false;
                state.plugin_search_results = results;
            }
            Ok(Err(e)) => {
                state.plugin_search_loading = false;
                state.add_log_entry(crate::state::LogLevel::Error, &format!("Search failed: {}", e));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                state.plugin_search_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                state.plugin_search_loading = false;
            }
        }
    }

    // Poll community download result
    if let Some(rx) = state.plugin_download_rx.take() {
        match rx.try_recv() {
            Ok(Ok(name)) => {
                state.plugin_downloading = None;
                state.plugin_community_installed.insert(name.clone());
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Installed from community: {}", name));
                discover_plugins(state);
            }
            Ok(Err(e)) => {
                state.plugin_downloading = None;
                state.add_log_entry(crate::state::LogLevel::Error, &format!("Install failed: {}", e));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                state.plugin_download_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                state.plugin_downloading = None;
            }
        }
    }

    // Import progress indicator
    if state.plugin_importing {
        ui.horizontal(|ui| {
            spinner::spinner_inline(ui, egui::Color32::from_rgb(255, 200, 0));
            ui.label(egui::RichText::new(T::plugin_importing(lang)).strong());
        });
        ui.separator();
    }

    // Download progress indicator
    if let Some(ref repo) = state.plugin_downloading {
        ui.horizontal(|ui| {
            spinner::spinner_inline(ui, egui::Color32::from_rgb(59, 130, 246));
            ui.label(egui::RichText::new(format!("{}: {}", T::downloading(lang), repo)).strong());
        });
        ui.separator();
    }

    // Tab bar: [已安装] [本地] [社区]
    ui.horizontal(|ui| {
        let installed_label = T::installed_tab(lang);
        let local_label = "本地";
        let community_label = T::community_tab(lang);

        if ui.selectable_label(state.plugin_tab == 0, egui::RichText::new(installed_label).strong()).clicked() {
            state.plugin_tab = 0;
        }
        if ui.selectable_label(state.plugin_tab == 1, egui::RichText::new(local_label).strong()).clicked() {
            state.plugin_tab = 1;
            // Auto-search local plugins on first visit
            if state.plugin_search_results.is_empty() && !state.plugin_search_loading {
                do_local_search(state);
            }
        }
        if ui.selectable_label(state.plugin_tab == 2, egui::RichText::new(community_label).strong()).clicked() {
            state.plugin_tab = 2;
            // Auto-search GitHub on first visit
            if state.plugin_search_results.is_empty() && !state.plugin_search_loading {
                do_community_search(state);
            }
        }

        ui.separator();

        // Import ZIP button (only on installed tab)
        if state.plugin_tab == 0 {
            ui.add_enabled_ui(!state.plugin_importing, |ui| {
                if ui.button(T::plugin_import_btn(lang)).clicked() {
                    do_import(state);
                }
            });
        }
    });
    ui.add_space(4.0);

    // Tab content
    match state.plugin_tab {
        0 => render_installed_tab(ui, state),
        1 => render_local_tab(ui, state),
        2 => render_community_tab(ui, state),
        _ => render_installed_tab(ui, state),
    }
}

// ============================================================================
// Installed Plugins Tab
// ============================================================================

fn render_installed_tab(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    if state.plugins.is_empty() {
        ui.label(egui::RichText::new(T::no_plugins(lang)).weak());
    } else {
        let plugins: Vec<_> = state.plugins.iter().map(|p| {
            (p.name.clone(), p.manifest_name.clone(), p.version.clone(), p.author.clone(),
             p.loaded, p.capabilities.clone(), p.enabled, p.commands.clone(), p.usage.clone())
        }).collect();

        let mut action: Option<PluginAction> = None;

        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            for (name, manifest_name, version, author, loaded, capabilities, enabled, commands, usage) in plugins.iter() {
                let is_expanded = state.plugin_expanded.contains(manifest_name);

                ui.group(|ui| {
                    // Header: [dot] [name] [version] [author] ... [? help]
                    ui.horizontal(|ui| {
                        let color = if *loaded { egui::Color32::from_rgb(34, 197, 94) } else { egui::Color32::GRAY };
                        ui.label(egui::RichText::new(if *loaded { "\u{25CF}" } else { "\u{25CB}" }).color(color));
                        ui.label(egui::RichText::new(name).strong());
                        ui.label(egui::RichText::new(format!("v{}", version)).weak().small());
                        ui.label(egui::RichText::new(format!("by {}", author)).weak().small());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Help icon (?)
                            if !usage.is_empty() {
                                ui.label(egui::RichText::new("?").strong().color(egui::Color32::from_rgb(100, 100, 200)))
                                    .on_hover_text(egui::RichText::new(usage.as_str()).small());
                            }
                        });
                    });

                    // Capabilities
                    if !capabilities.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(18.0);
                            for cap in capabilities {
                                let (label, color) = match cap.as_str() {
                                    "serial_port" => ("Serial", egui::Color32::from_rgb(34, 197, 94)),
                                    "ui_panel" => ("UI", egui::Color32::from_rgb(59, 130, 246)),
                                    "file_dialog" => ("File", egui::Color32::from_rgb(168, 85, 247)),
                                    "progress" => ("Progress", egui::Color32::from_rgb(245, 158, 11)),
                                    "logging" => ("Log", egui::Color32::from_rgb(107, 114, 128)),
                                    _ => (cap.as_str(), egui::Color32::GRAY),
                                };
                                ui.label(egui::RichText::new(label).small().color(color));
                            }
                        });
                    }

                    // Actions: [Enabled] ... [Uninstall]
                    ui.horizontal(|ui| {
                        ui.add_space(18.0);
                        if *loaded {
                            let mut en = *enabled;
                            ui.checkbox(&mut en, T::enabled_label(lang));
                            if en != *enabled {
                                action = Some(PluginAction::Toggle(manifest_name.clone(), en));
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(T::uninstall_label(lang)).clicked() {
                                log::info!("[Plugin] Uninstall clicked for: {}", manifest_name);
                                action = Some(PluginAction::Uninstall(manifest_name.clone()));
                            }
                        });
                    });
                });
                ui.add_space(4.0);
            }
        });

        match action {
            Some(PluginAction::Uninstall(manifest_name)) => {
                log::info!("[Plugin] === UNINSTALL START: {} ===", manifest_name);

                // Close the plugin's window if open
                state.plugin_windows.remove(&manifest_name);

                // STEP 1: FIRST unload the plugin library (releases DLL file lock)
                // This MUST happen before trying to delete the directory on Windows
                {
                    let mut plugins = get_loaded_plugins().lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(loaded) = plugins.get_mut(&manifest_name) {
                        loaded.unload();
                        log::info!("[Plugin] Unloaded library for: {}", manifest_name);
                    }
                    plugins.remove(&manifest_name);
                    log::info!("[Plugin] Removed from loaded: {}", manifest_name);
                }

                // STEP 2: Wait for Windows to fully release the file lock
                #[cfg(target_os = "windows")]
                {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }

                // STEP 3: NOW delete the directory (DLL is unloaded, files should be unlocked)
                let mut mgr = PluginManager::new();
                match mgr.uninstall(&manifest_name) {
                    Ok(()) => log::info!("[Plugin] Uninstall from disk OK: {}", manifest_name),
                    Err(e) => log::error!("[Plugin] Uninstall from disk FAILED: {} - {}", manifest_name, e),
                }

                // Verify directory was actually deleted
                let plugin_dir = mgr.plugins_dir().join(&manifest_name);
                if plugin_dir.exists() {
                    log::error!("[Plugin] WARNING: Directory still exists after uninstall: {:?}", plugin_dir);
                    // Force delete as last resort
                    let _ = std::fs::remove_dir_all(&plugin_dir);
                    if plugin_dir.exists() {
                        log::error!("[Plugin] CRITICAL: Could not delete directory even with force: {:?}", plugin_dir);
                    } else {
                        log::info!("[Plugin] Force delete succeeded");
                    }
                }

                // Clean up all plugin UI state
                state.plugin_active_panel = None;
                state.plugin_expanded.remove(&manifest_name);
                state.plugin_ui_repl_content.remove(&manifest_name);
                state.plugin_ui_repl_input.remove(&manifest_name);
                state.plugin_ui_file_tree.remove(&manifest_name);
                state.plugin_ui_editor_content.remove(&manifest_name);
                state.plugin_ui_editor_file.remove(&manifest_name);
                state.plugin_ui_layouts.remove(&manifest_name);

                // Re-discover remaining plugins
                let prev_count = state.plugins.len();
                discover_plugins(state);
                let new_count = state.plugins.len();
                log::info!("[Plugin] === UNINSTALL DONE: {} === (plugins: {} -> {})", manifest_name, prev_count, new_count);
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Uninstalled: {}", manifest_name));
            }
            Some(PluginAction::Toggle(manifest_name, enabled)) => {
                let mut mgr = PluginManager::new();
                if enabled { mgr.enable(&manifest_name); } else { mgr.disable(&manifest_name); }
                if let Some(p) = state.plugins.iter_mut().find(|p| p.manifest_name == manifest_name) {
                    p.enabled = enabled;
                }
                if let Some(loaded) = get_loaded_plugins().lock().unwrap_or_else(|e| e.into_inner()).get_mut(&manifest_name) {
                    loaded.set_enabled(enabled);
                }
                if !enabled {
                    state.plugin_expanded.remove(&manifest_name);
                }
            }
            None => {}
        }
    }
}

// ============================================================================
// Community Tab
// ============================================================================

fn render_community_tab(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Search bar - compact layout
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.plugin_search_query)
                .hint_text(T::search_placeholder(lang))
                .desired_width(ui.available_width() - 60.0)
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            do_community_search(state);
        }
        if ui.button(T::search_btn(lang)).clicked() {
            do_community_search(state);
        }
    });

    // Loading indicator
    if state.plugin_search_loading {
        ui.horizontal(|ui| {
            spinner::spinner_inline(ui, egui::Color32::from_rgb(59, 130, 246));
            ui.label(egui::RichText::new(T::searching(lang)).strong());
        });
    }
    ui.separator();

    // Community results (GitHub only)
    let github_results: Vec<_> = state.plugin_search_results.iter()
        .filter(|p| !p.repo_name.starts_with("local/"))
        .cloned()
        .collect();

    if github_results.is_empty() && !state.plugin_search_loading {
        ui.label(egui::RichText::new(T::no_results(lang)).weak());
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if !github_results.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("\u{2601}\u{FE0F} GitHub 社区插件").strong().size(13.0));
                    ui.label(egui::RichText::new(format!("({})", github_results.len())).weak().small());
                });
                ui.separator();
                for p in &github_results {
                    render_plugin_card(ui, &p, state, lang);
                }
            }
        });
    }
}

/// Render local plugins tab
fn render_local_tab(ui: &mut egui::Ui, state: &mut AppState) {
    let lang = state.language;

    // Search bar
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.plugin_search_query)
                .hint_text("搜索本地插件...")
                .desired_width(ui.available_width() - 60.0)
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            do_local_search(state);
        }
        if ui.button(T::search_btn(lang)).clicked() {
            do_local_search(state);
        }
    });

    // Loading indicator
    if state.plugin_search_loading {
        ui.horizontal(|ui| {
            spinner::spinner_inline(ui, egui::Color32::from_rgb(59, 130, 246));
            ui.label(egui::RichText::new(T::searching(lang)).strong());
        });
    }
    ui.separator();

    // Results
    let local_results: Vec<_> = state.plugin_search_results.iter()
        .filter(|p| p.repo_name.starts_with("local/"))
        .cloned()
        .collect();

    if local_results.is_empty() && !state.plugin_search_loading {
        ui.label(egui::RichText::new("未找到本地插件。将插件放在 plugins/ 目录下即可发现。").weak());
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if !local_results.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("\u{1F4E1} 本地源码插件").strong().size(13.0));
                    ui.label(egui::RichText::new(format!("({})", local_results.len())).weak().small());
                });
                ui.separator();
                for p in &local_results {
                    render_plugin_card(ui, &p, state, lang);
                }
            }
        });
    }
}

fn render_plugin_card(ui: &mut egui::Ui, p: &RegistryPlugin, state: &mut AppState, lang: crate::state::Language) {
    let repo_name = &p.repo_name;
    let description = &p.description;
    let stars = p.stars;
    let version = p.manifest.as_ref().map(|m| m.version.clone()).unwrap_or_default();
    let author = p.manifest.as_ref().map(|m| m.author.clone()).unwrap_or_default();
    let tags = p.manifest.as_ref().map(|m| m.tags.clone()).unwrap_or_default();
    let repo_url = &p.repo_url;

    ui.group(|ui| {
        // Header: name + version + author + install button
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("\u{1F4E6}").strong());
            ui.label(egui::RichText::new(repo_name).strong());
            if !version.is_empty() {
                ui.label(egui::RichText::new(format!("v{}", version)).weak().small());
            }
            if !author.is_empty() {
                ui.label(egui::RichText::new(format!("by {}", author)).weak().small());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_downloading = state.plugin_downloading.as_deref() == Some(repo_name.as_str());
                // Strip "local/" prefix for matching against installed plugins
                let plugin_name = repo_name.strip_prefix("local/").unwrap_or(repo_name);
                let is_installed = state.plugins.iter().any(|p| p.manifest_name == *plugin_name)
                    || state.plugin_community_installed.contains(plugin_name);

                if is_installed {
                    ui.label(egui::RichText::new(T::installed_label(lang))
                        .color(egui::Color32::from_rgb(34, 197, 94)).strong());
                } else if is_downloading {
                    ui.label(egui::RichText::new(T::downloading(lang)).weak());
                } else {
                    if ui.button(T::install_btn(lang)).clicked() {
                        do_community_install(state, repo_name.clone(), repo_url.clone());
                    }
                }
            });
        });

        // Description
        if !description.is_empty() {
            ui.label(egui::RichText::new(description.as_str()).small().weak());
        }

        // Tags
        if !tags.is_empty() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("\u{2605} {}", stars)).small().color(egui::Color32::from_rgb(245, 158, 11)));
                for tag in tags.iter().take(4) {
                    ui.label(egui::RichText::new(tag.as_str()).small().color(egui::Color32::from_rgb(107, 114, 128)));
                }
            });
        }
    });
    ui.add_space(4.0);
}

// ============================================================================
// Plugin Commands Panel
// ============================================================================

/// Render plugin window content — handles the UI layout or fallback command panel
pub fn render_plugin_window_content(ui: &mut egui::Ui, state: &mut AppState, manifest_name: &str) {
    // Clone layout to avoid borrow issues
    let layout = state.plugin_ui_layouts.get(manifest_name).cloned();
    if let Some(layout) = layout {
        let repl_content = state.plugin_ui_repl_content.entry(manifest_name.to_string()).or_default();
        let repl_input = state.plugin_ui_repl_input.entry(manifest_name.to_string()).or_default();
        let file_tree = state.plugin_ui_file_tree.entry(manifest_name.to_string()).or_default();
        let editor_content = state.plugin_ui_editor_content.entry(manifest_name.to_string()).or_default();
        let editor_file = state.plugin_ui_editor_file.entry(manifest_name.to_string()).or_insert_with(|| None);
        super::plugin_ui::render_dynamic_ui(ui, &layout, manifest_name, repl_content, repl_input, file_tree, editor_content, editor_file);
    } else {
        // Fallback: render generic command panel
        let commands = state.plugins.iter()
            .find(|p| p.manifest_name == manifest_name)
            .map(|p| p.commands.clone())
            .unwrap_or_default();
        if !commands.is_empty() {
            render_plugin_commands(ui, state, manifest_name, &commands);
        } else {
            ui.label("No UI layout declared by this plugin.");
        }
    }
}

pub fn render_plugin_commands(ui: &mut egui::Ui, state: &mut AppState, plugin_name: &str, commands: &[(String, String)]) {
    let lang = state.language;
    ui.label(egui::RichText::new(format!("{} Commands", plugin_name)).strong().small());
    ui.separator();

    if commands.is_empty() {
        ui.label(egui::RichText::new("No commands available").weak());
        return;
    }

    let cmd_names: Vec<String> = commands.iter().map(|c| c.0.clone()).collect();
    let selected = state.plugin_cmd_index;

    egui::ComboBox::from_id_salt("plugin_cmd_select")
        .selected_text(cmd_names.get(selected).unwrap_or(&String::new()))
        .show_ui(ui, |ui| {
            for (i, name) in cmd_names.iter().enumerate() {
                if ui.selectable_value(&mut state.plugin_cmd_index, i, name).changed() {
                    state.plugin_cmd_params = commands[i].1.clone();
                }
            }
        });

    if let Some((_, desc)) = commands.get(selected) {
        ui.label(egui::RichText::new(desc).small().weak());
    }
    ui.add_space(4.0);

    ui.label(egui::RichText::new(T::parameters_label(lang)).small().strong());
    ui.text_edit_multiline(&mut state.plugin_cmd_params);
    ui.add_space(4.0);

    if ui.button(egui::RichText::new(T::run_command_btn(lang)).strong()).clicked() {
        if let Some((cmd_name, _)) = commands.get(selected) {
            execute_plugin_command(state, plugin_name, cmd_name, &state.plugin_cmd_params.clone());
        }
    }

    if !state.plugin_cmd_result.is_empty() {
        ui.add_space(8.0);
        ui.label(egui::RichText::new(T::result_label(lang)).small().strong());
        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
            ui.label(egui::RichText::new(&state.plugin_cmd_result).monospace().small());
        });
    }
}

// ============================================================================
// Actions
// ============================================================================

enum PluginAction {
    Uninstall(String),
    Toggle(String, bool),
}

/// Start async plugin import (non-blocking)
fn do_import(state: &mut AppState) {
    let Some(zip_path) = rfd::FileDialog::new()
        .set_title("Import Plugin ZIP")
        .add_filter("Plugin ZIP", &["zip"])
        .add_filter("All Files", &["*"])
        .pick_file()
    else { return; };

    state.plugin_importing = true;
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Importing: {}", zip_path.display()));

    let (tx, rx) = std::sync::mpsc::channel();
    state.plugin_import_rx = Some(rx);

    std::thread::spawn(move || {
        let mut mgr = PluginManager::new();
        let result = match mgr.install_from_zip(&zip_path) {
            Ok(name) => Ok(name),
            Err(e) => Err(format!("{}", e)),
        };
        let _ = tx.send(result);
    });
}

/// Search local plugins (sync, from source directory)
fn do_local_search(state: &mut AppState) {
    if state.plugin_search_loading {
        return;
    }
    state.plugin_search_loading = true;
    let query = state.plugin_search_query.clone();

    let (tx, rx) = std::sync::mpsc::channel();
    state.plugin_search_rx = Some(rx);

    std::thread::spawn(move || {
        let registry = PluginRegistry::new();
        let local_plugins = registry.search_local(&query);
        let _ = tx.send(Ok(local_plugins));
    });
}

/// Search community plugins on GitHub only (async)
fn do_community_search(state: &mut AppState) {
    if state.plugin_search_loading {
        return;
    }
    state.plugin_search_loading = true;
    let query = state.plugin_search_query.clone();

    let (tx, rx) = std::sync::mpsc::channel();
    state.plugin_search_rx = Some(rx);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let registry = PluginRegistry::new();
            // GitHub only - no local plugins
            match registry.search(&query).await {
                Ok(plugins) => Ok(plugins),
                Err(e) => {
                    log::warn!("GitHub search failed: {}", e);
                    Ok(Vec::new())
                }
            }
        });
        let _ = tx.send(result);
    });
}

/// Install a plugin from community (async)
fn do_community_install(state: &mut AppState, repo_name: String, repo_url: String) {
    state.plugin_downloading = Some(repo_name.clone());
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Installing: {}", repo_name));

    let (tx, rx) = std::sync::mpsc::channel();
    state.plugin_download_rx = Some(rx);

    // Check if this is a local plugin
    if repo_name.starts_with("local/") {
        let source_path = std::path::PathBuf::from(&repo_url);
        let plugin_name = repo_name.strip_prefix("local/").unwrap_or(&repo_name).to_string();

        std::thread::spawn(move || {
            let result = install_local_plugin(&source_path, &plugin_name);
            let _ = tx.send(result);
        });
    } else {
        // GitHub plugin
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let registry = PluginRegistry::new();
                match registry.get_plugin(&repo_name).await {
                    Ok(plugin) => match registry.install(&plugin).await {
                        Ok(name) => Ok(name),
                        Err(e) => Err(format!("{}", e)),
                    },
                    Err(e) => Err(format!("{}", e)),
                }
            });
            let _ = tx.send(result);
        });
    }
}

/// Install a plugin from a local source directory
fn install_local_plugin(source_path: &std::path::Path, plugin_name: &str) -> Result<String, String> {
    let mgr = PluginManager::new();
    let dest_dir = mgr.plugins_dir().join(plugin_name);

    // Create destination directory
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

    // Copy all files from source to destination
    copy_dir_recursive(source_path, &dest_dir)
        .map_err(|e| format!("Failed to copy plugin files: {}", e))?;

    Ok(plugin_name.to_string())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

/// Discover and load all installed plugins
pub fn discover_plugins(state: &mut AppState) {
    log::info!("[Plugin] discover_plugins: clearing state");
    state.plugins.clear();
    state.callback_stores.clear();
    state.plugin_windows.clear();
    get_loaded_plugins().lock().unwrap_or_else(|e| e.into_inner()).clear();

    let mut mgr = PluginManager::new();
    mgr.discover();

    let installed_count = mgr.installed().len();
    log::info!("[Plugin] discover_plugins: found {} installed plugin(s) on disk", installed_count);

    let ext = match std::env::consts::OS {
        "macos" => "dylib",
        "windows" => "dll",
        _ => "so",
    };

    for (name, installed) in mgr.installed() {
        let lib_path = find_plugin_lib(&installed.install_path, ext);
        log::info!("[Plugin] Loading: {} from {:?} (exists: {})", name, installed.install_path, installed.install_path.exists());
        load_plugin_entry(state, name, &installed, lib_path.as_deref());
    }

    log::info!("[Plugin] discover_plugins: loaded {} plugin(s) total", state.plugins.len());
    state.add_log_entry(crate::state::LogLevel::Info, &format!("Found {} plugin(s)", state.plugins.len()));
}

fn load_plugin_entry(state: &mut AppState, name: &str, installed: &serialrun_core::plugin_install::InstalledPlugin, lib_path: Option<&std::path::Path>) {
    if let Some(path) = lib_path {
        match serialrun_core::plugin::LoadedPlugin::load(path) {
            Ok(mut loaded) => {
                let info = loaded.info().clone();
                let capabilities: Vec<String> = loaded.capabilities().iter().map(|c| format!("{:?}", c).to_lowercase()).collect();
                let commands: Vec<(String, String)> = loaded.commands().iter()
                    .map(|c| (c.name.clone(), c.description.clone()))
                    .collect();

                let callbacks = crate::plugin_callbacks::create_callbacks();
                let boxed = Box::new(callbacks);
                let callbacks_ptr = &*boxed as *const _;
                state.callback_stores.push(boxed);
                loaded.init(callbacks_ptr);

                // Fetch UI layout if the plugin declares one
                if let Some(layout_json) = loaded.get_ui_layout() {
                    if let Ok(layout) = serialrun_plugin_api::parse_ui_layout(&layout_json) {
                        state.plugin_ui_layouts.insert(installed.manifest.name.clone(), layout);
                    }
                }

                state.plugins.push(crate::state::PluginInfo {
                    name: info.name.clone(),
                    manifest_name: installed.manifest.name.clone(),
                    version: info.version.clone(),
                    author: info.author.clone(),
                    loaded: true,
                    capabilities,
                    enabled: installed.enabled,
                    commands,
                    usage: installed.manifest.usage.clone(),
                    toolbar: installed.manifest.toolbar.clone(),
                    window_config: installed.manifest.window.clone(),
                });
                get_loaded_plugins().lock().unwrap_or_else(|e| e.into_inner()).insert(installed.manifest.name.clone(), loaded);
                state.add_log_entry(crate::state::LogLevel::Info, &format!("Loaded: {} v{}", info.name, info.version));
            }
            Err(e) => {
                state.plugins.push(crate::state::PluginInfo {
                    name: name.to_string(),
                    manifest_name: installed.manifest.name.clone(),
                    version: installed.manifest.version.clone(),
                    author: installed.manifest.author.clone(),
                    loaded: false,
                    capabilities: Vec::new(),
                    enabled: installed.enabled,
                    commands: Vec::new(),
                    usage: installed.manifest.usage.clone(),
                    toolbar: installed.manifest.toolbar.clone(),
                    window_config: installed.manifest.window.clone(),
                });
                state.add_log_entry(crate::state::LogLevel::Warning, &format!("Load failed: {}", e));
            }
        }
    } else {
        state.plugins.push(crate::state::PluginInfo {
            name: name.to_string(),
            manifest_name: installed.manifest.name.clone(),
            version: installed.manifest.version.clone(),
            author: installed.manifest.author.clone(),
            loaded: false,
            capabilities: Vec::new(),
            enabled: installed.enabled,
            commands: Vec::new(),
            usage: installed.manifest.usage.clone(),
            toolbar: installed.manifest.toolbar.clone(),
            window_config: installed.manifest.window.clone(),
        });
    }
}

fn find_plugin_lib(dir: &std::path::Path, ext: &str) -> Option<std::path::PathBuf> {
    // First, check the plugin root directory
    if let Some(found) = std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        if p.extension().and_then(|e| e.to_str()) == Some(ext) { Some(p) } else { None }
    }) {
        return Some(found);
    }

    // Then, check platform-specific subdirectories (e.g., windows-x64/, macos-arm64/, linux-x64/)
    let platform_dirs = ["windows-x64", "macos-arm64", "macos-x64", "linux-x64", "linux-arm64"];
    for sub in &platform_dirs {
        let sub_path = dir.join(sub);
        if sub_path.is_dir() {
            if let Some(found) = std::fs::read_dir(&sub_path).ok()?.flatten().find_map(|e| {
                let p = e.path();
                if p.extension().and_then(|e| e.to_str()) == Some(ext) { Some(p) } else { None }
            }) {
                return Some(found);
            }
        }
    }

    // Finally, search recursively (up to 2 levels deep)
    fn search_recursive(dir: &std::path::Path, ext: &str, depth: usize) -> Option<std::path::PathBuf> {
        if depth > 2 { return None; }
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some(ext) {
                return Some(p);
            }
            if p.is_dir() {
                if let Some(found) = search_recursive(&p, ext, depth + 1) {
                    return Some(found);
                }
            }
        }
        None
    }

    search_recursive(dir, ext, 0)
}

fn execute_plugin_command(state: &mut AppState, plugin_name: &str, command: &str, params: &str) {
    let result = {
        let mut plugins = get_loaded_plugins().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(loaded) = plugins.get_mut(plugin_name) {
            loaded.execute_command(command, params)
        } else {
            Err(serialrun_core::plugin::PluginError::PluginError("Plugin not loaded".to_string()))
        }
    };
    match result {
        Ok(result) => {
            let output = if result.success {
                serde_json::to_string_pretty(&result.result.unwrap_or_default()).unwrap_or_default()
            } else {
                format!("Error: {}", result.error.unwrap_or_default())
            };
            state.plugin_cmd_result = output;
            state.add_log_entry(crate::state::LogLevel::Info, &format!("[{}] {} OK", plugin_name, command));
        }
        Err(e) => {
            state.plugin_cmd_result = format!("Error: {}", e);
            state.add_log_entry(crate::state::LogLevel::Error, &format!("[{}] {} failed: {}", plugin_name, command, e));
        }
    }
}
