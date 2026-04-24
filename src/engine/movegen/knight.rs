use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::{Move, Movelist};
use crate::engine::piece::Piece;
use crate::engine::square::Square;

use super::extract_pieces_from_bitboard;
use super::{NOT_A_FILE, NOT_AB_FILE, NOT_GH_FILE, NOT_H_FILE};

pub(super) fn gen_knight_moves(board: &Board, movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let knights = board.get_bitboard_for_piece(Piece::Knight, color);
    for pos in extract_pieces_from_bitboard(knights) {
        knight_moves(pos, board, movelist);
        knight_attacks(pos, board, movelist);
    }
}

fn knight_targets(pos: BitBoard) -> BitBoard {
    let mut target = (pos << 17) & NOT_A_FILE; // Up 2, Right 1
    target |= (pos << 15) & NOT_H_FILE; // Up 2, Left 1
    target |= (pos << 10) & NOT_AB_FILE; // Up 1, Right 2
    target |= (pos << 6) & NOT_GH_FILE; // Up 1, Left 2
    target |= (pos >> 17) & NOT_H_FILE; // Down 2, Left 1
    target |= (pos >> 15) & NOT_A_FILE; // Down 2, Right 1
    target |= (pos >> 10) & NOT_GH_FILE; // Down 1, Left 2
    target |= (pos >> 6) & NOT_AB_FILE; // Down 1, Right 2
    return target;
}

fn knight_moves(pos: Square, board: &Board, movelist: &mut Movelist) {
    let pos_bb = pos.to_bitboard();
    let occupied = board.get_occupied_squares();
    let mut targets = knight_targets(pos_bb);

    targets &= !occupied;
    for dest in extract_pieces_from_bitboard(targets) {
        movelist.add_move(Move::quiet(pos, dest));
    }
}

fn knight_attacks(pos: Square, board: &Board, movelist: &mut Movelist) {
    let pos_bb = pos.to_bitboard();
    let targets = knight_targets(pos_bb);
    let attacks = targets & board.get_occupied_squares_for_color(board.get_opponent_color());

    for dest in extract_pieces_from_bitboard(attacks) {
        movelist.add_move(Move::capture(pos, dest));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_knight_moves(&board, &mut list);
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
    fn knight_a_file_no_wrap() {
        let list = moves_for("8/8/8/8/8/8/8/N7 w - - 0 1");
        for mv in list.as_slice() {
            assert!(mv.to() != Square::H2, "knight wrapped from A1 to H-file");
        }
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
}
