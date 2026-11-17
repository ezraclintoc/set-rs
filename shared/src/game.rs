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
    use crate::card::is_set_ids;

    use super::*;
    use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
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

        pub fn submit_set(&mut self, cards: [u8; 3]) -> bool {
            let [a, b, c] = cards;
            if a == b || b == c || a == c {
                return false;
            }

            let Some(mut indexes) = cards
                .iter()
                .map(|c| self.board.iter().position(|b| b == c))
                .collect::<Option<Vec<usize>>>()
            else {
                return false;
            };

            if !is_set_ids(cards) {
                return false;
            }

            indexes.sort_unstable_by(|a, b| b.cmp(a));

            indexes.iter().for_each(|&i| {
                self.board.remove(i);
            });

            // TODO make dealt cards fill the removed cards
            self.deal();

            true
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
