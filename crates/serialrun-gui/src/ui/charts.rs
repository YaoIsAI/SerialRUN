use crate::state::{AppState, T};
use eframe::egui;

pub fn render_chart_panel(ui: &mut egui::Ui, state: &AppState) {
    let lang = state.language;
    ui.horizontal(|ui| {
        ui.label(T::data_rate(lang));
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
            points.clone(),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 100)),
        ));
    }

    // Draw max label
    painter.text(
        rect.left_top() + egui::vec2(5.0, 5.0),
        egui::Align2::LEFT_TOP,
        format!("Max: {:.1}", max_value),
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );

    // Handle hover — show value at cursor position
    if response.hovered() {
        if let Some(cursor) = response.hover_pos() {
            // Map cursor X to data index
            let relative_x = (cursor.x - rect.left()) / width;
            let idx = (relative_x * (state.chart_data.len() - 1).max(1) as f32).round() as usize;
            let idx = idx.min(state.chart_data.len() - 1);
            let value = state.chart_data[idx];

            // Draw crosshair
            let crosshair_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, 100);
            painter.line_segment(
                [egui::pos2(cursor.x, rect.top()), egui::pos2(cursor.x, rect.bottom())],
                egui::Stroke::new(1.0, crosshair_color),
            );
            painter.line_segment(
                [egui::pos2(rect.left(), cursor.y), egui::pos2(rect.right(), cursor.y)],
                egui::Stroke::new(1.0, crosshair_color),
            );

            // Draw dot at data point
            if idx < points.len() {
                painter.circle_filled(points[idx], 4.0, egui::Color32::from_rgb(255, 200, 0));
            }

            // Draw tooltip
            let tooltip_text = format!("{}: {:.1}", T::value(lang), value);
            let tooltip_pos = if cursor.x > rect.center().x {
                cursor - egui::vec2(80.0, 25.0)
            } else {
                cursor + egui::vec2(10.0, -25.0)
            };
            let tooltip_rect = egui::Rect::from_min_size(tooltip_pos, egui::vec2(80.0, 20.0));
            painter.rect_filled(tooltip_rect, 4.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200));
            painter.text(
                tooltip_pos + egui::vec2(4.0, 3.0),
                egui::Align2::LEFT_TOP,
                &tooltip_text,
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );
        }
    }
}
