use image::{DynamicImage, ImageBuffer, RgbaImage};
use std::cmp::Ordering;
use super::color_analysis::*;
use wasm_bindgen::prelude::*;

pub fn create_collage(images: Vec<DynamicImage>, width: u32, height: u32, color_shift: f32) -> Result<DynamicImage, JsValue> {
    // Separate images into black, white, desaturated, and colored based on thresholds
    let mut black_images = Vec::new();
    let mut white_images = Vec::new();
    let mut desaturated_images = Vec::new();
    let mut colored_images = Vec::new();
    
    for img in images {
        let black_percentage = calculate_black_percentage(&img);
        let white_percentage = calculate_white_percentage(&img);
        let desaturation_percentage = calculate_desaturation_percentage(&img);
        
        if black_percentage > 60.0 {
            black_images.push(img);
        } else if white_percentage > 60.0 { // Prioritize white images over desaturated
            white_images.push(img);
        } else if desaturation_percentage > 60.0 {
            desaturated_images.push(img);
        } else {
            colored_images.push(img);
        }
    }

    // Sort colored images by hue with hue shifting to start with blues
    let mut images_with_hue: Vec<(DynamicImage, f32)> = colored_images.into_iter()
        .map(|img| {
            let hue = calculate_dominant_hue(&img);
            // Shift hue by user-selected degrees
            let shifted_hue = (hue + color_shift) % 360.0;
            (img, shifted_hue)
        })
        .collect();

    images_with_hue.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

    let sorted_colored_images: Vec<DynamicImage> = images_with_hue.into_iter()
        .map(|(img, _)| img)
        .collect();

    // Combine images in the order: colored_images, white_images, desaturated_images, black_images
    let mut final_images = sorted_colored_images;
    final_images.extend(white_images);
    final_images.extend(desaturated_images);
    final_images.extend(black_images);

    let num_images = final_images.len();
    let aspect_ratio = width as f32 / height as f32;

    // Determine optimal number of columns and rows
    let mut best_rows = 1;
    let mut best_cols = 1;
    let mut min_diff = f32::INFINITY;

    for rows in 1..=num_images as u32 {
        let cols = (num_images as f32 / rows as f32).ceil() as u32;
        let current_aspect = cols as f32 / rows as f32;
        let diff = (current_aspect - aspect_ratio).abs();

        if diff < min_diff {
            min_diff = diff;
            best_rows = rows;
            best_cols = cols;
        }
    }

    let tile_size = (width / best_cols).min(height / best_rows);
    let collage_width = tile_size * best_cols;
    let collage_height = tile_size * best_rows;

    let mut collage: RgbaImage = ImageBuffer::new(collage_width, collage_height);

    // Generate diagonal order positions
    let mut positions = Vec::new();
    for s in 0..(best_rows + best_cols - 1) {
        for row in 0..best_rows {
            if s >= row {
                let col = s - row;
                if col < best_cols {
                    positions.push((col, row));
                }
            }
        }
    }

    // If there are more images than positions, expand the grid
    let current_rows = best_rows;
    let mut current_cols = best_cols;
    while positions.len() < final_images.len() {
        current_cols += 1;
        for row in 0..current_rows {
            let col = current_cols - 1 - row;
            if col < current_cols {
                positions.push((col, row));
            }
        }
        if positions.len() >= final_images.len() {
            break;
        }
    }

    // Assign images to positions in the specified order
    for (i, img) in final_images.into_iter().enumerate() {
        if let Some((col, row)) = positions.get(i) {
            let x = (*col * tile_size) as i64;
            let y = (*row * tile_size) as i64;

            let resized = img.resize_exact(tile_size, tile_size, image::imageops::FilterType::Nearest).to_rgba8();
            image::imageops::overlay(&mut collage, &resized, x, y);
        }
    }

    Ok(DynamicImage::ImageRgba8(collage))
}
