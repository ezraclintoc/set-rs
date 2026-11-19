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

            indexes.sort_unstable_by(|a, b| b.cmp(a));

            indexes.iter().for_each(|&i| {
                self.board.remove(i);
            });

            // TODO make dealt cards fill the removed cards
            self.deal();

            self.players.iter_mut().for_each(|pi| {
                if pi.id == player_id {
                    pi.score += 1;
                }
            });

            true
        }

        pub fn deal(&mut self) {
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

        pub fn join(&mut self, name: String, token: String) -> u32 {
            self.next_player_id += 1;
            self.players.push(PlayerInfo {
                id: self.next_player_id,
                name,
                score: 0,
                ready: false,
                connected: true,
            });
            self.tokens.insert(token, self.next_player_id);
            self.next_player_id
        }

        pub fn reconnect(&mut self, token: &str) -> Option<u32> {
            let id = *self.tokens.get(token)?;
            if let Some(player) = self.players.iter_mut().find(|p| p.id == id) {
                player.connected = true;
            }
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

        pub fn set_ready(&mut self, player_id: u32, ready: bool) -> Result<Phase, String> {
            if self.phase != Phase::Lobby && self.phase != Phase::Countdown {
                return Err("Cannot ready. Not in lobby.".into());
            }

            self.players.iter_mut().for_each(|pi| {
                if pi.id == player_id {
                    pi.ready = ready;
                }
            });
            self.check_ready();
            Ok(self.phase)
        }

        fn check_ready(&mut self) {
            if self.players.iter().all(|pi| !pi.connected || pi.ready) {
                if self.phase == Phase::Lobby {
                    self.phase = Phase::Countdown;
                }
            } else if self.phase == Phase::Countdown {
                self.phase = Phase::Lobby;
            }
        }

        // TODO: return the phase so the worker can set or delete the countdown alarm.
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
mod tests {
    use super::*;

    #[test]
    fn reconnect_with_known_token_keeps_seat() {
        let mut game = Game::new(0);
        let id = game.join("Ezra".into(), "tok".into());
        game.players[0].connected = false;

        assert_eq!(game.reconnect("tok"), Some(id));
        assert_eq!(game.players.len(), 1);
        assert!(game.players[0].connected);
    }

    #[test]
    fn reconnect_with_unknown_token_finds_nothing() {
        let mut game = Game::new(0);
        game.join("Ezra".into(), "tok".into());

        assert_eq!(game.reconnect("made-up"), None);
        assert_eq!(game.players.len(), 1);
    }

    fn started_game(seed: u64) -> Game {
        let mut game = Game::new(seed);
        game.phase = Phase::Countdown;
        game.start();
        game
    }

    #[test]
    fn find_set_returns_a_valid_set() {
        // 0, 1, 2 differ only in fill (solid, striped, empty), so they form a set.
        let mut game = Game::new(0);
        game.board = vec![0, 4, 1, 2];

        let found = game.find_set().expect("board contains a set");
        assert!(crate::card::is_set_ids(found));
    }

    #[test]
    fn find_set_returns_none_without_a_set() {
        let mut game = Game::new(0);
        assert_eq!(game.find_set(), None);

        // Two attributes vary over only two values each, so no three cards can form a set.
        game.board = vec![0, 1, 3, 4];
        assert_eq!(game.find_set(), None);
    }

    #[test]
    fn start_deals_a_board_with_a_set() {
        for seed in 0..100 {
            let game = started_game(seed);
            assert_eq!(game.phase, Phase::Playing);
            assert!(game.board.len() >= 12);
            assert!(game.find_set().is_some(), "seed {seed}");
        }
    }

    #[test]
    fn full_game_reaches_finished() {
        for seed in 0..100 {
            let mut game = started_game(seed);
            let id = game.join("Ezra".into(), "tok".into());
            let mut sets = 0;

            while game.phase == Phase::Playing {
                let set = game.find_set().expect("playing board always has a set");
                assert!(game.submit_set(set, id), "seed {seed}");
                sets += 1;
                assert!(sets <= 27, "seed {seed}: more sets than the deck allows");
            }

            assert_eq!(game.phase, Phase::Finished, "seed {seed}");
            let crate::ServerMsg::State { deck_left, .. } = game.state_msg() else {
                unreachable!()
            };
            assert_eq!(deck_left, 0, "seed {seed}");
            assert_eq!(game.find_set(), None, "seed {seed}");
            assert_eq!(game.board.len() + 3 * sets, 81, "seed {seed}: cards lost");
            assert_eq!(game.players[0].score as usize, sets, "seed {seed}");
        }
    }
}
