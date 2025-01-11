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
            web_sys::console::log_2(&"Empty response status:".into(), &response.status().as_u16().into());
            if response.status() == 401 {
                web_sys::console::log_1(&"Unauthorized, clearing token".into());
                clear_token_and_redirect();
            } else if response.status().is_success() {
                web_sys::console::log_1(&"Empty response success".into());
                success_handler();
            } else {
                web_sys::console::log_2(&"Empty response error:".into(), &response.status().as_u16().into());
                log_error(&format!("Failed to execute command: {:?}", response.status()));
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
            web_sys::console::log_2(&"API response status:".into(), &response.status().as_u16().into());
            if response.status() == 401 {
                web_sys::console::log_1(&"Unauthorized, clearing token".into());
                clear_token_and_redirect();
                set_default_state();
            } else if response.status() == 204 {
                web_sys::console::log_1(&"No content response".into());
                set_default_state();
            } else if response.status().is_success() {
                web_sys::console::log_1(&"Successful response".into());
                match response.json::<T>().await {
                    Ok(data) => success_handler(data),
                    Err(err) => {
                        web_sys::console::log_2(&"Failed to parse response:".into(), &err.to_string().into());
                        log_error(&format!("Failed to parse response: {:?}", err));
                        set_default_state();
                    }
                }
            } else {
                log_error(&format!("Failed to fetch data: {:?}", response.status()));
                set_default_state();
            }
        }
        Err(err) => {
            log_error(&format!("Request error: {:?}", err));
            set_default_state();
        }
    }
}
