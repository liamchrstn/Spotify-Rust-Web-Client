use super::app_state::{ViewMode, APP_STATE};  // Changed from crate::app_state
use egui::{Context, Ui};
use egui_extras::{TableBuilder, Column};
use crate::api_request::imagerender::get_or_load_image;

fn draw_vlines<R>(ui: &mut Ui, _height: f32, draw_left: bool, next: impl FnOnce(&mut Ui) -> R) {
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let rect = ui.available_rect_before_wrap();
    next(ui);
    if draw_left {
        ui.painter().vline(
            rect.left(),
            rect.top()..=rect.bottom(),
            stroke
        );
    }
}

pub fn show_saved_tracks_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.show_tracks {
        return;
    }

    let tracks = state.saved_tracks.clone();
    let is_loading = state.is_loading;
    let total_tracks = state.total_tracks;
    let mut view_mode = state.view_mode;
    let mut window_size = state.tracks_window_size;
    let mut tracks_window_open = state.tracks_window_open;
    
    egui::Window::new("Liked Songs")
        .open(&mut tracks_window_open)
        .default_size(window_size)
        .min_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            if is_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    if let Some(total) = total_tracks {
                        ui.label(format!(
                            "Loading tracks... ({} of {} loaded)", 
                            tracks.len(), 
                            total
                        ));
                    } else {
                        ui.label("Loading tracks...");
                    }
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                ui.label("View:");
                if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).clicked() {
                    window_size = (800.0, 600.0);
                    state.view_mode = ViewMode::Grid;
                }
                ui.add_space(8.0);
                if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).clicked() {
                    state.view_mode = ViewMode::List;
                    window_size = (400.0, 600.0);
                }
            });
            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                match view_mode {
                    ViewMode::List => show_list_view(ui, &tracks),
                    ViewMode::Grid => show_grid_view(ui, &tracks),
                }
            });
        });
        
    state.tracks_window_open = tracks_window_open;
}

fn show_list_view(ui: &mut Ui, tracks: &[(String, String, String)]) {
    for (track, artists, image_url) in tracks {
        ui.horizontal(|ui| {
            // Add album art
            if let Some(image) = get_or_load_image(ui.ctx(), image_url) {
                ui.add(image.fit_to_exact_size([40.0, 40.0].into()));
            }
            
            ui.vertical(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(track)
                        .size(16.0)
                        .strong()
                        .color(ui.visuals().strong_text_color())
                ).wrap());
                
                ui.add(egui::Label::new(
                    egui::RichText::new(artists)
                        .size(14.0)
                        .color(ui.visuals().weak_text_color())
                ).wrap());
            });
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
    }
}

fn show_grid_view(ui: &mut Ui, tracks: &[(String, String, String)]) {
    let available_width = ui.available_width();
    let column_width = (available_width / 3.0).max(100.0) - 10.0; // Add padding
    
    egui::ScrollArea::horizontal().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::top_down_justified(egui::Align::Center))
            .column(Column::exact(column_width))
            .column(Column::exact(column_width))
            .column(Column::exact(column_width))
            .vscroll(true)
            .body(|mut body| {
                let rows = (tracks.len() + 2) / 3;
                for row_idx in 0..rows {
                    body.row(100.0, |mut row| {
                        for col in 0..3 {
                            let idx = row_idx * 3 + col;
                            if let Some((track, artists, image_url)) = tracks.get(idx) {
                                row.col(|ui| {
                                    draw_vlines(ui, 100.0, col > 0, |ui| {
                                        ui.horizontal(|ui| {
                                            // Add album art
                                            if let Some(image) = get_or_load_image(ui.ctx(), image_url) {
                                                ui.add(image.fit_to_exact_size([80.0, 80.0].into()));
                                            }
                                            ui.add_space(8.0);
                                            ui.vertical(|ui| {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(track)
                                                            .size(16.0)
                                                            .strong()
                                                            .color(ui.visuals().strong_text_color())
                                                    ).wrap()
                                                );
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(artists)
                                                            .size(14.0)
                                                            .color(ui.visuals().weak_text_color())
                                                    ).wrap()
                                                );
                                            });
                                        });
                                    });
                                });
                            } else {
                                row.col(|ui| {
                                    draw_vlines(ui, 100.0, col > 0, |_| {});
                                });
                            }
                        }
                    });
                }
            });
    });
}
