/// Global ASCII spinner animation for consistent loading indicators across all panels.
use eframe::egui;

const BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn spinner_char(ui: &egui::Ui) -> char {
    let t = ui.ctx().input(|i| i.time);
    let frame = (t * 8.0) as usize % BRAILLE.len();
    BRAILLE[frame]
}

/// Render a braille spinner with colored text.
pub fn spinner_label(ui: &mut egui::Ui, text: &str, color: egui::Color32) -> egui::Response {
    let ch = spinner_char(ui);
    ui.label(egui::RichText::new(format!("{} {}", ch, text))
        .color(color).monospace())
}

/// Render an inline spinner (no text, just the animated character).
pub fn spinner_inline(ui: &mut egui::Ui, color: egui::Color32) -> egui::Response {
    let ch = spinner_char(ui);
    ui.label(egui::RichText::new(format!("{}", ch))
        .color(color).monospace())
}

/// Render a progress bar with ASCII spinner animation on the left.
pub fn progress_with_spinner(ui: &mut egui::Ui, progress: f32, text: &str, color: egui::Color32) {
    let ch = spinner_char(ui);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}", ch))
            .color(color).monospace().size(14.0));
        ui.add(egui::ProgressBar::new(progress).text(text));
    });
}
