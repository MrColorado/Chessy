use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::{Move, Movelist};
use crate::engine::piece::{Color, Piece};
use crate::engine::square::Square;

use super::extract_pieces_from_bitboard;
use super::{NOT_A_FILE, NOT_H_FILE, RANK_1, RANK_2, RANK_7, RANK_8};
use crate::engine::movegen::rook::rook_attack_from;

pub(super) fn gen_pawn_moves(board: &Board, check_mask: BitBoard, pin_mask: &[u64; 64], movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let pawns = board.get_bitboard_for_piece(Piece::Pawn, color);
    for pos in extract_pieces_from_bitboard(pawns) {
        let effective = check_mask & pin_mask[pos.to_index()];
        pawn_moves(pos, effective, board, movelist);
        pawn_attacks(pos, effective, check_mask, pin_mask, board, movelist);
    }
}

pub(super) fn gen_pawn_attacks(board: &Board, color: Color) -> BitBoard {
    let pawns = board.get_bitboard_for_piece(Piece::Pawn, color);

    let mut attacks = 0u64;
    for pos in extract_pieces_from_bitboard(pawns) {
        attacks |= pawn_square_attacks(pos, color);
    }
    attacks
}

pub(super) fn pawn_attack_from(sq: Square, color: Color) -> BitBoard {
    pawn_square_attacks(sq, color)
}

fn add_pawn_targets(
    from: Square,
    targets: BitBoard,
    promotion_rank: BitBoard,
    is_capture: bool,
    movelist: &mut Movelist,
) {
    for dest in extract_pieces_from_bitboard(targets) {
        if dest.to_bitboard() & promotion_rank != 0 {
            for piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                let mv = if is_capture {
                    Move::promote_capture(from, dest, piece)
                } else {
                    Move::promote(from, dest, piece)
                };
                movelist.add_move(mv);
            }
        } else if is_capture {
            movelist.add_move(Move::capture(from, dest, Piece::Pawn));
        } else {
            movelist.add_move(Move::quiet(from, dest, Piece::Pawn));
        }
    }
}

fn pawn_moves(pos: Square, effective: BitBoard, board: &Board, movelist: &mut Movelist) {
    let pos_bb = pos.to_bitboard();
    let occupied = board.get_occupied_squares();

    if board.get_white_to_move() {
        let dest = pos_bb << 8;
        if dest & occupied == 0 && dest & effective != 0 {
            add_pawn_targets(pos, dest, RANK_8, false, movelist);
            if pos_bb & RANK_2 != 0 {
                let double_dest = dest << 8;
                if double_dest & occupied == 0 && double_dest & effective != 0 {
                    movelist.add_move(Move::double_pawn_push(pos, Square(double_dest.trailing_zeros() as u8)));
                }
            }
        }
    } else {
        let dest = pos_bb >> 8;
        if dest & occupied == 0 && dest & effective != 0 {
            add_pawn_targets(pos, dest, RANK_1, false, movelist);
            if pos_bb & RANK_7 != 0 {
                let double_dest = dest >> 8;
                if double_dest & occupied == 0 && double_dest & effective != 0 {
                    movelist.add_move(Move::double_pawn_push(pos, Square(double_dest.trailing_zeros() as u8)));
                }
            }
        }
    }
}

fn pawn_attacks(pos: Square, effective: BitBoard, check_mask: BitBoard, pin_mask: &[u64; 64], board: &Board, movelist: &mut Movelist) {
    let pos_bb = pos.to_bitboard();
    let color = board.get_current_player_color();

    if board.get_white_to_move() {
        let left_attack = (pos_bb & NOT_A_FILE) << 7;
        let right_attack = (pos_bb & NOT_H_FILE) << 9;
        let attacks =
            (left_attack | right_attack) & board.get_occupied_squares_for_color(Color::Black) & effective;
        add_pawn_targets(pos, attacks, RANK_8, true, movelist);
        if let Some(ep_sq) = board.get_en_passant_square() {
            let ep_bb = ep_sq.to_bitboard();
            if (left_attack | right_attack) & ep_bb != 0
                && pin_mask[pos.to_index()] & ep_bb != 0
                && check_mask & ep_bb != 0
                && !ep_exposes_king_horizontally(board, color, pos, ep_sq)
            {
                movelist.add_move(Move::en_passant(pos, ep_sq));
            }
        }
    } else {
        let left_attack = (pos_bb & NOT_A_FILE) >> 9;
        let right_attack = (pos_bb & NOT_H_FILE) >> 7;
        let attacks =
            (left_attack | right_attack) & board.get_occupied_squares_for_color(Color::White) & effective;
        add_pawn_targets(pos, attacks, RANK_1, true, movelist);
        if let Some(ep_sq) = board.get_en_passant_square() {
            let ep_bb = ep_sq.to_bitboard();
            if (left_attack | right_attack) & ep_bb != 0
                && pin_mask[pos.to_index()] & ep_bb != 0
                && check_mask & ep_bb != 0
                && !ep_exposes_king_horizontally(board, color, pos, ep_sq)
            {
                movelist.add_move(Move::en_passant(pos, ep_sq));
            }
        }
    }
}

fn ep_exposes_king_horizontally(board: &Board, color: Color, from: Square, ep_sq: Square) -> bool {
    let king_sq = board.get_king_square(color);
    if king_sq.rank() != from.rank() {
        return false;
    }
    let captured_sq = if color == Color::White {
        Square(ep_sq.0 - 8)
    } else {
        Square(ep_sq.0 + 8)
    };
    let occ_after = board.get_occupied_squares() & !(from.to_bitboard() | captured_sq.to_bitboard());
    let enemy = color.flip();
    let sliders = board.get_bitboard_for_piece(Piece::Rook, enemy)
        | board.get_bitboard_for_piece(Piece::Queen, enemy);
    rook_attack_from(king_sq, occ_after) & sliders != 0
}

fn pawn_square_attacks(pos: Square, color: Color) -> BitBoard {
    let pos_bb = pos.to_bitboard();
    match color {
        Color::White => {
            let left_attack = (pos_bb & NOT_A_FILE) << 7;
            let right_attack = (pos_bb & NOT_H_FILE) << 9;
            left_attack | right_attack
        }
        Color::Black => {
            let left_attack = (pos_bb & NOT_A_FILE) >> 9;
            let right_attack = (pos_bb & NOT_H_FILE) >> 7;
            left_attack | right_attack
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_pawn_moves(&board, u64::MAX, &[u64::MAX; 64], &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square, promo: Option<Piece>) -> bool {
        list.as_slice()
            .iter()
            .any(|m| m.from() == from && m.to() == to && m.promotion() == promo)
    }

    #[test]
    fn white_double_push_from_rank2() {
        let list = moves_for("8/8/8/8/8/8/4P3/8 w - - 0 1");
        assert!(has_move(&list, Square::E2, Square::E4, None));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn white_blocked_no_push() {
        let list = moves_for("8/8/8/8/8/4p3/4P3/8 w - - 0 1");
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn white_blocked_on_rank3_no_double_push() {
        let list = moves_for("8/8/8/8/4p3/8/4P3/8 w - - 0 1");
        assert!(has_move(&list, Square::E2, Square::E3, None));
        assert!(!has_move(&list, Square::E2, Square::E4, None));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn white_promotion() {
        let list = moves_for("8/4P3/8/8/8/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E7, Square::E8, Some(Piece::Queen)));
        assert!(has_move(&list, Square::E7, Square::E8, Some(Piece::Rook)));
        assert!(has_move(&list, Square::E7, Square::E8, Some(Piece::Bishop)));
        assert!(has_move(&list, Square::E7, Square::E8, Some(Piece::Knight)));
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn white_capture() {
        let list = moves_for("8/8/8/8/8/3p1p2/4P3/8 w - - 0 1");
        assert!(has_move(&list, Square::E2, Square::D3, None));
        assert!(has_move(&list, Square::E2, Square::F3, None));
    }

    #[test]
    fn white_capture_promotion() {
        let list = moves_for("3r4/4P3/8/8/8/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E7, Square::D8, Some(Piece::Queen)));
        assert!(has_move(&list, Square::E7, Square::D8, Some(Piece::Rook)));
        assert!(has_move(&list, Square::E7, Square::D8, Some(Piece::Bishop)));
        assert!(has_move(&list, Square::E7, Square::D8, Some(Piece::Knight)));
    }

    #[test]
    fn white_en_passant() {
        let list = moves_for("8/8/8/3Pp3/8/8/8/8 w - e6 0 1");
        assert!(has_move(&list, Square::D5, Square::E6, None));
    }

    #[test]
    fn black_double_push_from_rank7() {
        let list = moves_for("8/4p3/8/8/8/8/8/8 b - - 0 1");
        assert!(has_move(&list, Square::E7, Square::E5, None));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn black_promotion() {
        let list = moves_for("8/8/8/8/8/8/4p3/8 b - - 0 1");
        assert!(has_move(&list, Square::E2, Square::E1, Some(Piece::Queen)));
        assert!(has_move(&list, Square::E2, Square::E1, Some(Piece::Rook)));
        assert!(has_move(&list, Square::E2, Square::E1, Some(Piece::Bishop)));
        assert!(has_move(&list, Square::E2, Square::E1, Some(Piece::Knight)));
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn black_en_passant() {
        let list = moves_for("8/8/8/8/3pP3/8/8/8 b - e3 0 1");
        assert!(has_move(&list, Square::D4, Square::E3, None));
    }

    fn pawn_attacks_for(fen: &str) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        gen_pawn_attacks(&board, board.get_current_player_color())
    }

    #[test]
    fn white_pawn_attacks_center() {
        // e4 pawn controls d5 and f5, not e5 (pawns don't push diagonally)
        let attacks = pawn_attacks_for("8/8/8/8/4P3/8/8/8 w - - 0 1");
        assert!(attacks & Square::D5.to_bitboard() != 0, "d5 controlled");
        assert!(attacks & Square::F5.to_bitboard() != 0, "f5 controlled");
        assert!(attacks & Square::E5.to_bitboard() == 0, "e5 not controlled");
        assert_eq!(attacks.count_ones(), 2);
    }

    #[test]
    fn white_pawn_a_file_no_wrap() {
        // a4 pawn can only attack b5 — no wrap around to h5
        let attacks = pawn_attacks_for("8/8/8/8/P7/8/8/8 w - - 0 1");
        assert!(attacks & Square::B5.to_bitboard() != 0, "b5 controlled");
        assert_eq!(attacks.count_ones(), 1);
    }

    #[test]
    fn white_pawn_h_file_no_wrap() {
        // h4 pawn can only attack g5
        let attacks = pawn_attacks_for("8/8/8/8/7P/8/8/8 w - - 0 1");
        assert!(attacks & Square::G5.to_bitboard() != 0, "g5 controlled");
        assert_eq!(attacks.count_ones(), 1);
    }

    #[test]
    fn black_pawn_attacks_center() {
        // e5 pawn controls d4 and f4
        let attacks = pawn_attacks_for("8/8/8/4p3/8/8/8/8 b - - 0 1");
        assert!(attacks & Square::D4.to_bitboard() != 0, "d4 controlled");
        assert!(attacks & Square::F4.to_bitboard() != 0, "f4 controlled");
        assert!(attacks & Square::E4.to_bitboard() == 0, "e4 not controlled");
        assert_eq!(attacks.count_ones(), 2);
    }

    #[test]
    fn pawn_attacks_union_of_two_pawns() {
        // White pawns on c3 and g3
        let attacks = pawn_attacks_for("8/8/8/8/8/2P3P1/8/8 w - - 0 1");
        assert!(
            attacks & Square::B4.to_bitboard() != 0,
            "b4 controlled by c3"
        );
        assert!(
            attacks & Square::D4.to_bitboard() != 0,
            "d4 controlled by c3"
        );
        assert!(
            attacks & Square::F4.to_bitboard() != 0,
            "f4 controlled by g3"
        );
        assert!(
            attacks & Square::H4.to_bitboard() != 0,
            "h4 controlled by g3"
        );
        assert_eq!(attacks.count_ones(), 4);
    }
}
