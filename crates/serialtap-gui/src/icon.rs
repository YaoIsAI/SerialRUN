pub fn generate_icon() -> egui::IconData {
    let size = 64;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);

    // "S" shape defined as a set of filled circles along the path
    // Centered at (32, 32), the S is drawn with radius ~18
    fn draw_s(pixels: &mut Vec<u8>, size: u32) {
        let s = size as f32;
        let cx = s / 2.0;
        let cy = s / 2.0;
        let r = s * 0.28; // outer radius of S stroke
        let stroke = s * 0.08; // stroke width

        for y in 0..size {
            for x in 0..size {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                // Background: green rounded rect
                let margin = 2.0;
                let corner_r = 10.0;
                let in_bg = px >= margin
                    && px < s - margin
                    && py >= margin
                    && py < s - margin;

                let bg_alpha = if in_bg {
                    // Rounded rectangle check
                    let dx = (px - margin).max(s - margin - px).max(0.0);
                    let dy = (py - margin).max(s - margin - py).max(0.0);
                    let dist_corner = ((dx - corner_r).max(0.0).powi(2)
                        + (dy - corner_r).max(0.0).powi(2))
                    .sqrt();
                    if dist_corner <= corner_r || (dx <= corner_r && dy <= corner_r) {
                        255u8
                    } else {
                        0u8
                    }
                } else {
                    0u8
                };

                // Green background
                let green = 180u8;
                let bg_r = 0u8;
                let bg_g = green;
                let bg_b = 120u8;

                // S letter mask: white on green background
                let dx = px - cx;
                let dy = py - cy;

                // S is composed of two arcs
                // Top arc: center at (cx, cy - r*0.4), going from right to left
                // Bottom arc: center at (cx, cy + r*0.4), going from left to right
                // Connection in the middle

                let arc_center_offset = r * 0.45;
                let arc_r = r * 0.55;

                // Top arc center
                let tc_x = cx;
                let tc_y = cy - arc_center_offset;
                let dtx = px - tc_x;
                let dty = py - tc_y;
                let dt = (dtx * dtx + dty * dty).sqrt();
                let angle_t = dty.atan2(dtx); // angle from center

                // Bottom arc center
                let bc_x = cx;
                let bc_y = cy + arc_center_offset;
                let dbx = px - bc_x;
                let dby = py - bc_y;
                let db = (dbx * dbx + dby * dby).sqrt();
                let angle_b = dby.atan2(dbx);

                // Middle diagonal stroke
                let in_middle = dx.abs() < stroke * 1.2
                    && dy > -arc_center_offset * 0.3
                    && dy < arc_center_offset * 0.3;

                // Top arc: from ~0 to ~PI (right to left, upper half)
                let in_top_arc = (dt - arc_r).abs() < stroke
                    && angle_t > -0.2
                    && angle_t < std::f32::consts::PI + 0.2;

                // Bottom arc: from ~-PI to ~0 (left to right, lower half)
                let in_bottom_arc = (db - arc_r).abs() < stroke
                    && angle_b > -std::f32::consts::PI - 0.2
                    && angle_b < 0.2;

                let in_s = in_top_arc || in_bottom_arc || in_middle;

                let (pr, pg, pb, pa) = if bg_alpha > 0 && in_s {
                    (255, 255, 255, 255) // white S
                } else if bg_alpha > 0 {
                    (bg_r, bg_g, bg_b, bg_alpha) // green bg
                } else {
                    (0, 0, 0, 0) // transparent
                };

                pixels.push(pr);
                pixels.push(pg);
                pixels.push(pb);
                pixels.push(pa);
            }
        }
    }

    draw_s(&mut pixels, size);

    egui::IconData {
        width: size,
        height: size,
        rgba: pixels,
    }
}
