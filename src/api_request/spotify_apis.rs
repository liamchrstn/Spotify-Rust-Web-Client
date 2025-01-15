use crate::utils::{log_error, clear_token_and_redirect};
use web_sys;
use js_sys;

// Handles responses that don't return any content (204 No Content)
pub async fn handle_empty_response<F>(response: reqwest::Result<reqwest::Response>, success_handler: F)
where
    F: FnOnce() -> (),
{
    match response {
        Ok(response) => {
            let status = response.status();
            if status == 401 {
                web_sys::console::log_1(&"Unauthorized, clearing token".into());
                clear_token_and_redirect();
            } else if status.is_success() {
                success_handler();
            } else {
                web_sys::console::log_2(&"Empty response error:".into(), &status.as_u16().into());
                log_error(&format!("Failed to execute command: {:?}", status));
            }
        }
        Err(err) => {
            web_sys::console::log_2(&"Empty response request error:".into(), &err.to_string().into());
            log_error(&format!("Request error: {:?}", err));
        }
    }
}

// Handles the response from an API request, calling the success handler if the request is successful
pub async fn handle_response<T, F>(response: reqwest::Result<reqwest::Response>, success_handler: F)
where
    F: FnOnce(T) -> (),
    T: serde::de::DeserializeOwned,
{
    // Set default window state in case of errors
    let set_default_state = || {
        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::set(&window, &"currentPlayerState".into(), &wasm_bindgen::JsValue::NULL);
            let _ = js_sys::Reflect::set(&window, &"currentPlaybackTime".into(), &0.0.into());
            let _ = js_sys::Reflect::set(&window, &"totalDuration".into(), &0.0.into());
            let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &false.into());
        }
    };

    match response {
        Ok(response) => {
            let status = response.status();
            if status == 401 {
                web_sys::console::log_1(&"Unauthorized, clearing token".into());
                clear_token_and_redirect();
                set_default_state();
            } else if status == 204 {
                web_sys::console::log_2(&"No content response:".into(), &status.as_u16().into());
                set_default_state();
            } else if status.is_success() {
                match response.json::<T>().await {
                    Ok(data) => success_handler(data),
                    Err(err) => {
                        web_sys::console::log_2(&"Failed to parse response:".into(), &err.to_string().into());
                        log_error(&format!("Failed to parse response: {:?}", err));
                        set_default_state();
                    }
                }
            } else {
                web_sys::console::log_2(&"API error:".into(), &status.as_u16().into());
                log_error(&format!("Failed to fetch data: {:?}", status));
                set_default_state();
            }
        }
        Err(err) => {
            log_error(&format!("Request error: {:?}", err));
            set_default_state();
        }
    }
}
