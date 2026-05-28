use crate::state::{AppState, T};
use eframe::egui;

pub fn render_chart_panel(ui: &mut egui::Ui, state: &AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label("Data Rate (bytes/s)");
        ui.separator();
        ui.label(format!("RX: {} {}", state.rx_count, T::bytes(lang)));
        ui.label(format!("TX: {} {}", state.tx_count, T::bytes(lang)));
    });

    ui.separator();

    let available = ui.available_size();
    let (response, painter) = ui.allocate_painter(available, egui::Sense::hover());

    let rect = response.rect;
    let width = rect.width();
    let height = rect.height();

    // Draw background
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));

    // Draw grid
    let grid_color = egui::Color32::from_gray(50);
    for i in 0..10 {
        let y = rect.top() + (height * i as f32 / 10.0);
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, grid_color),
        );
    }

    for i in 0..10 {
        let x = rect.left() + (width * i as f32 / 10.0);
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, grid_color),
        );
    }

    if state.chart_data.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            T::no_data(lang),
            egui::FontId::proportional(14.0),
            egui::Color32::GRAY,
        );
        return;
    }

    // Draw data line
    let max_value = state.chart_data.iter().cloned().fold(0.0f64, f64::max).max(1.0);
    let points: Vec<egui::Pos2> = state
        .chart_data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.left() + (width * i as f32 / (state.chart_data.len() - 1).max(1) as f32);
            let y = rect.bottom() - (height * v as f32 / max_value as f32);
            egui::pos2(x, y)
        })
        .collect();

    if points.len() > 1 {
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 100)),
        ));
    }

    // Draw labels
    painter.text(
        rect.left_top() + egui::vec2(5.0, 5.0),
        egui::Align2::LEFT_TOP,
        format!("Max: {:.1}", max_value),
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );
}
