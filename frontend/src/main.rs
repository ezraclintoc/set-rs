mod pages;
mod storage;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use pages::{game::*, home::*};

#[component]
pub fn App() -> impl IntoView {
    view! {
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
