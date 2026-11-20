mod components;
mod pages;
mod storage;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use pages::{game::*, home::*};

#[component]
pub fn App() -> impl IntoView {
    view! {
        // Stripe patterns for striped cards. Pattern ids are global to the page, so they are
        // defined once here instead of inside every card.
        <svg width="0" height="0" style="position:absolute">
            <defs>
                <pattern id="stripes-red" width="8" height="8" patternUnits="userSpaceOnUse">
                    <rect width="2.5" height="8" fill="var(--card-red)" />
                </pattern>
                <pattern id="stripes-purple" width="8" height="8" patternUnits="userSpaceOnUse">
                    <rect width="2.5" height="8" fill="var(--card-purple)" />
                </pattern>
                <pattern id="stripes-green" width="8" height="8" patternUnits="userSpaceOnUse">
                    <rect width="2.5" height="8" fill="var(--card-green)" />
                </pattern>
            </defs>
        </svg>
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=Home />
                <Route path=path!("/game/:code") view=Room />
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(App);
}
