use std::sync::OnceLock;

use crate::engine::board::{BitBoard, Board};
use crate::engine::movegen::magic::MagicTable;
use crate::engine::mv::Movelist;
use crate::engine::piece::Piece;
use crate::engine::square::Square;


use super::{add_capture_moves, add_quiet_moves, extract_pieces_from_bitboard};

static TABLE: OnceLock<MagicTable> = OnceLock::new();
pub(super) fn get_bishop_table() -> &'static MagicTable {
    TABLE.get_or_init(|| MagicTable::new(bishop_blocker_mask, bishop_attacks_slow))
}

pub(super) fn gen_bishop_moves(board: &Board, movelist: &mut Movelist) {
    let table = get_bishop_table();
    let color = board.get_current_player_color();
    let bishops = board.get_bitboard_for_piece(Piece::Bishop, color);
    let occupancy = board.get_occupied_squares();

    for sq in extract_pieces_from_bitboard(bishops) {
        let targets = table.attacks(sq, occupancy);
        add_quiet_moves(sq, targets, board, movelist);
        add_capture_moves(sq, targets, board, movelist);
    }
}

pub(super) fn gen_bishop_attacks(board: &Board) -> BitBoard {
    let table = get_bishop_table();
    let color = board.get_current_player_color();
    let bishops = board.get_bitboard_for_piece(Piece::Bishop, color);
    let occupancy = board.get_occupied_squares();
    let mut attacks = 0u64;

    for sq in extract_pieces_from_bitboard(bishops) {
        attacks |= table.attacks(sq, occupancy);
    }

    attacks
}

// Compute the blocker mask for a bishop on `sq`.
// Includes all squares on the same diagonal where a blocker could matter,
// excluding the board edges (a piece on the edge can't block further).
fn bishop_blocker_mask(sq: Square) -> BitBoard {
    let f = sq.file() as usize;
    let r = sq.rank() as usize;
    let mut mask = 0u64;

    // North - East
    for i in 1..7 {
        if f + i >= 7 || r + i >= 7 {
            break;
        }
        mask |= 1u64 << ((r + i) * 8 + (f + i));
    }
    // South - East
    for i in 1..7 {
        if i >= r || f + i >= 7 {
            break;
        }
        mask |= 1u64 << ((r - i) * 8 + (f + i));
    }
    // South - West
    for i in 1..7 {
        if i >= r || i >= f {
            break;
        }
        mask |= 1u64 << ((r - i) * 8 + (f - i));
    }
    // North - West
    for i in 1..7 {
        if r + i >= 7 || i >= f {
            break;
        }
        mask |= 1u64 << ((r + i) * 8 + (f - i));
    }

    mask
}

fn bishop_attacks_slow(sq: Square, occupency: BitBoard) -> BitBoard {
    let f = sq.file() as usize;
    let r = sq.rank() as usize;
    let mut attacks = 0u64;

    // North - East
    for i in 1..8 {
        if f + i >= 8 || r + i >= 8 {
            break;
        }
        attacks |= 1u64 << ((r + i) * 8 + (f + i));
        if occupency & (1u64 << ((r + i) * 8 + (f + i))) != 0 {
            break;
        }
    }
    // South - East
    for i in 1..8 {
        if i > r || f + i >= 8 {
            break;
        }
        attacks |= 1u64 << ((r - i) * 8 + (f + i));
        if occupency & (1u64 << ((r - i) * 8 + (f + i))) != 0 {
            break;
        }
    }
    // South - West
    for i in 1..8 {
        if i > r || i > f {
            break;
        }
        attacks |= 1u64 << ((r - i) * 8 + (f - i));
        if occupency & (1u64 << ((r - i) * 8 + (f - i))) != 0 {
            break;
        }
    }
    // North - West
    for i in 1..8 {
        if r + i >= 8 || i > f {
            break;
        }
        attacks |= 1u64 << ((r + i) * 8 + (f - i));
        if occupency & (1u64 << ((r + i) * 8 + (f - i))) != 0 {
            break;
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
        gen_bishop_moves(&board, &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square) -> bool {
        list.as_slice()
            .iter()
            .any(|m| m.from() == from && m.to() == to)
    }

    fn attacks_for(fen: &str) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        gen_bishop_attacks(&board)
    }

    #[test]
    fn bishop_open_board_move_count() {
        // Bishop on e4 — 13 diagonal squares on empty board
        let list = moves_for("8/8/8/8/4B3/8/8/8 w - - 0 1");
        assert_eq!(list.len(), 13);
    }

    #[test]
    fn bishop_blocked_by_own_pieces() {
        // Bishop on e4, own pawns on d5 and f5
        let list = moves_for("8/8/8/3P1P2/4B3/8/8/8 w - - 0 1");
        assert!(!has_move(&list, Square::E4, Square::D5), "cannot go to d5 (own piece)");
        assert!(!has_move(&list, Square::E4, Square::F5), "cannot go to f5 (own piece)");
        assert!(!has_move(&list, Square::E4, Square::C6), "blocked behind d5");
        assert!(!has_move(&list, Square::E4, Square::G6), "blocked behind f5");
        assert!(has_move(&list, Square::E4, Square::D3), "d3 reachable");
        assert!(has_move(&list, Square::E4, Square::F3), "f3 reachable");
    }

    #[test]
    fn bishop_captures_enemy() {
        // Bishop on e4, enemy on c6 and g6
        let list = moves_for("8/8/2p3p1/8/4B3/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E4, Square::C6), "can capture c6");
        assert!(has_move(&list, Square::E4, Square::G6), "can capture g6");
        assert!(!has_move(&list, Square::E4, Square::B7), "blocked behind c6");
        assert!(!has_move(&list, Square::E4, Square::H7), "blocked behind g6");
    }

    #[test]
    fn gen_attacks_single_bishop_empty_board() {
        // Bishop on e4 — 13 diagonal squares
        let attacks = attacks_for("8/8/8/8/4B3/8/8/8 w - - 0 1");
        assert_eq!(attacks.count_ones(), 13);
        assert!(attacks & Square::H7.to_bitboard() != 0, "h7 controlled (NE)");
        assert!(attacks & Square::A8.to_bitboard() != 0, "a8 controlled (NW)");
        assert!(attacks & Square::H1.to_bitboard() != 0, "h1 controlled (SE)");
        assert!(attacks & Square::B1.to_bitboard() != 0, "b1 controlled (SW)");
        assert!(attacks & Square::E4.to_bitboard() == 0, "e4 not in own attacks");
    }

    #[test]
    fn gen_attacks_includes_own_piece_square() {
        // Own pawn on g6 blocks NE ray — g6 still controlled, h7 not
        let attacks = attacks_for("8/8/6P1/8/4B3/8/8/8 w - - 0 1");
        assert!(attacks & Square::F5.to_bitboard() != 0, "f5 controlled");
        assert!(attacks & Square::G6.to_bitboard() != 0, "g6 controlled (blocker)");
        assert!(attacks & Square::H7.to_bitboard() == 0, "h7 not controlled (behind own piece)");
    }

    #[test]
    fn gen_attacks_union_of_two_bishops() {
        // Bishops on c1 and f1 — different diagonals, no overlap on empty board
        let attacks = attacks_for("8/8/8/8/8/8/8/2B2B2 w - - 0 1");
        assert!(attacks & Square::A3.to_bitboard() != 0, "a3 controlled by c1");
        assert!(attacks & Square::H3.to_bitboard() != 0, "h3 controlled by f1");
        assert!(attacks & Square::E2.to_bitboard() != 0, "e2 controlled by f1");
        assert!(attacks & Square::D2.to_bitboard() != 0, "d2 controlled by c1");
    }

    #[test]
    fn gen_attacks_black_bishop() {
        let attacks = attacks_for("8/8/8/8/4b3/8/8/8 b - - 0 1");
        assert_eq!(attacks.count_ones(), 13);
    }

    #[test]
    fn table_matches_slow_on_empty_board() {
        let table = MagicTable::new(bishop_blocker_mask, bishop_attacks_slow);
        for sq in 0..64u8 {
            let square = Square(sq);
            let expected = bishop_attacks_slow(square, 0);
            let got = table.attacks(square, 0);
            assert_eq!(got, expected, "empty board mismatch on {}", square);
        }
    }

    #[test]
    fn table_matches_slow_with_blockers() {
        let table = MagicTable::new(bishop_blocker_mask, bishop_attacks_slow);
        let occupancy = Square::G6.to_bitboard() | Square::C6.to_bitboard();

        for sq in 0..64u8 {
            let square = Square(sq);
            let expected = bishop_attacks_slow(square, occupancy);
            let got = table.attacks(square, occupancy);
            assert_eq!(got, expected, "blocker mismatch on {}", square);
        }
    }
}
