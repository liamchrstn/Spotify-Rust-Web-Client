use web_sys::window;

pub async fn get_color_shift() -> f32 {
    // Default color shift of 240 degrees (start from blue)
    let default_shift = 240.0;
    
    // Try to get from localStorage
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item("color_shift") {
                if let Ok(shift) = value.parse::<f32>() {
                    return shift;
                }
            }
        }
    }
    
    default_shift
}

pub fn save_color_shift(shift: f32) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("color_shift", &shift.to_string());
        }
    }
}
