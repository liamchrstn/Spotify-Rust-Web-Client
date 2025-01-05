use eframe::egui;
use crate::mediaplayer::scrubber::ScrubBar;
use crate::mediaplayer::scrubber::TimeManager;
use crate::ui::app_state::APP_STATE;

pub fn show_mediaplayer_window(ctx: &egui::Context) {
    let mut time_manager = TimeManager::new(100_000.0, 1.0);
    let mut state = APP_STATE.lock().unwrap();

    // Media player window
    egui::Window::new("Music Player")
        .resizable(true)
        .open(&mut state.player_window_open)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let padding = 20.0;
                let square_size = egui::vec2(100.0, 100.0);
                let total_size = ui.available_size();
                ui.set_min_size(total_size);

                // Album art placeholder
                let rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2((ui.available_width() -  square_size.x) * 0.5, padding),
                    square_size
                );
                ui.painter().rect_filled(rect, 10.0, egui::Color32::BLUE);
                ui.add_space(square_size.y + padding);
                
                time_manager.update();
                
                // Media controls
                let mut scrub_bar = ScrubBar::new(time_manager.end_time);
                let scrubber_height = 30.0;
                scrub_bar.add(ui, &mut time_manager.current_time, egui::vec2(square_size.x, scrubber_height));

                // Ensure the play container is below the scrubber
                ui.add_space(10.0); // Add some space between the scrubber and the play container
                let center_x = rect.center().x;
                let button_size = egui::vec2(40.0, 40.0); // Define button size here
                let spacing = 10.0;
                let button_container_width = button_size.x * 3.0 + spacing * 2.0; // Width of the button container
                ui.allocate_ui_at_rect(
                    egui::Rect::from_center_size(
                        egui::pos2(center_x, rect.max.y + padding + scrubber_height), // Adjusted y position
                        egui::vec2(button_container_width, button_size.y)
                    ),
                    |ui| {
                        scrub_bar.play_button(ui, &mut time_manager.playing, button_size);
                    }
                );

                // Add track title and artist name
                ui.label("Track Title");
                ui.label(egui::RichText::new("Artist Name").small());
            });
        });
    
    //continuous repaint while playing
    if time_manager.playing {
        ctx.request_repaint();
    }
}