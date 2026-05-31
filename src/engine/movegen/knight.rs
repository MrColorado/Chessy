use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::Movelist;
use crate::engine::piece::Piece;
use crate::engine::piece::Color;
use crate::engine::square::Square;

use super::extract_pieces_from_bitboard;
use super::{add_capture_moves, add_quiet_moves, NOT_A_FILE, NOT_AB_FILE, NOT_GH_FILE, NOT_H_FILE};

pub(super) fn gen_knight_moves(board: &Board, check_mask: BitBoard, pin_mask: &[u64; 64], movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let knights = board.get_bitboard_for_piece(Piece::Knight, color);
    for pos in extract_pieces_from_bitboard(knights) {
        let effective = check_mask & pin_mask[pos.to_index()];
        let targets = knight_targets(pos.to_bitboard()) & effective;
        add_quiet_moves(pos, targets, Piece::Knight, board, movelist);
        add_capture_moves(pos, targets, Piece::Knight, board, movelist);
    }
}

pub(super) fn gen_knight_captures(board: &Board, check_mask: BitBoard, pin_mask: &[u64; 64], movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let knights = board.get_bitboard_for_piece(Piece::Knight, color);
    for pos in extract_pieces_from_bitboard(knights) {
        let effective = check_mask & pin_mask[pos.to_index()];
        let targets = knight_targets(pos.to_bitboard()) & effective;
        add_capture_moves(pos, targets, Piece::Knight, board, movelist);
    }
}

pub(super) fn gen_knight_attacks(board: &Board, color: Color) -> BitBoard {
    let knights = board.get_bitboard_for_piece(Piece::Knight, color);
    let mut attacks = 0u64;

    for pos in extract_pieces_from_bitboard(knights) {
        attacks |= knight_targets(pos.to_bitboard());
    }

    attacks
}

pub(super) fn knights_attack_from(sq: Square) -> BitBoard {
    knight_targets(sq.to_bitboard())
}

fn knight_targets(pos: BitBoard) -> BitBoard {
    let mut target = (pos << 17) & NOT_A_FILE;
    target |= (pos << 15) & NOT_H_FILE;
    target |= (pos << 10) & NOT_AB_FILE;
    target |= (pos << 6) & NOT_GH_FILE;
    target |= (pos >> 17) & NOT_H_FILE;
    target |= (pos >> 15) & NOT_A_FILE;
    target |= (pos >> 10) & NOT_GH_FILE;
    target |= (pos >> 6) & NOT_AB_FILE;
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_knight_moves(&board, u64::MAX, &[u64::MAX; 64], &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square) -> bool {
        list.as_slice()
            .iter()
            .any(|m| m.from() == from && m.to() == to)
    }

    #[test]
    fn knight_center_all_moves() {
        let list = moves_for("8/8/8/8/3N4/8/8/8 w - - 0 1");
        assert_eq!(list.len(), 8);
        assert!(has_move(&list, Square::D4, Square::E6));
        assert!(has_move(&list, Square::D4, Square::C6));
        assert!(has_move(&list, Square::D4, Square::F5));
        assert!(has_move(&list, Square::D4, Square::B5));
        assert!(has_move(&list, Square::D4, Square::F3));
        assert!(has_move(&list, Square::D4, Square::B3));
        assert!(has_move(&list, Square::D4, Square::E2));
        assert!(has_move(&list, Square::D4, Square::C2));
    }

    #[test]
    fn knight_corner_a1_limited_moves() {
        let list = moves_for("8/8/8/8/8/8/8/N7 w - - 0 1");
        assert_eq!(list.len(), 2);
        assert!(has_move(&list, Square::A1, Square::B3));
        assert!(has_move(&list, Square::A1, Square::C2));
    }

    #[test]
    fn knight_corner_h8_limited_moves() {
        let list = moves_for("7N/8/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(list.len(), 2);
        assert!(has_move(&list, Square::H8, Square::G6));
        assert!(has_move(&list, Square::H8, Square::F7));
    }

    #[test]
    fn knight_h_file_no_wrap() {
        let list = moves_for("8/8/8/8/8/8/8/7N w - - 0 1");
        for mv in list.as_slice() {
            assert!(mv.to() != Square::A2, "knight wrapped from H1 to A-file");
        }
    }

    #[test]
    fn knight_blocked_by_own_pieces() {
        // Knight on D4 surrounded by own pieces on all target squares
        let list = moves_for("8/8/2PPP3/1P3P2/3N4/1P3P2/2PPP3/8 w - - 0 1");
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn knight_captures_enemy() {
        let list = moves_for("8/8/4p3/8/3N4/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::D4, Square::E6));
    }

    #[test]
    fn knight_does_not_capture_own_piece() {
        let list = moves_for("8/8/4P3/8/3N4/8/8/8 w - - 0 1");
        for mv in list.as_slice() {
            assert!(mv.to() != Square::E6, "knight captured own piece on E6");
        }
    }

    #[test]
    fn black_knight_moves() {
        let list = moves_for("8/8/8/8/3n4/8/8/8 b - - 0 1");
        assert_eq!(list.len(), 8);
    }

    #[test]
    fn black_knight_captures_white() {
        let list = moves_for("8/8/4P3/8/3n4/8/8/8 b - - 0 1");
        assert!(has_move(&list, Square::D4, Square::E6));
    }

    fn attacks_for(fen: &str) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        gen_knight_attacks(&board, board.get_current_player_color())
    }

    #[test]
    fn knight_attacks_center() {
        // Knight on d4 — 8 controlled squares
        let attacks = attacks_for("8/8/8/8/3N4/8/8/8 w - - 0 1");
        assert_eq!(attacks.count_ones(), 8);
        assert!(attacks & Square::E6.to_bitboard() != 0, "e6 controlled");
        assert!(attacks & Square::C6.to_bitboard() != 0, "c6 controlled");
        assert!(attacks & Square::F5.to_bitboard() != 0, "f5 controlled");
        assert!(attacks & Square::B5.to_bitboard() != 0, "b5 controlled");
        assert!(attacks & Square::D4.to_bitboard() == 0, "d4 not in own attacks");
    }

    #[test]
    fn knight_attacks_corner_a1() {
        // Knight on a1 — only 2 reachable squares
        let attacks = attacks_for("8/8/8/8/8/8/8/N7 w - - 0 1");
        assert_eq!(attacks.count_ones(), 2);
        assert!(attacks & Square::B3.to_bitboard() != 0, "b3 controlled");
        assert!(attacks & Square::C2.to_bitboard() != 0, "c2 controlled");
    }

    #[test]
    fn knight_attacks_include_own_piece_square() {
        // Knight controls e6 even when own pawn is there
        let attacks = attacks_for("8/8/4P3/8/3N4/8/8/8 w - - 0 1");
        assert!(attacks & Square::E6.to_bitboard() != 0, "e6 controlled despite own piece");
    }

    #[test]
    fn knight_attacks_union_of_two_knights() {
        // Knights on a1 and h8 — no overlap
        let attacks = attacks_for("7N/8/8/8/8/8/8/N7 w - - 0 1");
        // a1 knight: b3, c2
        assert!(attacks & Square::B3.to_bitboard() != 0);
        assert!(attacks & Square::C2.to_bitboard() != 0);
        // h8 knight: g6, f7
        assert!(attacks & Square::G6.to_bitboard() != 0);
        assert!(attacks & Square::F7.to_bitboard() != 0);
        assert_eq!(attacks.count_ones(), 4);
    }

    #[test]
    fn knights_attack_from_center() {
        let attacks = knights_attack_from(Square::D4);
        assert_eq!(attacks.count_ones(), 8);
    }

    #[test]
    fn knights_attack_from_corner_a1() {
        let attacks = knights_attack_from(Square::A1);
        assert_eq!(attacks.count_ones(), 2);
        assert!(attacks & Square::B3.to_bitboard() != 0);
        assert!(attacks & Square::C2.to_bitboard() != 0);
    }

    #[test]
    fn knights_attack_from_edge_h4() {
        let attacks = knights_attack_from(Square::H4);
        assert_eq!(attacks.count_ones(), 4);
        assert!(attacks & Square::G2.to_bitboard() != 0);
        assert!(attacks & Square::F3.to_bitboard() != 0);
        assert!(attacks & Square::F5.to_bitboard() != 0);
        assert!(attacks & Square::G6.to_bitboard() != 0);
    }

    #[test]
    fn black_knight_attacks() {
        // Black knight on d4 — same 8 squares, black to move
        let attacks = attacks_for("8/8/8/8/3n4/8/8/8 b - - 0 1");
        assert_eq!(attacks.count_ones(), 8);
    }
}
