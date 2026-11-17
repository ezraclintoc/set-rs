use worker::*;

#[durable_object]
pub struct Game {
    state: State,
    env: Env,
}

impl DurableObject for Game {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        Response::ok("game room")
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let ns = env.durable_object("GAME")?;
    let stub = ns.id_from_name("room-1")?.get_stub()?;
    stub.fetch_with_request(req).await
}