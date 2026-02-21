# Murmur

A serverless, decentralised social discussion protocol where the network breathes with attention.

Murmur is a fully P2P social network where your device is your identity, your archive, and a node in the network. Posts spread as far as interest carries them — hot content is widely replicated, cold content lives only on the author's device. Like a conversation at a party: the loudest and most interesting discussions are heard by everyone, while quiet exchanges fade naturally.

## Architecture

- **murmur-core** — Core library containing all protocol logic (identity, events, storage, networking, temperature)
- **murmur-tui** — Terminal UI for Phase 1 prototyping
- **murmur-desktop** — Tauri v2 + SolidJS desktop GUI

## Quick Start

```bash
# Prerequisites
rustup update stable

# Build
cargo build

# Run a node (terminal 1)
MURMUR_DATA=./node-alice cargo run -p murmur-tui

# Run a second node on the same network (terminal 2)
MURMUR_DATA=./node-bob cargo run -p murmur-tui -- --port 9001
```

The two nodes will discover each other via mDNS on the local network. Posts made on one node will appear on the other.

## Desktop App

```bash
# Prerequisites (Linux)
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev

# Install frontend dependencies
cd murmur-desktop && npm install && cd ..

# Run in development mode
cd murmur-desktop && npm run tauri dev

# Build for production
cd murmur-desktop && npm run tauri build
```

## TUI Controls

| Key | Action |
|-----|--------|
| `n` | Compose a new post |
| `r` | Reply to selected post |
| `Enter` | View thread |
| `j`/`k` or `↑`/`↓` | Navigate timeline |
| `i` | View profile |
| `Esc` | Back / cancel |
| `:` or `/` | Command mode |
| `q` | Quit |

## Commands

| Command | Description |
|---------|-------------|
| `/post <msg>` | Publish a post |
| `/reply <msg>` | Reply to selected post |
| `/thread` | View thread for selected post |
| `/peers` | List connected peers |
| `/whoami` | Show your identity |
| `/stats` | Show database statistics |
| `/help` | Show help |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MURMUR_DATA` | `./murmur-data` | Data directory for identity and database |
| `MURMUR_PORT` | `9000` | TCP listen port |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

## Phase 1 MVP

The current implementation covers Phase 1: **two desktop nodes exchanging signed posts over a local network**.

### What works:
- Ed25519 keypair generation and local identity persistence
- Signed event creation and verification
- SQLite-backed event storage
- P2P networking via libp2p (mDNS discovery, gossipsub broadcast)
- TUI with timeline, thread view, and compose
- Reply threading with parent/root references

### What's next (Phase 2+):
- Kademlia DHT for wide-area peer discovery
- Temperature calculation and display
- Temperature-based replication
- Profiles, follows, blocks
- Discovery and "New Voices" feeds
- Storage budget and eviction
- Proof-of-work registration

## Protocol

See [PROTOCOL.md](PROTOCOL.md) for the full protocol specification.

## License

TBD
