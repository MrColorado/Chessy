# chess

A UCI chess engine written in Rust, with a Lichess bot integration.

Binaries:
- `chess-uci` — UCI engine, usable in any compatible GUI (CuteChess, Arena, Banksia…)
- `chess-lichess` — Lichess bot client that streams events and plays games

## Features

What the engine currently implements.

### Board representation
- **Bitboards** — one `u64` per (piece type, color), 12 in total.
- A redundant **mailbox** array (`[u8; 64]`) kept in sync for fast "what's on this square?" lookups.
- Full game state: side to move, castling rights, en-passant square, halfmove clock, fullmove number.
- FEN parsing (`Board::from_fen`) and the standard starting position (`Board::new`).
- `make_move` / `unmake_move` with an `IrreversibleState` undo record (no board copying in search).

### Move generation
- **Legal** move generation (not pseudo-legal): generated moves are guaranteed legal.
- **Magic bitboards** for sliding pieces (rook / bishop / queen), with attack tables built at startup.
- Precomputed attack tables for knights and kings, ray tables for sliders.
- Handles all special moves: castling, en passant, promotions, double pawn pushes.
- Validated by a **perft** test suite (startpos, kiwipete, and other standard positions).

### Search
- **Negamax** with **alpha-beta** pruning.
- **Iterative deepening** from the root.
- **Quiescence search** at the leaves (captures only) with **delta pruning** to tame tactical horizons.
- **Move ordering**, which is what makes alpha-beta effective:
  - **TT move** first (the best move remembered for this exact position),
  - then captures by **MVV-LVA** (most valuable victim, least valuable attacker),
  - then **killer moves** (two per ply — quiet moves that caused a beta cutoff at the same depth).
- Draw detection for the **50-move rule**; mate scores are distance-to-mate aware (shorter mates preferred).

### Transposition table
- **Zobrist hashing**: a 64-bit position fingerprint, updated **incrementally** on every `make_move` /
  `unmake_move` (keys generated at compile time). A debug assertion checks the incremental hash against a
  from-scratch recomputation on every move.
- A fixed-size **transposition table** (16 MB, power-of-two, always-replace) caching each node's depth,
  score, bound type (exact / lower / upper) and best move.
- Probed at the top of each node (for cutoffs and move ordering) and stored on exit.

### Evaluation
- **Material** counting in centipawns.
- **Piece-square tables** (Michniewski PST) for positional bonuses.
- Maintained **incrementally** during search (`incremental_evaluate`) rather than recomputed each node.

### Interfaces
- **UCI** protocol (`chess-uci`) — drop-in engine for any UCI GUI.
- **Lichess bot** (`chess-lichess`) — streams events and plays real-time games.

### Benchmarking
- `cargo bench` runs Criterion micro-benchmarks for move generation, perft, and search.
- `scripts/bench.sh` runs a gauntlet against Stockfish (bridged to several Elo levels) via
  [fastchess](https://github.com/Disservin/fastchess) to estimate playing strength.

## Project layout

```
src/
  engine/
    board.rs       board state, make/unmake, FEN, Zobrist hashing, evaluation
    movegen/       legal move generation (magic bitboards for sliders)
    mv.rs          Move encoding + move list with ordering/scoring
    piece.rs       piece/color types, values, piece-square tables
    searcher.rs    negamax + alpha-beta, iterative deepening, quiescence, TT integration
    tt.rs          transposition table
    zobrist.rs     Zobrist keys (compile-time) + from-scratch hash
    square.rs      square/coordinate helpers
    rule.rs        check detection, perft
  uci.rs           UCI protocol
  lichess.rs       Lichess bot client
  bin/             uci.rs, lichess.rs entry points
```

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