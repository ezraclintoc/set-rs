use leptos::prelude::*;

/// Top bar for the game page. The full screen button hides the bar and, where the browser
/// allows it, also enters real browser full screen. A small floating button brings it back.
#[component]
pub fn GameHeader(code: String) -> impl IntoView {
    let fullscreen = RwSignal::new(false);

    // The browser can leave full screen on its own (Esc, or the back gesture on phones).
    // Listen for that so the header comes back too.
    let listener = window_event_listener_untyped("fullscreenchange", move |_| {
        if document().fullscreen_element().is_none() {
            fullscreen.set(false);
        }
    });
    on_cleanup(move || listener.remove());

    let enter = move |_| {
        fullscreen.set(true);
        // Not every browser supports this (iPhone Safari doesn't). If it fails, the header
        // is still hidden, which is the part that matters.
        if let Some(root) = document().document_element() {
            let _ = root.request_fullscreen();
        }
    };

    let exit = move |_| {
        fullscreen.set(false);
        if document().fullscreen_element().is_some() {
            document().exit_fullscreen();
        }
    };

    view! {
        <Show
            when=move || fullscreen.get()
            fallback=move || {
                view! {
                    <header class="header">
                        <span class="header-title">"SET"</span>
                        <span class="header-code">"Room " {code.clone()}</span>
                        <button class="header-button" on:click=enter>
                            "Full screen"
                        </button>
                    </header>
                }
            }
        >
            <button class="exit-fullscreen" on:click=exit aria-label="Exit full screen">
                "✕"
            </button>
        </Show>
    }
}
