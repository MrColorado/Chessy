use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::{Move, Movelist};
use crate::engine::piece::{Color, Piece};
use crate::engine::square::Square;

use super::extract_pieces_from_bitboard;
use super::{NOT_A_FILE, NOT_H_FILE, RANK_1, RANK_2, RANK_7, RANK_8};

pub(super) fn gen_pawn_moves(board: &Board, movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let pawns = board.get_bitboard_for_piece(Piece::Pawn, color);
    for pos in extract_pieces_from_bitboard(pawns) {
        pawn_moves(pos, board, movelist);
        pawn_attacks(pos, board, movelist);
    }
}

pub(super) fn gen_pawn_attacks(board: &Board, movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let pawns = board.get_bitboard_for_piece(Piece::Pawn, color);
    for pos in extract_pieces_from_bitboard(pawns) {
        pawn_attacks(pos, board, movelist);
    }
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
            movelist.add_move(Move::capture(from, dest));
        } else {
            movelist.add_move(Move::quiet(from, dest));
        }
    }
}

fn pawn_moves(pos: Square, board: &Board, movelist: &mut Movelist) {
    let pos_bb = pos.to_bitboard();
    let occupied = board.get_occupied_squares();

    if board.get_white_to_move() {
        let dest = pos_bb << 8;
        if dest & occupied == 0 {
            add_pawn_targets(pos, dest, RANK_8, false, movelist);
            if pos_bb & RANK_2 != 0 {
                let double_dest = dest << 8;
                if double_dest & occupied == 0 {
                    movelist.add_move(Move::quiet(
                        pos,
                        Square(double_dest.trailing_zeros() as u8),
                    ));
                }
            }
        }
    } else {
        let dest = pos_bb >> 8;
        if dest & occupied == 0 {
            add_pawn_targets(pos, dest, RANK_1, false, movelist);
            if pos_bb & RANK_7 != 0 {
                let double_dest = dest >> 8;
                if double_dest & occupied == 0 {
                    movelist.add_move(Move::quiet(
                        pos,
                        Square(double_dest.trailing_zeros() as u8),
                    ));
                }
            }
        }
    }
}

fn pawn_attacks(pos: Square, board: &Board, movelist: &mut Movelist) {
    let pos_bb = pos.to_bitboard();

    if board.get_white_to_move() {
        let left_attack = (pos_bb & NOT_A_FILE) << 7;
        let right_attack = (pos_bb & NOT_H_FILE) << 9;
        let attacks =
            (left_attack | right_attack) & board.get_occupied_squares_for_color(Color::Black);
        add_pawn_targets(pos, attacks, RANK_8, true, movelist);
        if let Some(ep_sq) = board.get_en_passant_square() {
            if (left_attack | right_attack) & ep_sq.to_bitboard() != 0 {
                movelist.add_move(Move::en_passant(pos, ep_sq));
            }
        }
    } else {
        let left_attack = (pos_bb & NOT_A_FILE) >> 9;
        let right_attack = (pos_bb & NOT_H_FILE) >> 7;
        let attacks =
            (left_attack | right_attack) & board.get_occupied_squares_for_color(Color::White);
        add_pawn_targets(pos, attacks, RANK_1, true, movelist);
        if let Some(ep_sq) = board.get_en_passant_square() {
            if (left_attack | right_attack) & ep_sq.to_bitboard() != 0 {
                movelist.add_move(Move::en_passant(pos, ep_sq));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_pawn_moves( &board, &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square, promo: Option<Piece>) -> bool {
        list.as_slice()
            .iter()
            .any(|m| m.from() == from && m.to() == to && m.promotion() == promo)
    }

    #[test]
    fn white_single_push() {
        let list = moves_for("8/8/8/8/8/8/4P3/8 w - - 0 1");
        assert!(has_move(&list, Square::E2, Square::E3, None));
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
    fn black_single_push() {
        let list = moves_for("8/4p3/8/8/8/8/8/8 b - - 0 1");
        assert!(has_move(&list, Square::E7, Square::E6, None));
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
}
