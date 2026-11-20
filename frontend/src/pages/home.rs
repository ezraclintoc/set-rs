use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::storage::*;

#[component]
pub fn Home() -> impl IntoView {
    let navigate = use_navigate();
    let (user, set_user) = signal(load_name());
    let (code, set_code) = signal(String::new());

    let can_join = move || !user.get().trim().is_empty() && !code.get().trim().is_empty();

    let join = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !can_join() {
            return;
        }
        save_name(user.get().trim());
        let code = String::from(js_sys::encode_uri_component(code.get().trim()));
        navigate(&format!("/game/{code}"), Default::default());
    };

    view! {
        <div class="app">
            <form class="join" on:submit=join>
                <h1>"SET"</h1>
                <input
                    placeholder="Username"
                    maxlength="19"
                    prop:value=user
                    on:input=move |ev| set_user.set(event_target_value(&ev))
                />
                <input
                    placeholder="Code"
                    prop:value=code
                    on:input=move |ev| set_code.set(event_target_value(&ev))
                />
                <button type="submit" disabled=move || !can_join()>"Join"</button>
            </form>
        </div>
    }
}
