# chess

A UCI chess engine written in Rust, with a Lichess bot integration.

Binaries:
- `chess-uci` — UCI engine, usable in any compatible GUI (CuteChess, Arena, Banksia…)
- `chess-lichess` — Lichess bot client that streams events and plays games

## Installation

Download the latest binaries from the [Releases page](../../releases).

### Linux

```sh
chmod +x chess-uci-*-linux-x86_64
./chess-uci-*-linux-x86_64
```

### macOS

The macOS binaries are signed ad-hoc but not notarized by Apple, so Gatekeeper will block them on first run with a message like:

> Apple could not verify "chess-uci-…" is free of malware…

Remove the quarantine attribute after downloading:

```sh
xattr -d com.apple.quarantine chess-uci-*-macos-arm64
xattr -d com.apple.quarantine chess-lichess-*-macos-arm64
chmod +x chess-uci-*-macos-arm64 chess-lichess-*-macos-arm64
```

Then run normally:

```sh
./chess-uci-*-macos-arm64
```

## Build from source

Requires a stable Rust toolchain.

```sh
cargo build --release
./target/release/chess-uci
```

## Usage

### UCI engine

Point your chess GUI at the `chess-uci` binary as an engine.

### Lichess bot

```sh
LICHESS_TOKEN=<your-bot-token> ./chess-lichess
```