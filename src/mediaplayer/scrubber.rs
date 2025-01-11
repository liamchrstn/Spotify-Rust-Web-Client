use egui::{pos2, Color32, Id, Rect, Sense, Stroke, Ui, Vec2};

pub struct ScrubBar {
    end_time: f64,
}

impl ScrubBar {
    pub fn new(end_time: f64) -> Self {
        Self { end_time }
    }

    pub fn add(&mut self, ui: &mut Ui, current_time: &mut f64, size: Vec2) {
        let line_thickness = 4.0;
        let circle_radius = line_thickness / 2.0;
        let pointer_radius = 6.0;
        let (scrub_response, scrub_painter) =
            ui.allocate_painter(size, Sense::union(Sense::click_and_drag(), Sense::hover()));

        // Base line coordinates
        let start_y = scrub_painter.clip_rect().center().y;
        let start_x = scrub_painter.clip_rect().min.x + pointer_radius + circle_radius;
        let end_x = scrub_painter.clip_rect().max.x - pointer_radius - circle_radius;

        // Draw base line with end caps
        scrub_painter.line_segment(
            [pos2(start_x, start_y), pos2(end_x, start_y)],
            Stroke::new(line_thickness, Color32::DARK_GRAY),
        );

        let start_center = pos2(start_x, start_y);
        let end_center = pos2(end_x, start_y);
        
        scrub_painter.circle_filled(start_center, circle_radius, Color32::DARK_GRAY);
        scrub_painter.circle_filled(end_center, circle_radius, Color32::DARK_GRAY);

        // Handle hover state and time calculation
        let mut hover_time = None;
        if let Some(hover_pos) = scrub_response.hover_pos() {
            if scrub_painter.clip_rect().contains(hover_pos) {
                let total = end_x - start_x;
                let distance = (hover_pos.x - start_x).clamp(0.0, total);
                let progress = (distance / total) as f64;
                hover_time = Some(self.end_time * progress);
            }
        }

        // Calculate and update current position
        let progress = (*current_time / self.end_time) as f32;
        let current_cursor_x = start_x + (end_x - start_x) * progress;

        if scrub_response.is_pointer_button_down_on() {
            let current_pos = scrub_response
                .interact_pointer_pos()
                .unwrap_or_else(|| pos2(start_x, start_y));
            let total = end_x - start_x;
            let distance = (current_pos.x - start_x).clamp(0.0, total);
            let progress = (distance / total) as f64;
            *current_time = self.end_time * progress;

            // Call the SDK's seek function
            let seek_time = *current_time as i32;
            let _ = js_sys::eval(&format!("window.seekTo && window.seekTo({seek_time})"));
        }

        // Draw hover indicator
        if let Some(hover_pos) = scrub_response.hover_pos() {
            if scrub_painter.clip_rect().contains(hover_pos) && !scrub_response.is_pointer_button_down_on() {
                scrub_painter.circle_filled(
                    pos2(
                        hover_pos.x.clamp(
                            scrub_painter.clip_rect().min.x + pointer_radius,
                            scrub_painter.clip_rect().max.x - pointer_radius
                        ),
                        start_y
                    ),
                    pointer_radius,
                    Color32::GRAY,
                );
            }
        }

        // Draw time tooltip
        if scrub_response.is_pointer_button_down_on() || scrub_response.hovered() {
            let text_time = hover_time.map_or_else(
                || time_stamp_to_string(*current_time),
                time_stamp_to_string
            );
            egui::show_tooltip_at_pointer(ui.ctx(), ui.layer_id(), Id::new("Scrub tooltip"), |ui| {
                ui.label(text_time);
            });
        }

        // Draw current position indicator
        scrub_painter.circle_filled(
            pos2(current_cursor_x, start_y),
            pointer_radius,
            Color32::WHITE,
        );

        // Draw time labels using dynamic layout
        let scrub_rect = scrub_painter.clip_rect();
        let text_height = ui.text_style_height(&egui::TextStyle::Body);
        let y_offset = (scrub_rect.height() - text_height) / 2.0;
        
        ui.horizontal(|ui| {
            ui.allocate_ui_at_rect(
                Rect::from_min_max(
                    pos2(scrub_rect.min.x - 60.0, scrub_rect.min.y + y_offset),
                    pos2(scrub_rect.min.x - 20.0, scrub_rect.min.y + y_offset + text_height)
                ),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(time_stamp_to_string(*current_time));
                    });
                }
            );
            ui.allocate_ui_at_rect(
                Rect::from_min_max(
                    pos2(scrub_rect.max.x + 20.0, scrub_rect.min.y + y_offset),
                    pos2(scrub_rect.max.x + 60.0, scrub_rect.min.y + y_offset + text_height)
                ),
                |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(time_stamp_to_string(self.end_time));
                        });
                    });
                }
            );
        });
    }
}

pub struct TimeManager {
    pub current_time: f64,
    pub end_time: f64,
    pub playing: bool,
}

impl TimeManager {
    pub fn new(duration_ms: f64, _speed: f32) -> Self {
        Self {
            current_time: 0.0,
            end_time: duration_ms,
            playing: false,
        }
    }
}

pub fn time_stamp_to_string(time: f64) -> String {
    let n = time as i32;
    let mins = n / (1000 * 60);
    let secs = (n / 1000) % 60;
    format!("{mins:02}:{secs:02}")
}
