mod helper;

use crate::helper::*;

use shared::game::{Game, Phase};
use shared::{ClientMsg, ServerMsg};
use worker::*;

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
                                (game.join(name, token.clone()), token)
                            }
                        };

                        self.save_game(&game).await?;

                        ws.serialize_attachment(player_id)?;

                        ws.send_with_str(serde_json::to_string(&ServerMsg::Welcome {
                            player_id,
                            token,
                        })?)?;
                    }
                    Ok(ClientMsg::SetReady { ready }) => {
                        let mut game = self.get_game().await?;
                        let Some(player_id) = ws.deserialize_attachment::<u32>()? else {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Invalid Player ID".into(),
                            })?)?;
                            return Ok(());
                        };

                        let Ok(phase) = game.set_ready(player_id, ready) else {
                            ws.send_with_str(serde_json::to_string(&ServerMsg::Rejected {
                                reason: "Unable To Ready. Not In Lobby.".into(),
                            })?)?;
                            return Ok(());
                        };

                        self.save_game(&game).await?;

                        if phase == Phase::Lobby
                            && self.state.storage().get_alarm().await?.is_some()
                        {
                            self.state.storage().delete_alarm().await?;
                        } else if phase == Phase::Countdown
                            && self.state.storage().get_alarm().await?.is_none()
                        {
                            self.state
                                .storage()
                                .set_alarm(std::time::Duration::from_secs(5))
                                .await?;
                        }

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
                    Err(_) => {}
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

        // The same player can be connected from another tab. Only mark them
        // disconnected when no other socket still belongs to them.
        let still_connected = self.state.get_websockets().iter().any(|w| {
            *w != ws && matches!(w.deserialize_attachment::<u32>(), Ok(Some(id)) if id == player_id)
        });
        if still_connected {
            return Ok(());
        }

        let mut game = self.get_game().await?;

        // TODO: disconnect can start the countdown (the last unready player left), but the
        // alarm is never set here. Return the phase from `disconnect` and set or delete the
        // alarm like `SetReady` does, ideally through a shared helper.
        game.disconnect(player_id);
        self.save_game(&game).await?;
        let _ = ws.close(Some(code as u16), Some(reason));

        let json = serde_json::to_string(&game.state_msg())?;
        for w in self.state.get_websockets() {
            let _ = w.send_with_str(json.clone());
        }

        Ok(())
    }

    async fn alarm(&self) -> Result<Response> {
        let mut game = self.get_game().await?;
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
            .unwrap_or_else(|| Game::new(Date::now().as_millis())))
    }

    async fn save_game(&self, game: &Game) -> Result<(), worker::Error> {
        self.state.storage().put("game", game).await
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/api/game/:id/ws", |req, ctx| async move {
            let id = match ctx.param("id") {
                Some(n) => n.clone(),
                None => return Response::error("missing id", 400),
            };
            let namespace = ctx.durable_object("SETROOM")?;
            let stub = namespace.id_from_name(&id)?.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .run(req, env)
        .await
}
