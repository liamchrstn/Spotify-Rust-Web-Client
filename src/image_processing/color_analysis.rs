use image::DynamicImage;

pub fn calculate_black_percentage(img: &DynamicImage) -> f32 {
    let rgba = img.to_rgba8();
    let mut black_pixels = 0;
    let total_pixels = (rgba.width() * rgba.height()) as f32;

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        if a > 0 && r < 30 && g < 30 && b < 30 {
            black_pixels += 1;
        }
    }

    (black_pixels as f32 / total_pixels) * 100.0
}

pub fn calculate_white_percentage(img: &DynamicImage) -> f32 {
    let rgba = img.to_rgba8();
    let mut white_pixels = 0;
    let total_pixels = (rgba.width() * rgba.height()) as f32;

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        if a > 0 && r > 225 && g > 225 && b > 225 {
            white_pixels += 1;
        }
    }

    (white_pixels as f32 / total_pixels) * 100.0
}

pub fn calculate_desaturation_percentage(img: &DynamicImage) -> f32 {
    let rgba = img.to_rgba8();
    let mut desaturated_pixels = 0;
    let total_pixels = (rgba.width() * rgba.height()) as f32;

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        if a > 0 {
            let max = r.max(g).max(b) as f32;
            let min = r.min(g).min(b) as f32;
            let saturation = if max > 0.0 { (max - min) / max } else { 0.0 };
            if saturation < 0.2 { // Low saturation threshold
                desaturated_pixels += 1;
            }
        }
    }

    (desaturated_pixels as f32 / total_pixels) * 100.0
}

pub fn calculate_dominant_hue(img: &DynamicImage) -> f32 {
    let rgba = img.to_rgba8();
    let mut hue_sum = 0.0;
    let mut weighted_count = 0.0;

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        if a > 0 {
            let (h, s, v) = rgb_to_hsv(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            // Weight by saturation and value to give more importance to colorful, bright pixels
            let weight = s * v;
            hue_sum += h * weight;
            weighted_count += weight;
        }
    }

    if weighted_count > 0.0 {
        hue_sum / weighted_count
    } else {
        0.0
    }
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    let value = max;

    (hue, saturation, value)
}
