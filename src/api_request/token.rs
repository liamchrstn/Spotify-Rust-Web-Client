use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::api_request::User_Profile::fetch_user_profile;

pub static ACCESS_TOKEN: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
pub static SDK_STATUS: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
pub fn set_access_token(token: String) {
    let mut stored_token = ACCESS_TOKEN.lock().unwrap();
    *stored_token = Some(token.clone());
    spawn_local(async move {
        fetch_user_profile(token).await;
    });
}

#[wasm_bindgen]
pub fn set_sdk_status(status: String) {
    let mut sdk_status = SDK_STATUS.lock().unwrap();
    *sdk_status = Some(status);
}