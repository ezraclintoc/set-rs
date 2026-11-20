mod helper;

use crate::helper::*;

use shared::game::{Game, Phase};
use shared::{ClientMsg, ServerMsg};
use worker::*;

const COUNTDOWN: std::time::Duration = std::time::Duration::from_secs(5);
const DELETE: std::time::Duration = std::time::Duration::from_mins(3);

#[expect(dead_code)]
#[durable_object]
pub struct SetRoom {
    state: State,
    env: Env,
}

impl DurableObject for SetRoom {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let upgrade = req.headers().get("Upgrade")?;
        if upgrade.as_deref() != Some("websocket") {
            return Response::error("Expected WebSocket upgrade", 426);
        }

        let pair = WebSocketPair::new()?;

        self.state.accept_web_socket(&pair.server);

        Response::from_websocket(pair.client)
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        match message {
            WebSocketIncomingMessage::String(text) => {
                match serde_json::from_str::<ClientMsg>(&text) {
                    Ok(ClientMsg::Join { name, token }) => {
                        if ws.deserialize_attachment::<u32>()?.is_some() {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Already joined".into(),
                            })?)?;
                            return Ok(());
                        }

                        let mut game = self.get_game().await?;

                        let reconnected = token.and_then(|t| game.reconnect(&t).map(|id| (id, t)));
                        let (player_id, token) = match reconnected {
                            Some(seat) => seat,
                            None => {
                                let token = new_token()?;
                                match game.join(name, token.clone()) {
                                    Ok(id) => (id, token),
                                    Err(reason) => {
                                        ws.send_with_str(serde_json::to_string(
                                            &ServerMsg::Rejected {
                                                reason: reason.into(),
                                            },
                                        )?)?;
                                        return Ok(());
                                    }
                                }
                            }
                        };

                        self.save_game(&game).await?;
                        self.sync_alarm(&game).await?;

                        ws.serialize_attachment(player_id)?;

                        ws.send_with_str(serde_json::to_string(&ServerMsg::Welcome {
                            player_id,
                            token,
                        })?)?;

                        let json = serde_json::to_string(&game.state_msg())?;
                        for w in self.state.get_websockets() {
                            let _ = w.send_with_str(&json);
                        }
                    }
                    Ok(ClientMsg::SetReady { ready }) => {
                        let mut game = self.get_game().await?;
                        let Some(player_id) = ws.deserialize_attachment::<u32>()? else {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Invalid Player ID".into(),
                            })?)?;
                            return Ok(());
                        };

                        if game.set_ready(player_id, ready).is_err() {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Unable To Ready. Not In Lobby.".into(),
                            })?)?;
                            return Ok(());
                        }

                        self.save_game(&game).await?;

                        self.sync_alarm(&game).await?;

                        // TODO: replace the broadcast loops with a shared `broadcast` helper.
                        let json = serde_json::to_string(&game.state_msg())?;
                        for w in self.state.get_websockets() {
                            let _ = w.send_with_str(json.clone());
                        }
                    }
                    Ok(ClientMsg::SubmitSet { cards }) => {
                        let mut game = self.get_game().await?;
                        let Some(player_id) = ws.deserialize_attachment::<u32>()? else {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Invalid Player ID".into(),
                            })?)?;
                            return Ok(());
                        };

                        let success = game.submit_set(cards, player_id);
                        if !success {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Invalid Set".into(),
                            })?)?;
                            return Ok(());
                        }

                        self.save_game(&game).await?;

                        let sf_json =
                            serde_json::to_string(&ServerMsg::SetFound { player_id, cards })?;

                        let s_json = serde_json::to_string(&game.state_msg())?;

                        for w in self.state.get_websockets() {
                            let _ = w.send_with_str(sf_json.clone());
                            let _ = w.send_with_str(s_json.clone());
                        }
                    }
                    Err(_) => ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                        reason: "Bad JSON".into(),
                    })?)?,
                }
            }
            WebSocketIncomingMessage::Binary(_) => {
                ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                    reason: "Binary Not Supported".into(),
                })?)?
            }
        }
        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        code: usize,
        reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        let Some(player_id) = ws.deserialize_attachment::<u32>()? else {
            return Ok(());
        };

        let still_connected = self.state.get_websockets().iter().any(|w| {
            *w != ws && matches!(w.deserialize_attachment::<u32>(), Ok(Some(id)) if id == player_id)
        });
        if still_connected {
            return Ok(());
        }

        let mut game = self.get_game().await?;

        game.disconnect(player_id);
        self.save_game(&game).await?;
        self.sync_alarm(&game).await?;
        let _ = ws.close(Some(code as u16), Some(reason));

        let json = serde_json::to_string(&game.state_msg())?;
        for w in self.state.get_websockets() {
            let _ = w.send_with_str(json.clone());
        }

        Ok(())
    }

    async fn alarm(&self) -> Result<Response> {
        let mut game = self.get_game().await?;
        if !game.players.iter().any(|pi| pi.connected) {
            self.state.storage().delete_all().await?;
            return Response::ok("");
        }

        if game.phase != Phase::Countdown {
            return Response::ok("");
        }

        game.start();
        self.save_game(&game).await?;

        let json = serde_json::to_string(&game.state_msg())?;
        self.state.get_websockets().iter().for_each(|w| {
            let _ = w.send_with_str(json.clone());
        });

        Response::ok("")
    }
}

impl SetRoom {
    async fn get_game(&self) -> Result<Game, worker::Error> {
        Ok(self
            .state
            .storage()
            .get::<Game>("game")
            .await?
            .unwrap_or_else(|| Game::new(getrandom::u64().expect("getrandom should not fail"))))
    }

    async fn save_game(&self, game: &Game) -> Result<(), worker::Error> {
        self.state.storage().put("game", game).await
    }

    async fn sync_alarm(&self, game: &Game) -> Result<()> {
        let storage = self.state.storage();
        match game.phase {
            Phase::Countdown => {
                if storage.get_alarm().await?.is_none() {
                    storage.set_alarm(COUNTDOWN).await?;
                }
            }
            _ if !game.players.iter().any(|pi| pi.connected) => {
                storage.set_alarm(DELETE).await?;
            }
            _ if storage.get_alarm().await?.is_some() => {
                storage.delete_alarm().await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/api/game/:code/ws", |req, ctx| async move {
            let code = match ctx.param("code") {
                Some(n) => n.to_ascii_lowercase().clone(),
                None => return Response::error("missing code", 400),
            };

            if code.len() < 4 || code.len() > 8 {
                return Response::error("invalid code length", 400);
            }

            let namespace = ctx.durable_object("SETROOM")?;
            let stub = namespace.id_from_name(&code)?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .run(req, env)
        .await
}
