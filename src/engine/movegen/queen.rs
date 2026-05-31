use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::Movelist;
use crate::engine::piece::{Piece, Color};

use super::bishop::get_bishop_table;
use super::rook::get_rook_table;
use super::{add_capture_moves, add_quiet_moves, extract_pieces_from_bitboard};

pub(super) fn gen_queen_moves(board: &Board, check_mask: BitBoard, pin_mask: &[u64; 64], movelist: &mut Movelist) {
    let rook_table = get_rook_table();
    let bishop_table = get_bishop_table();
    let color = board.get_current_player_color();
    let queens = board.get_bitboard_for_piece(Piece::Queen, color);
    let occupancy = board.get_occupied_squares();

    for sq in extract_pieces_from_bitboard(queens) {
        let effective = check_mask & pin_mask[sq.to_index()];
        let targets = (rook_table.attacks(sq, occupancy) | bishop_table.attacks(sq, occupancy)) & effective;
        add_quiet_moves(sq, targets, Piece::Queen, board, movelist);
        add_capture_moves(sq, targets, Piece::Queen, board, movelist);
    }
}

pub(super) fn gen_queen_captures(board: &Board, check_mask: BitBoard, pin_mask: &[u64; 64], movelist: &mut Movelist) {
    let rook_table = get_rook_table();
    let bishop_table = get_bishop_table();
    let color = board.get_current_player_color();
    let queens = board.get_bitboard_for_piece(Piece::Queen, color);
    let occupancy = board.get_occupied_squares();

    for sq in extract_pieces_from_bitboard(queens) {
        let effective = check_mask & pin_mask[sq.to_index()];
        let targets = (rook_table.attacks(sq, occupancy) | bishop_table.attacks(sq, occupancy)) & effective;
        add_capture_moves(sq, targets, Piece::Queen, board, movelist);
    }
}

pub(super) fn gen_queen_attacks(board: &Board, color: Color, occupancy: BitBoard) -> BitBoard {
    let rook_table = get_rook_table();
    let bishop_table = get_bishop_table();
    let queens = board.get_bitboard_for_piece(Piece::Queen, color);
    let mut attacks = 0u64;

    for sq in extract_pieces_from_bitboard(queens) {
        attacks |= rook_table.attacks(sq, occupancy) | bishop_table.attacks(sq, occupancy);
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;
    use crate::engine::square::Square;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_queen_moves(&board, u64::MAX, &[u64::MAX; 64], &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square) -> bool {
        list.as_slice()
            .iter()
            .any(|m| m.from() == from && m.to() == to)
    }

    fn attacks_for(fen: &str) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        gen_queen_attacks(&board, board.get_current_player_color(), board.get_occupied_squares())
    }

    #[test]
    fn queen_open_board_move_count() {
        // Queen on e4 — 14 rook squares + 13 bishop squares = 27 distinct squares
        let list = moves_for("8/8/8/8/4Q3/8/8/8 w - - 0 1");
        assert_eq!(list.len(), 27);
    }

    #[test]
    fn queen_blocked_by_own_pieces() {
        // Own pawns on e5 (blocks north) and f5 (blocks NE diagonal)
        let list = moves_for("8/8/8/4PP2/4Q3/8/8/8 w - - 0 1");
        assert!(!has_move(&list, Square::E4, Square::E5), "own piece on e5");
        assert!(!has_move(&list, Square::E4, Square::F5), "own piece on f5");
        assert!(!has_move(&list, Square::E4, Square::E6), "blocked behind e5");
        assert!(!has_move(&list, Square::E4, Square::G6), "blocked behind f5");
        assert!(has_move(&list, Square::E4, Square::E3), "e3 reachable");
        assert!(has_move(&list, Square::E4, Square::F3), "f3 reachable (SE diagonal)");
    }

    #[test]
    fn queen_captures_enemy() {
        // Enemy on e6 (north) and g6 (NE diagonal)
        let list = moves_for("8/8/4p1p1/8/4Q3/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E4, Square::E6), "can capture e6");
        assert!(has_move(&list, Square::E4, Square::G6), "can capture g6");
        assert!(!has_move(&list, Square::E4, Square::E7), "blocked behind e6");
        assert!(!has_move(&list, Square::E4, Square::H7), "blocked behind g6");
    }

    #[test]
    fn gen_attacks_single_queen_empty_board() {
        // Queen on e4 — 27 controlled squares (rook 14 + bishop 13, no overlap)
        let attacks = attacks_for("8/8/8/8/4Q3/8/8/8 w - - 0 1");
        assert_eq!(attacks.count_ones(), 27);
        // Rook rays
        assert!(attacks & Square::E1.to_bitboard() != 0, "e1 via rook S");
        assert!(attacks & Square::E8.to_bitboard() != 0, "e8 via rook N");
        assert!(attacks & Square::A4.to_bitboard() != 0, "a4 via rook W");
        assert!(attacks & Square::H4.to_bitboard() != 0, "h4 via rook E");
        // Bishop rays
        assert!(attacks & Square::H7.to_bitboard() != 0, "h7 via bishop NE");
        assert!(attacks & Square::A8.to_bitboard() != 0, "a8 via bishop NW");
        assert!(attacks & Square::H1.to_bitboard() != 0, "h1 via bishop SE");
        assert!(attacks & Square::B1.to_bitboard() != 0, "b1 via bishop SW");
        assert!(attacks & Square::E4.to_bitboard() == 0, "e4 not in own attacks");
    }

    #[test]
    fn gen_attacks_includes_own_piece_square() {
        // Own pawns on e6 (blocks N) and g6 (blocks NE) — both squares still controlled
        let attacks = attacks_for("8/8/4P1P1/8/4Q3/8/8/8 w - - 0 1");
        assert!(attacks & Square::E6.to_bitboard() != 0, "e6 controlled (blocker)");
        assert!(attacks & Square::G6.to_bitboard() != 0, "g6 controlled (blocker)");
        assert!(attacks & Square::E7.to_bitboard() == 0, "e7 not controlled");
        assert!(attacks & Square::H7.to_bitboard() == 0, "h7 not controlled");
    }

    #[test]
    fn gen_attacks_black_queen() {
        let attacks = attacks_for("8/8/8/8/4q3/8/8/8 b - - 0 1");
        assert_eq!(attacks.count_ones(), 27);
    }
}
