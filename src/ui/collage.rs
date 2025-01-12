use super::app_state::{APP_STATE, GradientDirection, StartingCorner}; // Import enums from app_state
use crate::image_processing::collage::create_collage;
use egui::{Context, Color32, ColorImage, load::SizedTexture, ProgressBar}; // Add Color32
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, Url};
use wasm_bindgen::JsCast;
use std::io::Cursor;

fn download_collage(image_data: &[u8]) {
    // Create a Blob from the image data
    let array = js_sys::Uint8Array::from(image_data);
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array);
    
    if let Ok(blob) = Blob::new_with_u8_array_sequence(&blob_parts) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h {
        0.0..=60.0 => (c, x, 0.0),
        60.0..=120.0 => (x, c, 0.0),
        120.0..=180.0 => (0.0, c, x),
        180.0..=240.0 => (0.0, x, c),
        240.0..=300.0 => (x, 0.0, c),
        300.0..=360.0 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub fn show_collage_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    let mut collage_window_open = state.collage_window_open;
    if !collage_window_open {
        return;
    }

    egui::Window::new("Create Collage")
        .default_pos(state.collage_window_pos)
        .resizable(true)
        .collapsible(true)
        .open(&mut collage_window_open) // Use local variable
        .show(ctx, |ui| {
            ui.label("Create a collage from your liked songs' album artwork");

            ui.collapsing("Collage Settings", |ui| {
                // Add input fields for width and height
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::DragValue::new(&mut state.collage_width).range(100..=3840));
                    ui.label("Height:");
                    ui.add(egui::DragValue::new(&mut state.collage_height).range(100..=2160));
                });

                // Add slider for hue shift
                ui.horizontal(|ui| {
                    ui.label("Starting Hue:")
                    .on_hover_text("Choose the starting color of the rainbow gradient effect");
                    let hue_shift = &mut state.hue_shift;
                    ui.add(egui::Slider::new(hue_shift, 0.0..=360.0).text("degrees").show_value(false));
                    let (r, g, b) = hsv_to_rgb(*hue_shift, 1.0, 1.0);
                    let color = Color32::from_rgb(r, g, b);
                    ui.colored_label(color, format!("{:.0}°", hue_shift));
                });

                // Add options for gradient direction
                ui.horizontal(|ui| {
                    ui.label("Gradient Direction:");
                    ui.selectable_value(&mut state.gradient_direction, GradientDirection::Diagonal, "Diagonal");
                    ui.selectable_value(&mut state.gradient_direction, GradientDirection::Horizontal, "Horizontal");
                    ui.selectable_value(&mut state.gradient_direction, GradientDirection::Vertical, "Vertical");
                });

                // Conditionally show options for starting corner or side
                match state.gradient_direction {
                    GradientDirection::Diagonal => {
                        ui.horizontal(|ui| {
                            ui.label("Starting Corner:");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::TopLeft, "Top Left");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::TopRight, "Top Right");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::BottomLeft, "Bottom Left");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::BottomRight, "Bottom Right");
                        });
                    },
                    GradientDirection::Horizontal => {
                        ui.horizontal(|ui| {
                            ui.label("Starting Side:");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::TopLeft, "Top");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::TopRight, "Bottom");
                        });
                    },
                    GradientDirection::Vertical => {
                        ui.horizontal(|ui| {
                            ui.label("Starting Side:");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::TopLeft, "Left");
                            ui.selectable_value(&mut state.starting_corner, StartingCorner::BottomLeft, "Right");
                        });
                    },
                }
            });

            // Show preview if we have a generated image
            if let Some(image_data) = &state.collage_image {
                if ui.button("Download Collage").clicked() {
                    download_collage(image_data);
                }
                
                // Convert image data to egui texture for preview
                if let Ok(img) = image::load_from_memory(image_data) {
                    let size = [img.width() as _, img.height() as _];
                    let pixels = img.to_rgba8();
                    let pixels = pixels.as_flat_samples();
                    let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    let texture = ctx.load_texture(
                        "collage-preview",
                        color_image,
                        Default::default()
                    );
                    
                    // Calculate preview size to fit window while maintaining aspect ratio
                    let max_width = ui.available_width().min(600.0);
                    let aspect_ratio = size[0] as f32 / size[1] as f32;
                    let preview_size = [max_width, max_width / aspect_ratio];
                    
                    ui.image(SizedTexture::new(texture.id(), preview_size));
                }
            }
            
            // Only show generate button when not loading
            if !state.collage_loading {
                if ui.button("Generate New Collage").clicked() {
                    // Clone tracks for async closure
                    let tracks = state.saved_tracks.clone();
                    let width = state.collage_width;
                    let height = state.collage_height;
                    let hue_shift = state.hue_shift; // Get hue shift value from state
                    let gradient_direction = state.gradient_direction;
                    let starting_corner = state.starting_corner;
                    
                    // Set collage_loading to true
                    state.collage_loading = true;
                    
                    spawn_local(async move {
                        // Download and process album artwork
                        let mut images = Vec::new();
                        let total_images = tracks.len();
                        let mut loaded_count = 0;
                        
                        // Update loading message
                        {
                            let mut state = APP_STATE.lock().unwrap();
                            state.loading_message = format!("Loading images (0/{})...", total_images);
                        }
                        
                        for (_, _, image_url, _) in tracks {
                            if let Ok(bytes) = reqwest::get(&image_url).await {
                                if let Ok(bytes) = bytes.bytes().await {
                                    if let Ok(img) = image::load_from_memory(&bytes) {
                                        images.push(img);
                                        loaded_count += 1;
                                        // Update progress
                                        let mut state = APP_STATE.lock().unwrap();
                                        state.progress = loaded_count as f32 / total_images as f32;
                                    }
                                }
                            }
                        }
                        
                        // Only proceed if we have images
                        if images.is_empty() {
                            let mut state = APP_STATE.lock().unwrap();
                            state.progress = 0.0;
                            state.collage_loading = false; // Reset collage_loading
                            return;
                        }
                        
                        // Create collage with downloaded images
                        if let Ok(collage) = create_collage(images, width, height, hue_shift, gradient_direction, starting_corner) {
                            // Create a cursor to write the image to
                            let mut cursor = Cursor::new(Vec::new());
                            if let Ok(_) = collage.write_to(&mut cursor, image::ImageFormat::Png) {
                                let buffer = cursor.into_inner();
                                let mut state = APP_STATE.lock().unwrap();
                                state.collage_image = Some(buffer);
                            }
                        }
                        
                        // Update loading state
                        let mut state = APP_STATE.lock().unwrap();
                        state.progress = 0.0;
                        state.collage_loading = false; // Reset collage_loading
                    });
                }
            }
            
            if state.collage_loading {
                let progress_text = format!("{}/{}", (state.progress * state.saved_tracks.len() as f32).round() as i32, state.saved_tracks.len());
                ui.add(ProgressBar::new(state.progress).animate(true).text(progress_text));
            }
        });

    state.collage_window_open = collage_window_open;
}
