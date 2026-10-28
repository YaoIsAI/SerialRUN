//! In-app updater: GitHub release check, download progress and platform installers.

use crate::state::{AppState, LogLevel};
use eframe::egui;
use serialrun_core::update::{
    self as core_update, DownloadOutcome, UpdateCandidate, UpdateCheckOutcome,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};

const AUTO_CHECK_INTERVAL_MS: i64 = 24 * 60 * 60 * 1000;
const STARTUP_DELAY_MS: i64 = 3_000;

#[derive(Debug, Clone)]
pub enum UpdatePhase {
    Idle,
    Checking,
    UpToDate,
    Available(UpdateCandidate),
    Downloading {
        version: String,
        asset_name: String,
        downloaded: u64,
        total: u64,
        speed_bps: f64,
    },
    Ready {
        version: String,
        asset_name: String,
        path: PathBuf,
        checksum_verified: bool,
    },
    Installing {
        version: String,
    },
    Error(String),
}

pub enum UpdateEvent {
    Checked(Result<UpdateCheckOutcome, String>),
    Progress {
        downloaded: u64,
        total: u64,
    },
    Downloaded {
        version: String,
        asset_name: String,
        result: Result<DownloadOutcome, String>,
    },
}

pub struct UpdateState {
    pub phase: UpdatePhase,
    pub candidate: Option<UpdateCandidate>,
    rx: Option<Receiver<UpdateEvent>>,
    cancel: Arc<AtomicBool>,
    pub show_dialog: bool,
    manual_check: bool,
    pub auto_check_done: bool,
    launched_at: i64,
    download_started_at: Option<Instant>,
    last_speed_sample: Option<(Instant, u64)>,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            phase: UpdatePhase::Idle,
            candidate: None,
            rx: None,
            cancel: Arc::new(AtomicBool::new(false)),
            show_dialog: false,
            manual_check: false,
            auto_check_done: false,
            launched_at: chrono::Utc::now().timestamp_millis(),
            download_started_at: None,
            last_speed_sample: None,
        }
    }
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn home_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn update_dir() -> PathBuf {
    home_dir().join(".serialrun").join("updates")
}

/// Called once per frame: schedules the startup check and drains background events.
pub fn poll(state: &mut AppState, ctx: &egui::Context) {
    schedule_auto_check(state, ctx);

    let mut events = Vec::new();
    if let Some(rx) = &state.update.rx {
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
    }
    if events.is_empty() {
        if state.update.rx.is_some() {
            ctx.request_repaint_after(Duration::from_millis(120));
        }
        return;
    }

    for event in events {
        match event {
            UpdateEvent::Checked(result) => handle_checked(state, result),
            UpdateEvent::Progress { downloaded, total } => {
                update_progress(state, downloaded, total);
            }
            UpdateEvent::Downloaded {
                version,
                asset_name,
                result,
            } => handle_downloaded(state, version, asset_name, result),
        }
    }

    if state.update.rx.is_some() {
        ctx.request_repaint_after(Duration::from_millis(120));
    } else {
        ctx.request_repaint();
    }
}

fn schedule_auto_check(state: &mut AppState, ctx: &egui::Context) {
    if state.update.auto_check_done {
        return;
    }
    let now = chrono::Utc::now().timestamp_millis();
    if now - state.update.launched_at < STARTUP_DELAY_MS {
        ctx.request_repaint_after(Duration::from_millis(500));
        return;
    }
    state.update.auto_check_done = true;
    if !state.auto_update_check {
        return;
    }
    if state.last_update_check > 0
        && now - state.last_update_check < AUTO_CHECK_INTERVAL_MS
    {
        return;
    }
    if !matches!(
        &state.update.phase,
        UpdatePhase::Idle | UpdatePhase::UpToDate | UpdatePhase::Error(_)
    ) {
        return;
    }
    start_check(state, false);
}

pub fn start_check(state: &mut AppState, manual: bool) {
    if matches!(
        &state.update.phase,
        UpdatePhase::Checking
            | UpdatePhase::Downloading { .. }
            | UpdatePhase::Ready { .. }
            | UpdatePhase::Installing { .. }
    ) {
        return;
    }

    if manual {
        state.update.show_dialog = true;
        state.dismissed_update_version.clear();
    }
    state.update.manual_check = manual;
    state.update.phase = UpdatePhase::Checking;
    let (tx, rx) = mpsc::channel();
    state.update.rx = Some(rx);
    std::thread::spawn(move || {
        let result = core_update::check_for_update(current_version());
        let _ = tx.send(UpdateEvent::Checked(result));
    });
}

fn handle_checked(state: &mut AppState, result: Result<UpdateCheckOutcome, String>) {
    state.update.rx = None;
    let manual = state.update.manual_check;
    state.update.manual_check = false;

    match result {
        Ok(UpdateCheckOutcome::UpToDate) => {
            state.last_update_check = chrono::Utc::now().timestamp_millis();
            state.update.phase = UpdatePhase::UpToDate;
            state.update.candidate = None;
            state.add_log_entry(
                LogLevel::Info,
                &format!("Update check: v{} is the latest version", current_version()),
            );
            if manual {
                state.update.show_dialog = true;
            }
        }
        Ok(UpdateCheckOutcome::Available(candidate)) => {
            state.last_update_check = chrono::Utc::now().timestamp_millis();
            state.add_log_entry(
                LogLevel::Info,
                &format!(
                    "Update available: v{} -> v{}",
                    current_version(),
                    candidate.release.version
                ),
            );
            state.update.phase = UpdatePhase::Available(candidate.clone());
            state.update.candidate = Some(candidate);
            state.update.show_dialog = manual;
        }
        Err(error) => {
            state.add_log_entry(LogLevel::Warning, &format!("Update check failed: {}", error));
            state.update.phase = UpdatePhase::Error(error);
            if manual {
                state.update.show_dialog = true;
            }
        }
    }
}

pub fn start_download(state: &mut AppState) {
    let candidate = match &state.update.candidate {
        Some(c) => c.clone(),
        None => {
            state.update.phase =
                UpdatePhase::Error("No update information. Check again.".into());
            return;
        }
    };
    let version = candidate.release.version.clone();
    let asset_name = candidate.asset.name.clone();
    let total = candidate.asset.size;
    state.update.cancel = Arc::new(AtomicBool::new(false));
    let cancel = state.update.cancel.clone();
    state.update.phase = UpdatePhase::Downloading {
        version: version.clone(),
        asset_name: asset_name.clone(),
        downloaded: 0,
        total,
        speed_bps: 0.0,
    };
    state.update.download_started_at = Some(Instant::now());
    state.update.last_speed_sample = Some((Instant::now(), 0));
    state.update.show_dialog = true;

    let (tx, rx) = mpsc::channel();
    state.update.rx = Some(rx);
    std::thread::spawn(move || {
        let dir = update_dir();
        let result = core_update::download_update(
            &candidate,
            &dir,
            &cancel,
            |downloaded, total| {
                let _ = tx.send(UpdateEvent::Progress { downloaded, total });
            },
        );
        let _ = tx.send(UpdateEvent::Downloaded {
            version,
            asset_name,
            result,
        });
    });
}

fn update_progress(state: &mut AppState, downloaded: u64, total: u64) {
    let speed = {
        let now = Instant::now();
        match state.update.last_speed_sample {
            Some((last_time, last_bytes)) if now.duration_since(last_time).as_millis() >= 400 => {
                let elapsed = now.duration_since(last_time).as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (downloaded.saturating_sub(last_bytes)) as f64 / elapsed
                } else {
                    0.0
                };
                state.update.last_speed_sample = Some((now, downloaded));
                speed
            }
            Some(_) => match &state.update.phase {
                UpdatePhase::Downloading { speed_bps, .. } => *speed_bps,
                _ => 0.0,
            },
            None => 0.0,
        }
    };
    if let UpdatePhase::Downloading {
        downloaded: current,
        total: current_total,
        speed_bps,
        ..
    } = &mut state.update.phase
    {
        *current = downloaded;
        if total > 0 {
            *current_total = total;
        }
        *speed_bps = speed;
    }
}

fn handle_downloaded(
    state: &mut AppState,
    version: String,
    asset_name: String,
    result: Result<DownloadOutcome, String>,
) {
    state.update.rx = None;
    state.update.download_started_at = None;
    state.update.last_speed_sample = None;
    match result {
        Ok(outcome) => {
            state.add_log_entry(
                LogLevel::Info,
                &format!(
                    "Update v{} downloaded{}",
                    version,
                    if outcome.checksum_verified {
                        " (checksum verified)"
                    } else {
                        ""
                    }
                ),
            );
            state.update.phase = UpdatePhase::Ready {
                version,
                asset_name,
                path: outcome.path,
                checksum_verified: outcome.checksum_verified,
            };
            state.update.show_dialog = true;
        }
        Err(error) => {
            let cancelled = error.to_ascii_lowercase().contains("cancel");
            state.add_log_entry(
                LogLevel::Warning,
                &format!("Update download failed: {}", error),
            );
            state.update.phase = match state.update.candidate.clone() {
                Some(candidate) if cancelled => UpdatePhase::Available(candidate),
                _ => UpdatePhase::Error(error),
            };
            state.update.show_dialog = true;
        }
    }
}

pub fn cancel_download(state: &mut AppState) {
    state.update.cancel.store(true, Ordering::Relaxed);
}

/// Entry point for the status-bar version label. Idle/up-to-date triggers a
/// manual check; other phases just surface the dialog.
pub fn open_dialog(state: &mut AppState) {
    match &state.update.phase {
        UpdatePhase::Idle | UpdatePhase::UpToDate => start_check(state, true),
        _ => state.update.show_dialog = true,
    }
}

pub fn dismiss_dialog(state: &mut AppState) {
    state.update.show_dialog = false;
    if let UpdatePhase::Available(candidate) = &state.update.phase {
        state.dismissed_update_version = candidate.release.version.clone();
    }
}

pub fn open_release_page(state: &mut AppState) {
    let url = match &state.update.phase {
        UpdatePhase::Available(candidate) => candidate.release.html_url.clone(),
        _ => core_update::RELEASES_URL.to_string(),
    };
    if let Err(e) = open_url(&url) {
        state.add_log_entry(LogLevel::Warning, &format!("Cannot open browser: {}", e));
    }
}

/// Install the downloaded update and restart. On success this function does not
/// return because the process exits so the installer can replace it.
pub fn install_and_restart(state: &mut AppState) -> Result<(), String> {
    let (version, asset_name, path) = match &state.update.phase {
        UpdatePhase::Ready {
            version,
            asset_name,
            path,
            ..
        } => (version.clone(), asset_name.clone(), path.clone()),
        _ => return Err("No downloaded update is ready".into()),
    };
    if !path.exists() {
        return Err("Downloaded update file is missing".into());
    }
    state.update.phase = UpdatePhase::Installing {
        version: version.clone(),
    };

    let result = platform_install(&path, &asset_name);
    match result {
        Ok(()) => {
            state.add_log_entry(
                LogLevel::Info,
                &format!("Installing v{} and restarting...", version),
            );
            std::process::exit(0);
        }
        Err(error) => {
            state.add_log_entry(LogLevel::Error, &format!("Update install failed: {}", error));
            state.update.phase = UpdatePhase::Error(error.clone());
            Err(error)
        }
    }
}

fn platform_install(path: &Path, asset_name: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if !asset_name.to_ascii_lowercase().ends_with("-setup.exe") {
            reveal_in_folder(path);
            return Err(
                "This download is a portable ZIP. Close SerialRUN, extract it over the old copy, then start it again."
                    .into(),
            );
        }
        spawn_windows_installer(path)
    }
    #[cfg(target_os = "macos")]
    {
        if asset_name.to_ascii_lowercase().ends_with(".dmg") {
            open_path(path)?;
            return Err(
                "The DMG was opened. Drag SerialRUN to Applications to finish the update.".into(),
            );
        }
        spawn_macos_replacer(path)
    }
    #[cfg(target_os = "linux")]
    {
        if asset_name.to_ascii_lowercase().ends_with(".deb") {
            open_path(path)?;
            return Err(
                "The .deb package was opened in your package manager. Confirm the installation there."
                    .into(),
            );
        }
        spawn_linux_replacer(path)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (path, asset_name);
        Err("Automatic installation is not supported on this platform".into())
    }
}

fn process_id() -> u32 {
    std::process::id()
}

#[cfg(target_os = "windows")]
fn spawn_windows_installer(installer: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate current executable: {}", e))?;
    let script = update_dir().join("install-update.ps1");
    if let Some(parent) = script.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create update directory: {}", e))?;
    }
    let script_body = r#"param(
    [int]$ProcessId,
    [string]$Installer,
    [string]$FallbackExe
)
$ErrorActionPreference = 'SilentlyContinue'
Wait-Process -Id $ProcessId -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 600
$proc = Start-Process -FilePath $Installer -ArgumentList '/SILENT','/SUPPRESSMSGBOXES','/NORESTART' -PassThru -Wait
if ($proc -and $proc.ExitCode -ne 0) {
    if ($FallbackExe -and (Test-Path $FallbackExe)) { Start-Process -FilePath $FallbackExe }
    exit 1
}
$exe = $null
$roots = @(
    'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
)
foreach ($root in $roots) {
    Get-ChildItem $root | Where-Object { $_.PSChildName -like '*30798AFE-C4D1-402E-AAE5-5652D993C04F*' } | ForEach-Object {
        $loc = (Get-ItemProperty $_.PSPath -Name InstallLocation).InstallLocation
        if ($loc -and (Test-Path (Join-Path $loc 'serialrun.exe'))) { $exe = Join-Path $loc 'serialrun.exe' }
    }
    if ($exe) { break }
}
if (-not $exe) {
    $fallback = Join-Path $env:LOCALAPPDATA 'Programs\SerialRUN\serialrun.exe'
    if (Test-Path $fallback) { $exe = $fallback }
}
if ($exe) {
    Start-Process -FilePath $exe
} elseif ($FallbackExe -and (Test-Path $FallbackExe)) {
    Start-Process -FilePath $FallbackExe
}
"#;
    std::fs::write(&script, script_body)
        .map_err(|e| format!("Cannot write installer helper: {}", e))?;
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
        ])
        .arg(&script)
        .arg("-ProcessId")
        .arg(process_id().to_string())
        .arg("-Installer")
        .arg(installer)
        .arg("-FallbackExe")
        .arg(&current_exe)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Cannot start installer helper: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn spawn_macos_replacer(zip: &Path) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate current executable: {}", e))?;
    let app = exe
        .ancestors()
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("app"))
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            "SerialRUN is not running from an .app bundle. Download the DMG from the release page."
                .to_string()
        })?;
    if exe.to_string_lossy().contains("AppTranslocation") {
        return Err(
            "SerialRUN is running from a read-only location. Drag it to Applications and retry."
                .into(),
        );
    }
    let parent = app
        .parent()
        .ok_or_else(|| "Cannot locate the Applications folder".to_string())?;
    if !can_write_dir(parent) {
        reveal_in_folder(&app);
        return Err(format!(
            "No permission to update {}. Move SerialRUN to Applications (or a writable folder) and retry.",
            app.display()
        ));
    }

    let script = update_dir().join("install-update.command");
    if let Some(dir) = script.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Cannot create update directory: {}", e))?;
    }
    let script_body = r#"#!/bin/bash
PID="$1"
ZIP="$2"
APP="$3"
while kill -0 "$PID" 2>/dev/null; do sleep 0.3; done
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
if ! ditto -x -k "$ZIP" "$TMP"; then exit 1; fi
NEW_APP="$(find "$TMP" -maxdepth 3 -name 'SerialRUN.app' -type d | head -n 1)"
if [ -z "$NEW_APP" ]; then exit 1; fi
BACKUP="$APP.bak-update"
rm -rf "$BACKUP"
if ! mv "$APP" "$BACKUP"; then exit 1; fi
if ! ditto "$NEW_APP" "$APP"; then mv "$BACKUP" "$APP"; exit 1; fi
rm -rf "$BACKUP"
xattr -dr com.apple.quarantine "$APP" 2>/dev/null
open "$APP"
"#;
    std::fs::write(&script, script_body)
        .map_err(|e| format!("Cannot write updater helper: {}", e))?;
    std::process::Command::new("/bin/bash")
        .arg(&script)
        .arg(process_id().to_string())
        .arg(zip)
        .arg(&app)
        .spawn()
        .map_err(|e| format!("Cannot start updater helper: {}", e))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn spawn_linux_replacer(tarball: &Path) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate current executable: {}", e))?;
    if !can_write_file(&exe) {
        reveal_in_folder(tarball);
        return Err(format!(
            "No permission to replace {}. Install the .deb package from the release page instead.",
            exe.display()
        ));
    }

    let script = update_dir().join("install-update.sh");
    if let Some(dir) = script.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Cannot create update directory: {}", e))?;
    }
    let script_body = r#"#!/bin/bash
PID="$1"
TARBALL="$2"
EXE="$3"
while kill -0 "$PID" 2>/dev/null; do sleep 0.3; done
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
if ! tar -xzf "$TARBALL" -C "$TMP"; then exit 1; fi
NEW="$(find "$TMP" -maxdepth 2 -name 'serialrun' -type f | head -n 1)"
if [ -z "$NEW" ]; then exit 1; fi
if ! cp "$NEW" "$EXE"; then exit 1; fi
chmod +x "$EXE"
DIR="$(dirname "$EXE")"
for extra in serialrun-cli serialrun-mcp; do
  SRC="$(find "$TMP" -maxdepth 2 -name "$extra" -type f | head -n 1)"
  if [ -n "$SRC" ] && [ -w "$DIR/$extra" ]; then cp "$SRC" "$DIR/$extra"; fi
done
nohup "$EXE" >/dev/null 2>&1 &
"#;
    std::fs::write(&script, script_body)
        .map_err(|e| format!("Cannot write updater helper: {}", e))?;
    std::process::Command::new("/bin/bash")
        .arg(&script)
        .arg(process_id().to_string())
        .arg(tarball)
        .arg(&exe)
        .spawn()
        .map_err(|e| format!("Cannot start updater helper: {}", e))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn can_write_file(path: &Path) -> bool {
    std::fs::OpenOptions::new().write(true).open(path).is_ok()
}

#[cfg(target_os = "macos")]
fn can_write_dir(path: &Path) -> bool {
    let probe = path.join(format!(".serialrun-update-probe-{}", process_id()));
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

pub fn open_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]).arg(path);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(path);
        c
    };
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(path);
        c
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let mut cmd = std::process::Command::new("true");
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("{}", e))
}

pub fn open_url(url: &str) -> Result<(), String> {
    open_path(Path::new(url))
}

pub fn reveal_in_folder(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let dir = path.parent().unwrap_or(path);
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_dir_lives_under_serialrun_home() {
        let dir = update_dir();
        assert!(dir.ends_with(Path::new(".serialrun").join("updates")));
    }
}
