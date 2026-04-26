use std::sync::OnceLock;

use crate::engine::board::{BitBoard, Board};
use crate::engine::movegen::magic::MagicTable;
use crate::engine::mv::Movelist;
use crate::engine::piece::Piece;
use crate::engine::square::Square;

use super::{add_capture_moves, add_quiet_moves, extract_pieces_from_bitboard};

static TABLE: OnceLock<MagicTable> = OnceLock::new();
pub(super) fn get_rook_table() -> &'static MagicTable {
    TABLE.get_or_init(|| MagicTable::new(rook_blocker_mask, rook_attacks_slow))
}

pub(super) fn gen_rook_moves(board: &Board, movelist: &mut Movelist) {
    let table = get_rook_table();
    let color = board.get_current_player_color();
    let rooks = board.get_bitboard_for_piece(Piece::Rook, color);
    let occupancy = board.get_occupied_squares();

    for sq in extract_pieces_from_bitboard(rooks) {
        let targets = table.attacks(sq, occupancy);
        add_quiet_moves(sq, targets, board, movelist);
        add_capture_moves(sq, targets, board, movelist);
    }
}

pub(super) fn gen_rook_attacks(board: &Board) -> BitBoard {
    let table = get_rook_table();
    let color = board.get_current_player_color();
    let rooks = board.get_bitboard_for_piece(Piece::Rook, color);
    let occupancy = board.get_occupied_squares();
    let mut attacks = 0u64;

    for sq in extract_pieces_from_bitboard(rooks) {
        attacks |= table.attacks(sq, occupancy);
    }

    attacks
}

// Compute the blocker mask for a rook on `sq`.
// Includes all squares on the same rank/file where a blocker could matter,
// excluding the board edges (a piece on the edge can't block further).
fn rook_blocker_mask(sq: Square) -> BitBoard {
    let f = sq.file() as usize;
    let r = sq.rank() as usize;
    let mut mask = 0u64;

    // North: ranks r+1..6 on same file (exclude rank 7 edge)
    for rank in (r + 1)..7 {
        mask |= 1u64 << (rank * 8 + f);
    }
    // South: ranks 1..r on same file (exclude rank 0 edge)
    if r > 0 {
        for rank in 1..r {
            mask |= 1u64 << (rank * 8 + f);
        }
    }
    // East: files f+1..6 on same rank (exclude file 7 edge)
    for file in (f + 1)..7 {
        mask |= 1u64 << (r * 8 + file);
    }
    // West: files 1..f on same rank (exclude file 0 edge)
    if f > 0 {
        for file in 1..f {
            mask |= 1u64 << (r * 8 + file);
        }
    }

    mask
}

// Compute the actual attack bitboard for a rook on `sq` given an occupancy.
// Traces all 4 rays and stops at the first occupied square (included).
fn rook_attacks_slow(sq: Square, occupancy: BitBoard) -> BitBoard {
    let f = sq.file() as usize;
    let r = sq.rank() as usize;
    let mut attacks = 0u64;

    // North
    for rank in (r + 1)..8 {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if occupancy & bit != 0 {
            break;
        }
    }
    // South
    if r > 0 {
        for rank in (0..r).rev() {
            let bit = 1u64 << (rank * 8 + f);
            attacks |= bit;
            if occupancy & bit != 0 {
                break;
            }
        }
    }
    // East
    for file in (f + 1)..8 {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occupancy & bit != 0 {
            break;
        }
    }
    // West
    if f > 0 {
        for file in (0..f).rev() {
            let bit = 1u64 << (r * 8 + file);
            attacks |= bit;
            if occupancy & bit != 0 {
                break;
            }
        }
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_rook_moves(&board, &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square) -> bool {
        list.as_slice()
            .iter()
            .any(|m| m.from() == from && m.to() == to)
    }

    fn attacks_for(fen: &str) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        gen_rook_attacks(&board)
    }


    // Rook on e4 (sq=28, file=4, rank=3).
    // Expected mask:
    //   North : e5, e6, e7        (ranks 4,5,6 on file 4)
    //   South : e2, e3            (ranks 1,2 on file 4)   — e1 excluded (edge)
    //   East  : f4, g4            (files 5,6 on rank 3)   — h4 excluded (edge)
    //   West  : b4, c4, d4        (files 1,2,3 on rank 3) — a4 excluded (edge)
    #[test]
    fn blocker_mask_e4() {
        let mask = rook_blocker_mask(Square::E4);

        // North
        assert!(
            mask & Square::E5.to_bitboard() != 0,
            "e5 should be in the mask"
        );
        assert!(
            mask & Square::E6.to_bitboard() != 0,
            "e6 should be in the mask"
        );
        assert!(
            mask & Square::E7.to_bitboard() != 0,
            "e7 should be in the mask"
        );
        assert!(
            mask & Square::E8.to_bitboard() == 0,
            "e8 should not be in the mask (edge)"
        );

        // South
        assert!(
            mask & Square::E2.to_bitboard() != 0,
            "e2 should be in the mask"
        );
        assert!(
            mask & Square::E3.to_bitboard() != 0,
            "e3 should be in the mask"
        );
        assert!(
            mask & Square::E1.to_bitboard() == 0,
            "e1 should not be in the mask (edge)"
        );

        // East
        assert!(
            mask & Square::F4.to_bitboard() != 0,
            "f4 should be in the mask"
        );
        assert!(
            mask & Square::G4.to_bitboard() != 0,
            "g4 should be in the mask"
        );
        assert!(
            mask & Square::H4.to_bitboard() == 0,
            "h4 should not be in the mask (edge)"
        );

        // West
        assert!(
            mask & Square::B4.to_bitboard() != 0,
            "b4 should be in the mask"
        );
        assert!(
            mask & Square::C4.to_bitboard() != 0,
            "c4 should be in the mask"
        );
        assert!(
            mask & Square::D4.to_bitboard() != 0,
            "d4 should be in the mask"
        );
        assert!(
            mask & Square::A4.to_bitboard() == 0,
            "a4 should not be in the mask (edge)"
        );

        // e4 itself excluded
        assert!(
            mask & Square::E4.to_bitboard() == 0,
            "e4 should not be in its own mask"
        );

        assert_eq!(mask.count_ones(), 10, "e4 mask should have 10 bits set");
    }

    #[test]
    fn attacks_empty_board() {
        let attacks = rook_attacks_slow(Square::E4, 0);
        assert!(
            attacks & Square::E8.to_bitboard() != 0,
            "e8 reachable on empty board"
        );
        assert!(
            attacks & Square::E1.to_bitboard() != 0,
            "e1 reachable on empty board"
        );
        assert!(
            attacks & Square::A4.to_bitboard() != 0,
            "a4 reachable on empty board"
        );
        assert!(
            attacks & Square::H4.to_bitboard() != 0,
            "h4 reachable on empty board"
        );
        assert!(
            attacks & Square::E4.to_bitboard() == 0,
            "e4 itself not in attacks"
        );
    }

    #[test]
    fn attacks_blocked_north_and_east() {
        // Blockers on e6 and g4
        let occupancy = Square::E6.to_bitboard() | Square::G4.to_bitboard();
        let attacks = rook_attacks_slow(Square::E4, occupancy);

        // North: e5 and e6 reachable, e7 blocked
        assert!(attacks & Square::E5.to_bitboard() != 0, "e5 reachable");
        assert!(
            attacks & Square::E6.to_bitboard() != 0,
            "e6 reachable (capture the blocker)"
        );
        assert!(attacks & Square::E7.to_bitboard() == 0, "e7 blocked");

        // East: f4 and g4 reachable, h4 blocked
        assert!(attacks & Square::F4.to_bitboard() != 0, "f4 reachable");
        assert!(
            attacks & Square::G4.to_bitboard() != 0,
            "g4 reachable (capture the blocker)"
        );
        assert!(attacks & Square::H4.to_bitboard() == 0, "h4 blocked");

        // South and West: unobstructed, reach the edges
        assert!(attacks & Square::E1.to_bitboard() != 0, "e1 reachable");
        assert!(attacks & Square::A4.to_bitboard() != 0, "a4 reachable");
    }

    #[test]
    fn blocker_mask_a1() {
        let mask = rook_blocker_mask(Square::A1);
        // North: a2..a7 (ranks 1..6, file 0) = 6 squares
        // East:  b1..g1 (files 1..6, rank 0) = 6 squares
        assert_eq!(mask.count_ones(), 12);
    }

    #[test]
    fn rook_open_file_move_count() {
        // Rook on e4, empty board — 14 squares reachable
        let list = moves_for("8/8/8/8/4R3/8/8/8 w - - 0 1");
        assert_eq!(list.len(), 14);
    }

    #[test]
    fn rook_blocked_by_own_pieces() {
        // Rook on e4, own pawns on e5 and g4
        let list = moves_for("8/8/8/4P3/4R1P1/8/8/8 w - - 0 1");
        assert!(
            !has_move(&list, Square::E4, Square::E5),
            "cannot capture own piece on e5"
        );
        assert!(
            !has_move(&list, Square::E4, Square::G4),
            "cannot capture own piece on g4"
        );
        assert!(
            !has_move(&list, Square::E4, Square::H4),
            "blocked behind own piece on g4"
        );
        assert!(
            !has_move(&list, Square::E4, Square::E6),
            "blocked behind own piece on e5"
        );
        assert!(
            has_move(&list, Square::E4, Square::F4),
            "f4 is empty and reachable"
        );
    }

    #[test]
    fn rook_captures_enemy() {
        // Rook on e4, enemy pieces on e6 and b4
        let list = moves_for("8/8/4p3/8/1p2R3/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E4, Square::E6), "can capture e6");
        assert!(has_move(&list, Square::E4, Square::B4), "can capture b4");
        assert!(
            !has_move(&list, Square::E4, Square::E7),
            "blocked behind enemy on e6"
        );
        assert!(
            !has_move(&list, Square::E4, Square::A4),
            "blocked behind enemy on b4"
        );
    }

    #[test]
    fn table_matches_slow_on_empty_board() {
        let table = MagicTable::new(rook_blocker_mask, rook_attacks_slow);
        for sq in 0..64u8 {
            let square = Square(sq);
            let expected = rook_attacks_slow(square, 0);
            let got = table.attacks(square, 0);
            assert_eq!(got, expected, "empty board mismatch on {}", square);
        }
    }

    #[test]
    fn table_matches_slow_with_blockers() {
        let table = MagicTable::new(rook_blocker_mask, rook_attacks_slow);
        let occupancy =
            Square::E6.to_bitboard() | Square::G4.to_bitboard() | Square::B1.to_bitboard();

        for sq in 0..64u8 {
            let square = Square(sq);
            let expected = rook_attacks_slow(square, occupancy);
            let got = table.attacks(square, occupancy);
            assert_eq!(got, expected, "blocker mismatch on {}", square);
        }
    }

    #[test]
    fn gen_attacks_single_rook_empty_board() {
        // Rook on e4 alone — controls 14 squares (7 on file + 7 on rank)
        let attacks = attacks_for("8/8/8/8/4R3/8/8/8 w - - 0 1");
        assert_eq!(attacks.count_ones(), 14);
        assert!(attacks & Square::E1.to_bitboard() != 0, "e1 controlled");
        assert!(attacks & Square::E8.to_bitboard() != 0, "e8 controlled");
        assert!(attacks & Square::A4.to_bitboard() != 0, "a4 controlled");
        assert!(attacks & Square::H4.to_bitboard() != 0, "h4 controlled");
        assert!(
            attacks & Square::E4.to_bitboard() == 0,
            "rook does not control its own square"
        );
    }

    #[test]
    fn gen_attacks_includes_own_piece_square() {
        // Own pawn on e6 blocks the rook — e6 is still controlled, e7 is not
        let attacks = attacks_for("8/8/4P3/8/4R3/8/8/8 w - - 0 1");
        assert!(attacks & Square::E5.to_bitboard() != 0, "e5 controlled");
        assert!(
            attacks & Square::E6.to_bitboard() != 0,
            "e6 controlled (blocker square)"
        );
        assert!(
            attacks & Square::E7.to_bitboard() == 0,
            "e7 not controlled (behind own piece)"
        );
    }

    #[test]
    fn gen_attacks_union_of_two_rooks() {
        // Rooks on a1 and h8 — no shared rank/file, so they don't block each other
        // a1 attacks 14, h8 attacks 14, overlap = {h1, a8} → union = 26
        let attacks = attacks_for("7R/8/8/8/8/8/8/R7 w - - 0 1");
        assert_eq!(attacks.count_ones(), 26);
        assert!(
            attacks & Square::A8.to_bitboard() != 0,
            "a8 controlled by a1 rook"
        );
        assert!(
            attacks & Square::H1.to_bitboard() != 0,
            "h1 controlled by both rooks"
        );
        assert!(
            attacks & Square::H8.to_bitboard() == 0,
            "h8 not in attacks (rook's own square)"
        );
        assert!(
            attacks & Square::A1.to_bitboard() == 0,
            "a1 not in attacks (rook's own square)"
        );
    }

    #[test]
    fn gen_attacks_black_rooks() {
        // Black rook on e4, black to move
        let attacks = attacks_for("8/8/8/8/4r3/8/8/8 b - - 0 1");
        assert_eq!(attacks.count_ones(), 14);
        assert!(attacks & Square::E1.to_bitboard() != 0);
        assert!(attacks & Square::E8.to_bitboard() != 0);
    }
}
