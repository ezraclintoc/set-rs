use leptos::prelude::*;

const NAME_KEY: &str = "set_name";
const TOKEN_KEY: &str = "set_token_";

pub fn save_name(name: &str) {
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = storage.set_item(NAME_KEY, name);
    }
}

pub fn load_name() -> String {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(NAME_KEY).ok().flatten())
        .unwrap_or_default()
}

pub fn save_token(token: &str, code: &str) {
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = storage.set_item(&(TOKEN_KEY.to_owned() + code), token);
    }
}

pub fn load_token(code: &str) -> Option<String> {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(&(TOKEN_KEY.to_owned() + code)).ok().flatten())
}
