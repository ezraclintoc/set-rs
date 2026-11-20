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
    use crate::{ServerMsg, card::missing_id};

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
            if self.phase != Phase::Countdown {
                return;
            }
            self.phase = Phase::Playing;
            self.deal();
        }

        pub fn submit_set(&mut self, cards: [u8; 3], player_id: u32) -> bool {
            if self.phase != Phase::Playing {
                return false;
            }

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

            self.deal_in_place(&mut indexes);
            self.deal();

            self.players.iter_mut().for_each(|pi| {
                if pi.id == player_id {
                    pi.score += 1;
                }
            });

            true
        }

        fn deal_in_place(&mut self, indexes: &mut [usize]) {
            if self.deck.len() >= 3 && self.board.len() == 12 {
                indexes.iter().for_each(|&i| {
                    self.board[i] = self.deck.pop().unwrap();
                });
            } else {
                indexes.sort_unstable();
                indexes.iter().rev().for_each(|&i| {
                    self.board.remove(i);
                });
            }
        }

        fn deal(&mut self) {
            while self.board.len() < 12 && self.deck.len() >= 3 {
                self.deal_three();
            }
            while self.find_set().is_none() {
                if self.deck.len() >= 3 {
                    self.deal_three();
                } else {
                    self.phase = Phase::Finished;
                    break;
                }
            }
        }

        fn deal_three(&mut self) {
            let n = self.deck.len();
            self.board.extend(self.deck.drain(n - 3..));
        }

        pub fn join(&mut self, name: String, token: String) -> Result<u32, &'static str> {
            let name: String = name.trim().into();
            if name.chars().count() <= 3 {
                return Err("Your name is too short");
            }
            if name.chars().count() >= 20 {
                return Err("Your name is too long");
            }
            if self
                .players
                .iter()
                .any(|pi| pi.name.eq_ignore_ascii_case(&name))
            {
                return Err("Someone already has that name");
            }

            self.next_player_id += 1;
            self.players.push(PlayerInfo {
                id: self.next_player_id,
                name,
                score: 0,
                ready: false,
                connected: true,
            });
            self.tokens.insert(token, self.next_player_id);
            self.check_ready();
            Ok(self.next_player_id)
        }

        pub fn reconnect(&mut self, token: &str) -> Option<u32> {
            let id = *self.tokens.get(token)?;
            if let Some(player) = self.players.iter_mut().find(|p| p.id == id) {
                player.connected = true;
            }
            self.check_ready();
            Some(id)
        }

        pub fn state_msg(&self) -> ServerMsg {
            ServerMsg::State {
                phase: self.phase,
                board: self.board.clone(),
                players: self.players.clone(),
                deck_left: self.deck.len() as u8,
            }
        }

        pub fn set_ready(&mut self, player_id: u32, ready: bool) -> Result<(), String> {
            if self.phase != Phase::Lobby && self.phase != Phase::Countdown {
                return Err("Cannot ready. Not in lobby.".into());
            }

            // TODO use find to in case there are no players with id
            self.players.iter_mut().for_each(|pi| {
                if pi.id == player_id {
                    pi.ready = ready;
                }
            });

            self.check_ready();
            Ok(())
        }

        fn check_ready(&mut self) {
            let anyone_connected = self.players.iter().any(|pi| pi.connected);
            if anyone_connected && self.players.iter().all(|pi| !pi.connected || pi.ready) {
                if self.phase == Phase::Lobby {
                    self.phase = Phase::Countdown;
                }
            } else if self.phase == Phase::Countdown {
                self.phase = Phase::Lobby;
            }
        }

        pub fn disconnect(&mut self, player_id: u32) {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                player.connected = false;
            }
            self.check_ready();
        }

        pub fn find_set(&self) -> Option<[u8; 3]> {
            let mut on_board = [false; 81];
            for &card in &self.board {
                on_board[card as usize] = true;
            }

            for (i, &a) in self.board.iter().enumerate() {
                for &b in &self.board[i + 1..] {
                    let c = missing_id(a, b);
                    if on_board[c as usize] {
                        return Some([a, b, c]);
                    }
                }
            }
            None
        }
    }
}

#[cfg(feature = "server")]
pub use server::*;

#[cfg(all(test, feature = "server"))]
mod tests;
