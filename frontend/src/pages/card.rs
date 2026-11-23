use crate::components::card::CardFace;
use leptos::prelude::*;
use shared::card::Card;

#[component]
pub fn CardDisplay() -> impl IntoView {
    view! {
        <div class="gallery">
            {(0..81u8)
                .map(|id| view! { <CardFace card=Card::from_id(id) /> })
                .collect_view()}
        </div>
    }
}
