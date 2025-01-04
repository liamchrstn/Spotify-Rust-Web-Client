use egui::Image;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Cache to store loaded images
static IMAGE_CACHE: Lazy<Mutex<HashMap<String, Image<'static>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_or_load_image(ctx: &egui::Context, url: &str) -> Option<Image<'static>> {
    let mut cache = IMAGE_CACHE.lock().unwrap();
    let url = url.to_string(); // Convert to owned String
    
    if let Some(image) = cache.get(&url) {
        return Some(image.clone());
    }

    // Include the image bytes in the context and create a new Image
    ctx.include_bytes(url.clone(), vec![]); // Empty bytes for now, will be loaded async
    let image = Image::from_uri(url.clone());
    cache.insert(url, image.clone());
    
    Some(image)
}
