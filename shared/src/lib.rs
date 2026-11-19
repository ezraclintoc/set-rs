pub mod card;
pub mod game;

use crate::game::{Phase, PlayerInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Join { name: String, token: Option<String> },
    SubmitSet { cards: [u8; 3] },
    SetReady { ready: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Welcome {
        player_id: u32,
        token: String,
    },
    State {
        phase: Phase,
        board: Vec<u8>,
        players: Vec<PlayerInfo>,
        deck_left: u8,
    },
    SetFound {
        player_id: u32,
        cards: [u8; 3],
    },
    Rejected {
        reason: String,
    },
}
