use egui::Context;
use super::app_state::APP_STATE;
use egui_theme_switch::global_theme_switch;

pub fn show_settings_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.settings_window_open {
        return;
    }

    egui::Window::new("Settings")
        .open(&mut state.settings_window_open)
        .show(ctx, |ui| {
            ui.heading("Appearance");
            ui.horizontal(|ui| {
                ui.label("Theme:");
                global_theme_switch(ui);
            });
            ui.add_space(8.0);
            ui.label("This is the settings window.");
            // ... add settings controls here ...
        });
}
