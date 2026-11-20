use super::*;

fn started_game(seed: u64) -> Game {
    let mut game = Game::new(seed);
    game.phase = Phase::Countdown;
    game.start();
    game
}

fn deck_left(game: &Game) -> u8 {
    let crate::ServerMsg::State { deck_left, .. } = game.state_msg() else {
        unreachable!()
    };
    deck_left
}

#[test]
fn reconnect_with_known_token_keeps_seat() {
    let mut game = Game::new(0);
    let id = game.join("Ezra".into(), "tok".into()).unwrap();
    game.players[0].connected = false;

    assert_eq!(game.reconnect("tok"), Some(id));
    assert_eq!(game.players.len(), 1);
    assert!(game.players[0].connected);
}

#[test]
fn reconnect_with_unknown_token_finds_nothing() {
    let mut game = Game::new(0);
    game.join("Ezra".into(), "tok".into()).unwrap();

    assert_eq!(game.reconnect("made-up"), None);
    assert_eq!(game.players.len(), 1);
}

#[test]
fn last_unready_player_leaving_starts_countdown() {
    let mut game = Game::new(0);
    let a = game.join("Alice".into(), "a".into()).unwrap();
    let b = game.join("Bobby".into(), "b".into()).unwrap();
    game.set_ready(a, true).unwrap();
    assert_eq!(game.phase, Phase::Lobby);

    game.disconnect(b);
    assert_eq!(game.phase, Phase::Countdown);
}

#[test]
fn everyone_leaving_cancels_countdown() {
    let mut game = Game::new(0);
    let a = game.join("Alice".into(), "a".into()).unwrap();
    game.set_ready(a, true).unwrap();
    assert_eq!(game.phase, Phase::Countdown);

    game.disconnect(a);
    assert_eq!(game.phase, Phase::Lobby);
}

#[test]
fn joining_during_countdown_cancels_it() {
    let mut game = Game::new(0);
    let a = game.join("Alice".into(), "a".into()).unwrap();
    game.set_ready(a, true).unwrap();
    assert_eq!(game.phase, Phase::Countdown);

    game.join("Bobby".into(), "b".into()).unwrap();
    assert_eq!(game.phase, Phase::Lobby);
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
fn claim_on_twelve_card_board_replaces_in_place() {
    for seed in 0..100 {
        let mut game = started_game(seed);
        if game.board.len() != 12 {
            continue;
        }
        let id = game.join("Ezra".into(), "tok".into()).unwrap();
        let before = game.board.clone();
        let set = game.find_set().unwrap();

        assert!(game.submit_set(set, id), "seed {seed}");

        assert!(game.board.len() >= 12, "seed {seed}");
        for (i, &card) in before.iter().enumerate() {
            if set.contains(&card) {
                assert!(
                    !before.contains(&game.board[i]),
                    "seed {seed}: slot {i} not refilled"
                );
            } else {
                assert_eq!(game.board[i], card, "seed {seed}: slot {i} moved");
            }
        }
    }
}

#[test]
fn claim_on_fifteen_card_board_shrinks_to_twelve() {
    let mut game = Game::new(0);
    let id = game.join("Ezra".into(), "tok".into()).unwrap();
    game.phase = Phase::Playing;
    // 0, 1, 2 form a set, and so do 3, 4, 5, so the shrunk board still has one.
    game.board = (0..15).collect();
    let deck_before = deck_left(&game);

    assert!(game.submit_set([0, 1, 2], id));

    assert_eq!(game.board, (3..15).collect::<Vec<u8>>());
    assert_eq!(deck_left(&game), deck_before);
}

#[test]
fn full_game_reaches_finished() {
    for seed in 0..100 {
        let mut game = started_game(seed);
        let id = game.join("Ezra".into(), "tok".into()).unwrap();
        let mut sets = 0;

        while game.phase == Phase::Playing {
            let set = game.find_set().expect("playing board always has a set");
            assert!(game.submit_set(set, id), "seed {seed}");
            sets += 1;
            assert!(sets <= 27, "seed {seed}: more sets than the deck allows");
        }

        assert_eq!(game.phase, Phase::Finished, "seed {seed}");
        assert_eq!(deck_left(&game), 0, "seed {seed}");
        assert_eq!(game.find_set(), None, "seed {seed}");
        assert_eq!(game.board.len() + 3 * sets, 81, "seed {seed}: cards lost");
        assert_eq!(game.players[0].score as usize, sets, "seed {seed}");
    }
}
