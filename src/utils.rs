use web_sys::console;

pub fn log_error(message: &str) {
    console::error_1(&message.into());
}

pub fn clear_token_and_redirect() {
    if let Some(window) = web_sys::window() {
        if let Ok(local_storage) = window.local_storage() {
            if let Some(storage) = local_storage {
                let _ = storage.remove_item("spotify_token");
            }
        }
        window.location().set_href("/").unwrap();
    }
}