use std::io::{self, BufRead};
use crate::engine::{ChessEngine, SearchParams, SquareMove};
use crate::engine::piece::Piece;
use crate::engine::square::Square;

pub fn run(engine: &mut impl ChessEngine) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim();

        if line == "uci" {
            println!("id name Chessy");
            println!("id author MrColorado");
            println!("uciok");
        } else if line == "isready" {
            println!("readyok");
        } else if line == "ucinewgame" {
            engine.new_game();
        } else if line.starts_with("position") {
            handle_position(engine, line);
        } else if line.starts_with("go") {
            engine.start_search(parse_go(line));
        } else if line == "stop" {
            engine.stop();
        } else if line == "ponderhit" {
            engine.ponderhit();
        } else if line == "quit" {
            break;
        }
    }
}

fn handle_position(engine: &mut impl ChessEngine, line: &str) {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 2 {
        return;
    }

    let (fen, moves_idx) = match tokens[1] {
        "startpos" => (None, 2),
        "fen" => {
            let end = tokens[2..]
                .iter()
                .position(|&t| t == "moves")
                .map(|i| i + 2)
                .unwrap_or(tokens.len());
            (Some(tokens[2..end].join(" ")), end)
        }
        _ => return,
    };

    let moves_start = if moves_idx < tokens.len() && tokens[moves_idx] == "moves" {
        moves_idx + 1
    } else {
        tokens.len()
    };

    let moves: Vec<SquareMove> = tokens[moves_start..]
        .iter()
        .filter_map(|&s| parse_move(s))
        .collect();

    engine.set_position(fen.as_deref(), &moves);
}

fn parse_move(s: &str) -> Option<SquareMove> {
    if s.len() < 4 || s.len() > 5 {
        return None;
    }
    let from = Square::from_str(&s[0..2])?;
    let to = Square::from_str(&s[2..4])?;
    let promotion = if s.len() == 5 {
        Some(match s.as_bytes()[4] {
            b'q' => Piece::Queen,
            b'r' => Piece::Rook,
            b'b' => Piece::Bishop,
            b'n' => Piece::Knight,
            _ => return None,
        })
    } else {
        None
    };
    Some(SquareMove { from, to, promotion })
}

fn parse_go(line: &str) -> SearchParams {
    let mut params = SearchParams {
        depth: None,
        movetime: None,
        wtime: None,
        btime: None,
        winc: None,
        binc: None,
        infinite: false,
        ponder: false,
    };

    let mut tokens = line.split_whitespace();
    tokens.next(); // skip "go"

    while let Some(token) = tokens.next() {
        match token {
            "depth"    => params.depth    = tokens.next().and_then(|v| v.parse().ok()),
            "movetime" => params.movetime = tokens.next().and_then(|v| v.parse().ok()),
            "wtime"    => params.wtime    = tokens.next().and_then(|v| v.parse().ok()),
            "btime"    => params.btime    = tokens.next().and_then(|v| v.parse().ok()),
            "winc"     => params.winc     = tokens.next().and_then(|v| v.parse().ok()),
            "binc"     => params.binc     = tokens.next().and_then(|v| v.parse().ok()),
            "infinite" => params.infinite = true,
            "ponder"   => params.ponder   = true,
            _          => {}
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::square::Square;
    use crate::engine::piece::Piece;

    struct MockEngine {
        last_fen: Option<String>,
        last_moves: Vec<(Square, Square, Option<Piece>)>,
        last_search: Option<SearchParams>,
        new_game_called: bool,
        stop_called: bool,
        ponderhit_called: bool,
    }

    impl MockEngine {
        fn new() -> Self {
            MockEngine {
                last_fen: None,
                last_moves: Vec::new(),
                last_search: None,
                new_game_called: false,
                stop_called: false,
                ponderhit_called: false,
            }
        }
    }

    impl ChessEngine for MockEngine {
        fn new_game(&mut self) { self.new_game_called = true; }
        fn set_position(&mut self, fen: Option<&str>, moves: &[SquareMove]) {
            self.last_fen = fen.map(String::from);
            self.last_moves = moves.iter().map(|m| (m.from, m.to, m.promotion)).collect();
        }
        fn start_search(&mut self, params: SearchParams) { self.last_search = Some(params); }
        fn stop(&mut self) { self.stop_called = true; }
        fn ponderhit(&mut self) { self.ponderhit_called = true; }
    }

    // --- parse_move ---

    #[test]
    fn parse_move_quiet() {
        let mv = parse_move("e2e4").unwrap();
        assert_eq!(mv.from, Square::E2);
        assert_eq!(mv.to, Square::E4);
        assert_eq!(mv.promotion, None);
    }

    #[test]
    fn parse_move_promotion() {
        let mv = parse_move("e7e8q").unwrap();
        assert_eq!(mv.from, Square::E7);
        assert_eq!(mv.to, Square::E8);
        assert_eq!(mv.promotion, Some(Piece::Queen));
    }

    #[test]
    fn parse_move_all_promotion_pieces() {
        assert_eq!(parse_move("a7a8q").unwrap().promotion, Some(Piece::Queen));
        assert_eq!(parse_move("a7a8r").unwrap().promotion, Some(Piece::Rook));
        assert_eq!(parse_move("a7a8b").unwrap().promotion, Some(Piece::Bishop));
        assert_eq!(parse_move("a7a8n").unwrap().promotion, Some(Piece::Knight));
    }

    #[test]
    fn parse_move_invalid() {
        assert!(parse_move("e2").is_none());
        assert!(parse_move("e2e4e5f").is_none());
        assert!(parse_move("z9z9").is_none());
        assert!(parse_move("e7e8x").is_none());
    }

    // --- handle_position ---

    #[test]
    fn position_startpos() {
        let mut engine = MockEngine::new();
        handle_position(&mut engine, "position startpos");
        assert_eq!(engine.last_fen, None);
        assert!(engine.last_moves.is_empty());
    }

    #[test]
    fn position_startpos_with_moves() {
        let mut engine = MockEngine::new();
        handle_position(&mut engine, "position startpos moves e2e4 e7e5");
        assert_eq!(engine.last_fen, None);
        assert_eq!(engine.last_moves.len(), 2);
        assert_eq!(engine.last_moves[0], (Square::E2, Square::E4, None));
        assert_eq!(engine.last_moves[1], (Square::E7, Square::E5, None));
    }

    #[test]
    fn position_fen() {
        let mut engine = MockEngine::new();
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        handle_position(&mut engine, &format!("position fen {fen}"));
        assert_eq!(engine.last_fen.as_deref(), Some(fen));
        assert!(engine.last_moves.is_empty());
    }

    #[test]
    fn position_fen_with_moves() {
        let mut engine = MockEngine::new();
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        handle_position(&mut engine, &format!("position fen {fen} moves e7e5"));
        assert_eq!(engine.last_fen.as_deref(), Some(fen));
        assert_eq!(engine.last_moves.len(), 1);
        assert_eq!(engine.last_moves[0], (Square::E7, Square::E5, None));
    }

    // --- parse_go ---

    #[test]
    fn go_infinite() {
        let p = parse_go("go infinite");
        assert!(p.infinite);
        assert!(!p.ponder);
        assert_eq!(p.depth, None);
    }

    #[test]
    fn go_depth() {
        let p = parse_go("go depth 5");
        assert_eq!(p.depth, Some(5));
        assert!(!p.infinite);
    }

    #[test]
    fn go_movetime() {
        let p = parse_go("go movetime 1000");
        assert_eq!(p.movetime, Some(1000));
    }

    #[test]
    fn go_time_controls() {
        let p = parse_go("go wtime 60000 btime 45000 winc 1000 binc 500");
        assert_eq!(p.wtime, Some(60000));
        assert_eq!(p.btime, Some(45000));
        assert_eq!(p.winc, Some(1000));
        assert_eq!(p.binc, Some(500));
    }

    #[test]
    fn go_ponder() {
        let p = parse_go("go ponder");
        assert!(p.ponder);
        assert!(!p.infinite);
    }
}
