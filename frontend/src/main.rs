mod components;
mod pages;
mod storage;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use pages::{card::*, game::*, home::*};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=Home />
                <Route path=path!("/cards") view=CardDisplay />
                <Route path=path!("/game/:code") view=GameRedirect />
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(App);
}
