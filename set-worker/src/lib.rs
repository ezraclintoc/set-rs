mod helper;

use crate::helper::*;

use shared::game::Game;
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
                        let mut game = self
                            .state
                            .storage()
                            .get::<Game>("game")
                            .await?
                            .unwrap_or_else(|| Game::new(Date::now().as_millis()));

                        let token = token.unwrap_or_else(|| new_token().unwrap());
                        let uid = game.join(name, token.clone());

                        self.state.storage().put("game", game).await?;

                        let json = serde_json::to_string(&ServerMsg::Welcome {
                            player_id: uid,
                            token,
                        })?;
                        ws.send_with_str(json)?
                    }
                    Ok(ClientMsg::SetReady { ready: _ }) => {}
                    Ok(ClientMsg::SubmitSet { cards: _ }) => {}
                    Err(_) => {}
                }
            }
            WebSocketIncomingMessage::Binary(_) => ws.send_with_str("binary not supported")?,
        }
        Ok(())
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // TODO add router
    let ns = env.durable_object("GAME")?;
    let stub = ns.id_from_name("room-1")?.get_stub()?;
    stub.fetch_with_request(req).await
}
