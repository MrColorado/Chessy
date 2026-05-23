use crate::engine::board::Board;
use crate::engine::piece::Color;
use crate::engine::movegen::{gen_attacks_for_color, gen_legal_moves};

pub fn is_in_check(board: &Board, color: Color) -> bool {
    let king_square = board.get_king_square(color);
    let opponent_attacks = gen_attacks_for_color(board, color.flip());
    opponent_attacks & king_square.to_bitboard() != 0
}

pub fn perft(board: &mut Board, depth: u32) -> u64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;

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
}
