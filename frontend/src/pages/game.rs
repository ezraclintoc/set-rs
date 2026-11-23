use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_params_map;

use futures::StreamExt;
use futures::channel::mpsc;
use gloo_net::websocket::{Message, futures::WebSocket};
use leptos::logging::*;
use leptos::task::spawn_local;
use shared::card::*;
use shared::game::{Phase, PlayerInfo};
use shared::{ClientMsg, ServerMsg};

use crate::components::card::CardFace;
use crate::components::header::GameHeader;
use crate::storage::*;

#[component]
pub fn GameRedirect() -> impl IntoView {
    if load_name().trim().is_empty() {
        Either::Left(view! { <Redirect path="/" /> })
    } else {
        Either::Right(view! { <Game /> })
    }
}

#[component]
pub fn Game() -> impl IntoView {
    let player_id = RwSignal::new(None::<u32>);
    let phase = RwSignal::new(None::<Phase>);
    let board = RwSignal::new(Vec::<u8>::new());
    let selected_cards = RwSignal::new(Vec::<u8>::new());
    let deck_left = RwSignal::new(81);
    let players = RwSignal::new(Vec::<PlayerInfo>::new());

    let error = RwSignal::new(None::<String>);

    let params = use_params_map();
    let code = params.read_untracked().get("code").unwrap_or_default();
    // `code` is moved into the socket task below, so the header gets its own copy.
    let header_code = code.clone();

    let location = window().location();
    let host = if location.port().unwrap() == "8080" {
        "localhost:8888".to_string()
    } else {
        location.host().unwrap()
    };
    let scheme = if location.protocol().unwrap() == "https:" {
        "wss"
    } else {
        "ws"
    };
    let url = format!("{scheme}://{host}/api/game/{code}/ws");
    let ws = WebSocket::open(&url).unwrap();
    let (write, mut read) = ws.split();
    let (tx, rx) = mpsc::unbounded::<Message>();

    let json = serde_json::to_string(&ClientMsg::Join {
        name: load_name(),
        token: load_token(code.as_str()),
    })
    .unwrap();
    let _ = tx.unbounded_send(Message::Text(json));

    // StoredValue makes the sender Copy, so event handlers in the view can use it.
    let tx = StoredValue::new(tx);
    let send = move |msg: ClientMsg| {
        let Ok(json) = serde_json::to_string(&msg) else {
            return;
        };
        tx.with_value(|tx| {
            let _ = tx.unbounded_send(Message::Text(json));
        });
    };

    spawn_local(async move {
        let _ = rx.map(Ok).forward(write).await;
    });
    spawn_local(async move {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            match serde_json::from_str::<ServerMsg>(&text) {
                Ok(ServerMsg::Welcome {
                    player_id: pid,
                    token,
                }) => {
                    player_id.set(Some(pid));
                    save_token(token.as_str(), code.as_str());
                }
                Ok(ServerMsg::State {
                    phase: ph,
                    board: b,
                    players: pl,
                    deck_left: dl,
                }) => {
                    phase.set(Some(ph));
                    board.set(b);
                    players.set(pl);
                    deck_left.set(dl);
                }
                // TODO: highlight the found set.
                Ok(ServerMsg::SetFound { .. }) => {}
                Ok(ServerMsg::Rejected { reason }) => {
                    error.set(Some(reason));
                }
                Err(e) => leptos::logging::error!("bad server message: {e}"),
            }
        }
    });

    // Whether this player is ready, read from the latest State so the server stays the source of truth.
    let me_ready = move || {
        let Some(id) = player_id.get() else {
            return false;
        };
        players.with(|ps| ps.iter().any(|p| p.id == id && p.ready))
    };
    let in_lobby = move || matches!(phase.get(), Some(Phase::Lobby | Phase::Countdown));
    let toggle_card = move |id: u8| {
        selected_cards.update(|sel| {
            if sel.contains(&id) {
                sel.retain(|&c| c != id);
            } else {
                sel.push(id);
            }

            if let Ok(cards) = <[u8; 3]>::try_from(sel.as_slice()) {
                if is_set_ids(cards) {
                    send(ClientMsg::SubmitSet { cards });
                }
                sel.clear();
            }

            log!("{:?}", sel);
        })
    };

    view! {
        <div class="game">
            <GameHeader code=header_code />
            <div class="main">
                <ul class="players">
                    {move || players.get().into_iter().filter(|pi| pi.connected).map(|pi| {  let playing = matches!(phase.get(), Some(Phase::Playing | Phase::Finished)); view! {
                        <li class="player" class:ready=pi.ready class:offline=!pi.connected>
                            <span class="name">{pi.name.clone()}</span>
                            <span class="status">
                                {if playing { pi.score.to_string() } else if pi.ready { "Ready".to_string() } else { "Waiting".to_string() }}
                            </span>

                        </li>
                    }}).collect_view()}
                </ul>
                <div class="play-area">
                    <Show when=in_lobby>
                        <button on:click=move |_| send(ClientMsg::SetReady { ready: !me_ready() })>
                            {move || if me_ready() { "Not ready" } else { "Ready" }}
                        </button>
                    </Show>
                    <Show when=move || phase.get() == Some(Phase::Countdown)>
                        <p>"Everyone is ready. Starting..."</p>
                    </Show>
                    <Show when=move || phase.get() == Some(Phase::Playing)>
                        <div class="board">
                            {move || board.get().iter()
                                .map(|&id| view! { <button on:click=move |_| toggle_card(id) class:selected=move || selected_cards.with(|s| s.contains(&id))><CardFace card=Card::from_id(id) /></button> })
                                .collect_view()}
                        </div>
                    </Show>
                </div>
                // Third grid column, on the right of the board.
                <div class="side-info">
                    <Show when=move || phase.get() == Some(Phase::Playing)>
                        <p class="deck-left">{move || format!("{} cards left", deck_left.get())}</p>
                    </Show>
                </div>
            </div>
        </div>
    }
}
