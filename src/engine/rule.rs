use crate::engine::board::Board;
use crate::engine::mv::Movelist;
use crate::engine::piece::Color;
use crate::engine::movegen::{gen_attacks_for_color, gen_moves};

pub fn is_in_check(board: &Board, color: Color) -> bool {
    let king_square = board.get_king_square(color);
    let opponent_attacks = gen_attacks_for_color(board, color.flip());
    opponent_attacks & king_square.to_bitboard() != 0
}

pub fn gen_legal_moves(board: &mut Board) -> Movelist {
    let pseudo_moves = gen_moves(board);
    let color = board.get_current_player_color();
    let mut legal = Movelist::new();
    for &mv in pseudo_moves.as_slice() {
        let state = board.make_move(mv);
        if !is_in_check(board, color) {
            legal.add_move(mv);
        }
        board.unmake_move(mv, state);
    }
    legal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;

    fn perft(board: &mut Board, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        let moves = gen_legal_moves(board);
        if depth == 1 {
            return moves.len() as u64;
        }
        let mut count = 0;
        for &mv in moves.as_slice() {
            let state = board.make_move(mv);
            count += perft(board, depth - 1);
            board.unmake_move(mv, state);
        }
        count
    }

    // Starting position — authoritative perft values
    #[test]
    fn perft_startpos_d1() {
        assert_eq!(perft(&mut Board::new(), 1), 20);
    }

    #[test]
    fn perft_startpos_d2() {
        assert_eq!(perft(&mut Board::new(), 2), 400);
    }

    #[test]
    fn perft_startpos_d3() {
        assert_eq!(perft(&mut Board::new(), 3), 8_902);
    }

    #[test]
    fn perft_startpos_d4() {
        assert_eq!(perft(&mut Board::new(), 4), 197_281);
    }

    // Kiwipete — stresses castling, en passant, promotions
    fn kiwipete() -> Board {
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap()
    }

    #[test]
    fn perft_kiwipete_d1() {
        assert_eq!(perft(&mut kiwipete(), 1), 48);
    }

    #[test]
    fn perft_kiwipete_d2() {
        assert_eq!(perft(&mut kiwipete(), 2), 2_039);
    }

    #[test]
    fn perft_kiwipete_d3() {
        assert_eq!(perft(&mut kiwipete(), 3), 97_862);
    }

    // Position 3 — stresses en passant edge cases
    fn pos3() -> Board {
        Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap()
    }

    #[test]
    fn perft_pos3_d1() {
        assert_eq!(perft(&mut pos3(), 1), 14);
    }

    #[test]
    fn perft_pos3_d2() {
        assert_eq!(perft(&mut pos3(), 2), 191);
    }

    #[test]
    fn perft_pos3_d3() {
        assert_eq!(perft(&mut pos3(), 3), 2_812);
    }

    #[test]
    fn perft_suite() {
        const EPD: &str = include_str!("../../tests/fixtures/standard.epd");
        const MAX_DEPTH: u32 = 4;

        let mut failures = Vec::new();

        for (line_idx, line) in EPD.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let segments: Vec<&str> = line.split(';').collect();
            let fen = segments[0].trim();

            let mut board = match Board::from_fen(fen) {
                Ok(b) => b,
                Err(e) => {
                    failures.push(format!("line {}: bad FEN {:?}: {}", line_idx + 1, fen, e));
                    continue;
                }
            };

            for seg in &segments[1..] {
                let seg = seg.trim();
                let mut tokens = seg.split_whitespace();
                let depth_tag = match tokens.next() { Some(t) => t, None => continue };
                let expected_str = match tokens.next() { Some(t) => t, None => continue };

                if !depth_tag.starts_with('D') { continue; }
                let depth: u32 = match depth_tag[1..].parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                if depth > MAX_DEPTH { continue; }

                let expected: u64 = match expected_str.parse() {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                let got = perft(&mut board, depth);
                if got != expected {
                    failures.push(format!(
                        "line {} D{}: {} — expected {} got {}",
                        line_idx + 1, depth, fen, expected, got
                    ));
                }
            }
        }

        if !failures.is_empty() {
            panic!("{} failure(s):\n{}", failures.len(), failures.join("\n"));
        }
    }
}