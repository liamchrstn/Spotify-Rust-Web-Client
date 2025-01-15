use egui::Ui;
use egui_extras::{TableBuilder, Column};
use crate::api_request::imagerender::get_or_load_image;

#[derive(PartialEq)]
pub enum ListViewMode {
    Tracks,
    Playlists,
}

fn render_default_square(ui: &mut Ui, rect: egui::Rect) {
    // Draw grey background
    ui.painter().rect_filled(
        rect,
        0.0,
        egui::Color32::from_gray(128),
    );

    // Draw music note icon
    let font_size = rect.height() * 0.4;
    let text = egui::RichText::new("♫")
        .size(font_size)
        .color(egui::Color32::from_gray(200));
    
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text.text(),
        egui::FontId::proportional(font_size),
        egui::Color32::from_gray(200),
    );
}

fn render_square_with_image(ui: &mut Ui, size: f32, image_url: &str) {
    let (rect, _) = ui.allocate_exact_size([size, size].into(), egui::Sense::hover());

    if !image_url.is_empty() {
        if let Some(texture) = get_or_load_image(ui.ctx(), image_url) {
            texture.paint_at(ui, rect);
        } else {
            render_default_square(ui, rect);
        }
    } else {
        render_default_square(ui, rect);
    }
}

pub fn show_list_view(ui: &mut Ui, tracks: &[&(String, String, String, String)], mode: ListViewMode) {
    for (track, artists, image_url, uri_or_id) in tracks {
        let row_response = ui.horizontal(|ui| {
            render_square_with_image(ui, 40.0, image_url);
            
            ui.vertical(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(track)
                        .size(16.0)
                        .strong()
                        .color(ui.visuals().strong_text_color())
                        .text_style(egui::TextStyle::Body)
                ).wrap());
                
                if mode != ListViewMode::Playlists {
                    ui.add(egui::Label::new(
                        egui::RichText::new(artists)
                            .size(14.0)
                            .color(ui.visuals().weak_text_color())
                    ).wrap());
                }
            });
        }).response;

        // Make the row clickable
        if row_response.interact(egui::Sense::click()).clicked() {
            match mode {
                ListViewMode::Tracks => {
                    let uri = uri_or_id.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        crate::api_request::track_status::play_track(uri).await;
                    });
                }
                ListViewMode::Playlists => {
                    let id = uri_or_id.clone();
                    let token = web_sys::window()
                        .and_then(|window| window.local_storage().ok().flatten())
                        .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                        .unwrap_or_default();
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        crate::api_request::playlist_tracks::fetch_playlist_tracks(id, token).await;
                    });
                }
            }
        }
        
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
    }
}

pub fn show_grid_view(ui: &mut Ui, tracks: &[&(String, String, String, String)], total_tracks: Option<i32>, saved_tracks_len: usize, loaded_tracks_count: i32, mode: ListViewMode) {
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
                            if let Some((track, artists, image_url, uri_or_id)) = tracks.get(idx) {
                                row.col(|ui| {
                                    ui.scope(|ui| {
                                        draw_vlines(ui, 100.0, col > 0, |ui| {
                                            ui.horizontal(|ui| {
                                                render_square_with_image(ui, 80.0, image_url);
                                                ui.add_space(8.0);
                                                ui.vertical(|ui| {
                                                    ui.add(
                                                        egui::Label::new(
                                                            egui::RichText::new(track)
                                                                .size(16.0)
                                                                .strong()
                                                                .color(ui.visuals().strong_text_color())
                                                                .text_style(egui::TextStyle::Body)
                                                        ).wrap()
                                                    );
                                                    if mode != ListViewMode::Playlists {
                                                        ui.add(
                                                            egui::Label::new(
                                                                egui::RichText::new(artists)
                                                                    .size(14.0)
                                                                    .color(ui.visuals().weak_text_color())
                                                            ).wrap()
                                                        );
                                                    }
                                                });
                                                
                                                // Make the cell clickable
                                                if ui.rect_contains_pointer(ui.min_rect()) {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                }
                                                if ui.rect_contains_pointer(ui.min_rect()) && ui.input(|i| i.pointer.primary_clicked()) {
                                                    match mode {
                                                        ListViewMode::Tracks => {
                                                            let uri = uri_or_id.clone();
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                crate::api_request::track_status::play_track(uri).await;
                                                            });
                                                        }
                                                        ListViewMode::Playlists => {
                                                            let id = uri_or_id.clone();
                                                            let token = web_sys::window()
                                                                .and_then(|window| window.local_storage().ok().flatten())
                                                                .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                                                                .unwrap_or_default();
                                                            
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                crate::api_request::playlist_tracks::fetch_playlist_tracks(id, token).await;
                                                            });
                                                        }
                                                    }
                                                }
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

                // Add Load More button only after the last row if we have more tracks to load
                if let Some(total) = total_tracks {
                    if tracks.len() >= saved_tracks_len && loaded_tracks_count < total {
                        body.row(50.0, |mut row| {
                            // Use all three columns for the button
                            row.col(|_| {});  // Empty first column
                            row.col(|ui| {
                                // Center the button in the middle column
                                if ui.button("Load More").clicked() {
                                    let token = web_sys::window()
                                        .and_then(|window| window.local_storage().ok().flatten())
                                        .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                                        .unwrap_or_default();
                                    
                                    wasm_bindgen_futures::spawn_local(async move {
                                        crate::api_request::saved_tracks::load_more_tracks(token, false).await;
                                    });
                                }
                            });
                            row.col(|_| {});  // Empty third column
                        });
                    }
                }
            });
    });
}

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
