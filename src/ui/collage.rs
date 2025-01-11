use super::app_state::APP_STATE;
use crate::image_processing::user_interface::get_color_shift;
use crate::image_processing::collage::create_collage;
use egui::Context;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, Url};
use wasm_bindgen::JsCast;
use std::io::Cursor;

pub fn show_collage_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.collage_window_open {
        return;
    }

    egui::Window::new("Create Collage")
        .default_pos(state.collage_window_pos)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Create a collage from your liked songs' album artwork");
            
            if ui.button("Generate Collage").clicked() {
                state.is_loading = true;
                
                // Clone tracks for async closure
                let tracks = state.saved_tracks.clone();
                
                spawn_local(async move {
                    // Get color shift preference from user
                    let color_shift = get_color_shift().await;
                    
                    // Download and process album artwork
                    let mut images = Vec::new();
                    let total_images = tracks.len();
                    let mut loaded_count = 0;
                    
                    // Update loading message
                    {
                        let mut state = APP_STATE.lock().unwrap();
                        state.is_loading = true;
                        state.loading_message = format!("Loading images (0/{})...", total_images);
                    }
                    
                    for (_, _, image_url, _) in tracks {
                        if let Ok(bytes) = reqwest::get(&image_url).await {
                            if let Ok(bytes) = bytes.bytes().await {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    images.push(img);
                                    loaded_count += 1;
                                    
                                    // Update progress message
                                    let mut state = APP_STATE.lock().unwrap();
                                    state.loading_message = format!("Loading images ({}/{})...", loaded_count, total_images);
                                }
                            }
                        }
                    }
                    
                    // Only proceed if we have images
                    if images.is_empty() {
                        let mut state = APP_STATE.lock().unwrap();
                        state.is_loading = false;
                        state.loading_message = "Failed to load any images.".to_string();
                        return;
                    }
                    
                    // Update status for collage creation
                    {
                        let mut state = APP_STATE.lock().unwrap();
                        state.loading_message = "Creating collage...".to_string();
                    }
                    
                    // Create collage with downloaded images
                    if let Ok(collage) = create_collage(images, 1920, 1080, color_shift) {
                        // Create a cursor to write the image to
                        let mut cursor = Cursor::new(Vec::new());
                        if let Ok(_) = collage.write_to(&mut cursor, image::ImageFormat::Png) {
                            let buffer = cursor.into_inner();
                            
                            // Create a Blob from the image data
                            let array = js_sys::Uint8Array::from(&buffer[..]);
                            let blob_parts = js_sys::Array::new();
                            blob_parts.push(&array);
                            
                            if let Ok(blob) = Blob::new_with_u8_array_sequence(&blob_parts) {
                                // Create a download URL
                                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                    // Create and click a download link
                                    if let Some(window) = web_sys::window() {
                                        if let Some(document) = window.document() {
                                            if let Some(link) = document.create_element("a").ok() {
                                                let link = link.dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                                                link.set_href(&url);
                                                link.set_download("collage.png");
                                                link.click();
                                                
                                                // Clean up
                                                let _ = Url::revoke_object_url(&url);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Update loading state
                    let mut state = APP_STATE.lock().unwrap();
                    state.is_loading = false;
                    state.loading_message = String::new();
                });
            }
            
            if state.is_loading || !state.loading_message.is_empty() {
                ui.spinner();
                ui.label(&state.loading_message);
            }
        });
}
