use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::{Move, Movelist};
use crate::engine::piece::{Piece, Color};
use crate::engine::square::Square;

use super::{extract_pieces_from_bitboard, gen_attacks_for_color, add_capture_moves, add_quiet_moves, NOT_A_FILE, NOT_H_FILE};

pub(super) fn gen_king_moves(board: &Board, movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let king = board.get_bitboard_for_piece(Piece::King, color);
    let enemy_attacks = gen_attacks_for_color(board, board.get_opponent_color());
    for pos in extract_pieces_from_bitboard(king) {
        let targets = king_targets(pos.to_bitboard()) & !enemy_attacks;
        add_quiet_moves(pos, targets, board, movelist);
        add_capture_moves(pos, targets, board, movelist);
    }
    castle_moves(board, enemy_attacks, movelist); 
}

pub(super) fn gen_king_attacks(board: &Board, color: Color) -> BitBoard {
    let king = board.get_bitboard_for_piece(Piece::King, color);
    let mut attacks = 0u64;
    
    for pos in extract_pieces_from_bitboard(king) {
        let targets = king_targets(pos.to_bitboard());
        attacks |= targets;
    }
    attacks
}

fn king_targets(pos: BitBoard) -> BitBoard {
    let mut target = pos << 8;
    target |= pos >> 8;
    target |= (pos << 1) & NOT_A_FILE;
    target |= (pos >> 1) & NOT_H_FILE;
    target |= (pos << 9) & NOT_A_FILE;
    target |= (pos << 7) & NOT_H_FILE;
    target |= (pos >> 7) & NOT_A_FILE;
    target |= (pos >> 9) & NOT_H_FILE;
    target
}

fn castle_moves(board: &Board, enemy_attacks: BitBoard, movelist: &mut Movelist) {
    let color: Color = board.get_current_player_color();
    if board.can_castle_kingside(color) && kingside_path_safe(color, enemy_attacks) {
        movelist.add_move(Move::kingside_castle(color));
    }
    if board.can_castle_queenside(color) && queenside_path_safe(color, enemy_attacks) {
        movelist.add_move(Move::queenside_castle(color));
    }
}

fn kingside_path_safe(color: Color, opponent_attacks: BitBoard) -> bool {
    let transit = match color {
        Color::White => Square::E1.to_bitboard() | Square::F1.to_bitboard() | Square::G1.to_bitboard(),
        Color::Black => Square::E8.to_bitboard() | Square::F8.to_bitboard() | Square::G8.to_bitboard(),
    };
    opponent_attacks & transit == 0
}

fn queenside_path_safe(color: Color, opponent_attacks: BitBoard) -> bool {
    let transit = match color {
        Color::White => Square::E1.to_bitboard() | Square::D1.to_bitboard() | Square::C1.to_bitboard(),
        Color::Black => Square::E8.to_bitboard() | Square::D8.to_bitboard() | Square::C8.to_bitboard(),
    };
    opponent_attacks & transit == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::mv::MoveKind;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_king_moves(&board, &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square) -> bool {
        list.as_slice().iter().any(|m| m.from() == from && m.to() == to)
    }

    fn has_castle(list: &Movelist, kind: MoveKind) -> bool {
        list.as_slice().iter().any(|m| m.kind() == kind)
    }

    // --- Basic movement ---

    #[test]
    fn king_center_all_moves() {
        let list = moves_for("8/8/8/8/3K4/8/8/8 w - - 0 1");
        assert_eq!(list.len(), 8);
    }

    #[test]
    fn king_corner_a1() {
        let list = moves_for("8/8/8/8/8/8/8/K7 w - - 0 1");
        assert_eq!(list.len(), 3);
        assert!(has_move(&list, Square::A1, Square::A2));
        assert!(has_move(&list, Square::A1, Square::B1));
        assert!(has_move(&list, Square::A1, Square::B2));
    }

    #[test]
    fn king_corner_h8() {
        let list = moves_for("7k/8/8/8/8/8/8/8 b - - 0 1");
        assert_eq!(list.len(), 3);
        assert!(has_move(&list, Square::H8, Square::G8));
        assert!(has_move(&list, Square::H8, Square::H7));
        assert!(has_move(&list, Square::H8, Square::G7));
    }

    #[test]
    fn king_blocked_by_own_pieces() {
        // King on d2 fully surrounded by own pawns
        let list = moves_for("8/8/8/8/8/2PPP3/2PKP3/2PPP3 w - - 0 1");
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn king_captures_enemy() {
        let list = moves_for("8/8/8/5p2/4K3/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E4, Square::F5));
    }

    #[test]
    fn king_does_not_capture_own_piece() {
        let list = moves_for("8/8/8/5P2/4K3/8/8/8 w - - 0 1");
        for mv in list.as_slice() {
            assert!(mv.to() != Square::F5, "king captured own piece on F5");
        }
    }

    // --- Castling: basic ---

    #[test]
    fn white_kingside_castle() {
        let list = moves_for("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        assert!(has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn white_queenside_castle() {
        let list = moves_for("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1");
        assert!(has_castle(&list, MoveKind::QueensideCastle));
    }

    #[test]
    fn black_kingside_castle() {
        let list = moves_for("4k2r/8/8/8/8/8/8/4K3 b k - 0 1");
        assert!(has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn black_queenside_castle() {
        let list = moves_for("r3k3/8/8/8/8/8/8/4K3 b q - 0 1");
        assert!(has_castle(&list, MoveKind::QueensideCastle));
    }

    #[test]
    fn no_castle_without_rights() {
        let list = moves_for("4k3/8/8/8/8/8/8/4K2R w - - 0 1");
        assert!(!has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn no_kingside_castle_when_path_blocked() {
        // Knight on f1 blocks the path
        let list = moves_for("4k3/8/8/8/8/8/8/4KN1R w K - 0 1");
        assert!(!has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn no_queenside_castle_when_path_blocked() {
        // Bishop on c1 blocks the path
        let list = moves_for("4k3/8/8/8/8/8/8/R1B1K3 w Q - 0 1");
        assert!(!has_castle(&list, MoveKind::QueensideCastle));
    }

    // --- Castling: requires attacked squares (will fail until implemented) ---

    #[test]
    fn no_castle_when_king_in_check() {
        // Black rook on e3 attacks e1 — king is in check
        let list = moves_for("4k3/8/8/8/8/4r3/8/4K2R w K - 0 1");
        assert!(!has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn no_kingside_castle_through_attacked_square() {
        // Black rook on f3 attacks f1 — king passes through
        let list = moves_for("4k3/8/8/8/8/5r2/8/4K2R w K - 0 1");
        assert!(!has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn no_kingside_castle_onto_attacked_square() {
        // Black rook on g3 attacks g1 — king lands on attacked square
        let list = moves_for("4k3/8/8/8/8/6r1/8/4K2R w K - 0 1");
        assert!(!has_castle(&list, MoveKind::KingsideCastle));
    }

    #[test]
    fn no_queenside_castle_through_attacked_square() {
        // Black rook on d3 attacks d1 — king passes through
        let list = moves_for("4k3/8/8/8/8/3r4/8/R3K3 w Q - 0 1");
        assert!(!has_castle(&list, MoveKind::QueensideCastle));
    }

    #[test]
    fn no_queenside_castle_onto_attacked_square() {
        // Black rook on c3 attacks c1 — king lands on attacked square
        let list = moves_for("4k3/8/8/8/8/2r5/8/R3K3 w Q - 0 1");
        assert!(!has_castle(&list, MoveKind::QueensideCastle));
    }
}
