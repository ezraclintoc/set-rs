use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: u32,
    pub name: String,
    pub score: u32,
    pub ready: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Lobby,
    Countdown,
    Playing,
    Finished,
}

#[cfg(feature = "server")]
mod server {
    use super::*;
    use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    pub struct Game {
        pub phase: Phase,
        pub players: Vec<PlayerInfo>,
        tokens: HashMap<String, u32>,
        pub board: Vec<u8>,
        deck: Vec<u8>,
        next_player_id: u32,
    }

    impl Game {
        pub fn new(seed: u64) -> Self {
            let mut rng = StdRng::seed_from_u64(seed);
            let mut deck: Vec<u8> = (0..81).collect();
            deck.shuffle(&mut rng);

            Self {
                phase: Phase::Lobby,
                players: Vec::new(),
                tokens: HashMap::new(),
                board: Vec::new(),
                deck,
                next_player_id: 0,
            }
        }

        pub fn start(&mut self) {
            self.phase = Phase::Playing;
            self.deal();
        }

        pub fn deal(&mut self) {
            if self.deck.len() >= 3 {
                while self.board.len() < 12 {
                    self.deal_three();
                }
                // TODO add no set checking
            }
        }

        fn deal_three(&mut self) {
            let n = self.deck.len();
            self.board.extend(self.deck.drain(n - 3..));
        }
    }
}

#[cfg(feature = "server")]
pub use server::*;
