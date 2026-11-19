# set-rs

A realtime multiplayer version of the card game SET, written entirely in Rust and running on Cloudflare Workers with Durable Objects.

Players join a room, ready up, and race to find sets on a shared board. Every room is a single Durable Object that owns the game state and settles who claimed a set first.

> **Status:** early development. The shared game logic is in progress; the server and frontend have not been started yet.

## How to play

Every card has four attributes, each with three possible values:

| Attribute | Values |
| --- | --- |
| Shape | Diamond, Oval, Squiggle |
| Count | One, Two, Three |
| Color | Red, Purple, Green |
| Fill | Solid, Striped, Empty |

A **set** is three cards where, for each attribute, the values are either all the same or all different. Twelve cards are dealt face up, and the first player to spot a set and claim it scores a point. If no set exists on the board, three more cards are dealt. The game ends when the deck is empty and no set remains.

## Why Durable Objects

The interesting part of SET as a multiplayer game is the race: several players can claim overlapping sets within milliseconds of each other. A Durable Object processes one event at a time, so the first valid claim wins and any later claim on the same cards is rejected cleanly, without locks or database transactions.

Each room is one Durable Object, addressed by its room code with `id_from_name(code)`. The object:

* holds the deck, board, players, scores, and game phase
* accepts player WebSockets through the Hibernation API, so idle rooms cost nothing
* saves the full game state to storage after every change, so nothing is lost when it hibernates
* uses alarms for the ready countdown

## Architecture

```
set-rs/
├── Cargo.toml        # workspace
├── flake.nix         # dev shell (Rust + wasm target, wrangler, trunk)
├── shared/           # card types, rules, and the message protocol
├── worker/           # Cloudflare Worker and the Game Durable Object
└── frontend/         # Leptos client, built with trunk
```

### `shared`

Compiled into both the worker and the frontend, so the client and server can never disagree about the rules or the protocol.

* `Card` and its attribute enums (`Shape`, `Count`, `Color`, `Fill`)
* Card IDs: each of the 81 cards is a `u8`, encoded as `shape*27 + count*9 + color*3 + fill`, so the network only ever sends small integers and `Card::from_id` rebuilds the full card
* `is_set`: an attribute is valid when its three values sum to a multiple of 3, which is true exactly when they are all equal or all different
* `ClientMsg` and `ServerMsg`, the WebSocket protocol
* `Game` (behind the `server` feature): the authoritative game state and rules. The frontend does not compile it.

Keeping `Game` as a plain struct means full games can be tested with `cargo test` natively, and the Durable Object stays a thin layer that loads state, calls a method, saves, and broadcasts.

### Protocol

Messages are JSON, tagged by a `type` field.

**Client to server**

| Message | Purpose |
| --- | --- |
| `Join { name, token }` | Join a room, or rejoin with a saved token |
| `SetReady { ready }` | Toggle ready in the lobby |
| `SubmitSet { cards }` | Claim a set by **card ID** |

**Server to client**

| Message | Purpose |
| --- | --- |
| `Welcome { player_id, token }` | Sent once on join; the token is kept in localStorage for reconnects |
| `State { ... }` | Full snapshot of the room: phase, board, players, cards left in the deck |
| `Countdown { seconds }` | Everyone is ready and the game is about to start |
| `SetFound { player_id, cards }` | Someone scored; the client highlights the set before the next `State` |
| `Rejected { reason }` | An invalid or stale claim |

Claims use card IDs instead of board positions because the board changes as sets are replaced. A claim made against an outdated board simply fails, since the cards are no longer on the table.

### Game flow

1. Players join the lobby and mark themselves ready.
2. When every connected player is ready, the room starts a short countdown. Anyone unreadying or joining cancels it.
3. Twelve cards are dealt, plus three more at a time while no set exists.
4. A valid claim scores a point. Replacement cards go into the same slots so the board does not reshuffle, unless the board is above 12 cards, in which case the set is just removed.
5. The game ends when the deck is empty and no set remains.

### Rendering

Cards are drawn as inline SVG generated from their attributes, with no image assets. Each shape is one reusable path, repeated by count, colored by stroke and fill, and striped with an SVG pattern.

## Development

The dev environment is a Nix flake.

```bash
nix develop

# run the tests for the game logic
cargo test -p shared --features server

# run the worker locally
cd worker && wrangler dev

# run the frontend
cd frontend && trunk serve
```

## Roadmap

### Shared game logic

* [x] Card attribute enums and `Card`
* [x] Card ID encoding with `Card::from_id` and `Card::id`
* [x] `is_set`
* [x] Client and server message types
* [x] `Game::new` and `deal` (in progress)
* [ ] `find_set`
* [ ] `join`, `set_ready`, and `claim`
* [ ] In place card replacement after a set is found
* [ ] Game over detection
* [ ] Tests for card ID round trips, `is_set`, `find_set`, and full simulated games

### Worker

* [ ] Route `/room/:code` to a Game Durable Object and upgrade to a WebSocket
* [ ] Accept sockets with the Hibernation API, with the player ID stored as a socket attachment
* [ ] Persist and restore `Game` from Durable Object storage
* [ ] Join, reconnect by token, and ready handling
* [ ] Countdown alarm and game start
* [ ] Set claims and broadcasting
* [ ] Validate all incoming card IDs before use
* [ ] Serve the built frontend as static assets

### Frontend

* [ ] Leptos app with a join screen and room codes
* [ ] SVG card component
* [ ] Lobby with ready toggles and countdown
* [ ] Board, card selection, and claim submission
* [ ] Instant local feedback for invalid picks using `is_set`
* [ ] Scoreboard, set found animation, and game over screen
* [ ] Reconnect using the saved token

### Planned features

* [ ] **Public rooms:** a Lobby Durable Object that rooms register with, a browsable room list, and host controls such as kick
* [ ] **Solo practice:** play alone against a timer
* [ ] **Hint button:** highlight one card from a set found by `find_set`

## Disclaimer

SET is a trademark of SET Enterprises, Inc. This is an unofficial, noncommercial project made for learning and is not affiliated with or endorsed by SET Enterprises.
