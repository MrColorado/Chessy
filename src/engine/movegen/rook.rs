use std::sync::OnceLock;

use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::Movelist;
use crate::engine::piece::Piece;
use crate::engine::square::Square;

use super::{add_capture_moves, add_quiet_moves, extract_pieces_from_bitboard};

static TABLE: OnceLock<RookTable> = OnceLock::new();

fn get_table() -> &'static RookTable {
    TABLE.get_or_init(RookTable::new)
}

pub(super) fn gen_rook_moves(board: &Board, movelist: &mut Movelist) {
    let table    = get_table();
    let color    = board.get_current_player_color();
    let rooks    = board.get_bitboard_for_piece(Piece::Rook, color);
    let occupancy = board.get_occupied_squares();

    for sq in extract_pieces_from_bitboard(rooks) {
        let targets = table.attacks(sq, occupancy);
        add_quiet_moves(sq, targets, board, movelist);
        add_capture_moves(sq, targets, board, movelist);
    }
}

pub(super) fn gen_rook_attacks(board: &Board) -> BitBoard {
    let table     = get_table();
    let color     = board.get_current_player_color();
    let rooks     = board.get_bitboard_for_piece(Piece::Rook, color);
    let occupancy = board.get_occupied_squares();
    let mut attacks = 0u64;

    for sq in extract_pieces_from_bitboard(rooks) {
        attacks |= table.attacks(sq, occupancy);
    }
    
    attacks
}

struct RookTable {
    masks:   [BitBoard; 64],
    magics:  [u64; 64],
    shifts:  [u32; 64],
    attacks: [Vec<BitBoard>; 64],
}

// Simple xorshift64 RNG — no external dependency needed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    // AND of three randoms produces sparse numbers — better magic candidates.
    fn sparse(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}

impl RookTable {
    pub fn new() -> Self {
        let mut masks  = [0u64; 64];
        let mut magics = [0u64; 64];
        let mut shifts = [0u32; 64];
        let mut attacks: [Vec<BitBoard>; 64] = std::array::from_fn(|_| Vec::new());

        let mut rng = Rng(1_070_372);

        for sq in 0..64u8 {
            let square = Square(sq);
            let mask   = rook_blocker_mask(square);
            let n      = mask.count_ones();
            let shift  = 64 - n;
            let size   = 1usize << n;

            masks[sq as usize]  = mask;
            shifts[sq as usize] = shift;

            let mut occ_list = vec![0u64; size];
            let mut att_list = vec![0u64; size];
            let mut subset   = 0u64;
            for i in 0..size {
                occ_list[i] = subset;
                att_list[i] = rook_attacks_slow(square, subset);
                subset = subset.wrapping_sub(mask) & mask;
            }

            // Find a magic number with no index collisions.
            'search: loop {
                let magic = rng.sparse();

                // Quick filter: magic must spread bits into the high byte.
                if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 {
                    continue;
                }

                let mut table = vec![0u64; size];
                for i in 0..size {
                    let index = (occ_list[i].wrapping_mul(magic) >> shift) as usize;
                    if table[index] == 0 {
                        table[index] = att_list[i];
                    } else if table[index] != att_list[i] {
                        continue 'search; // collision — try another magic
                    }
                }

                magics[sq as usize]  = magic;
                attacks[sq as usize] = table;
                break;
            }
        }

        RookTable { masks, magics, shifts, attacks }
    }

    fn attacks(&self, sq: Square, occupancy: BitBoard) -> BitBoard {
        let i       = sq.to_index();
        let relevant = occupancy & self.masks[i];
        let index   = relevant.wrapping_mul(self.magics[i]) >> self.shifts[i];
        self.attacks[i][index as usize]
    }
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
        if occupancy & bit != 0 { break; }
    }
    // South
    if r > 0 {
        for rank in (0..r).rev() {
            let bit = 1u64 << (rank * 8 + f);
            attacks |= bit;
            if occupancy & bit != 0 { break; }
        }
    }
    // East
    for file in (f + 1)..8 {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occupancy & bit != 0 { break; }
    }
    // West
    if f > 0 {
        for file in (0..f).rev() {
            let bit = 1u64 << (r * 8 + file);
            attacks |= bit;
            if occupancy & bit != 0 { break; }
        }
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(mask & Square::E5.to_bitboard() != 0, "e5 should be in the mask");
        assert!(mask & Square::E6.to_bitboard() != 0, "e6 should be in the mask");
        assert!(mask & Square::E7.to_bitboard() != 0, "e7 should be in the mask");
        assert!(mask & Square::E8.to_bitboard() == 0, "e8 should not be in the mask (edge)");

        // South
        assert!(mask & Square::E2.to_bitboard() != 0, "e2 should be in the mask");
        assert!(mask & Square::E3.to_bitboard() != 0, "e3 should be in the mask");
        assert!(mask & Square::E1.to_bitboard() == 0, "e1 should not be in the mask (edge)");

        // East
        assert!(mask & Square::F4.to_bitboard() != 0, "f4 should be in the mask");
        assert!(mask & Square::G4.to_bitboard() != 0, "g4 should be in the mask");
        assert!(mask & Square::H4.to_bitboard() == 0, "h4 should not be in the mask (edge)");

        // West
        assert!(mask & Square::B4.to_bitboard() != 0, "b4 should be in the mask");
        assert!(mask & Square::C4.to_bitboard() != 0, "c4 should be in the mask");
        assert!(mask & Square::D4.to_bitboard() != 0, "d4 should be in the mask");
        assert!(mask & Square::A4.to_bitboard() == 0, "a4 should not be in the mask (edge)");

        // e4 itself excluded
        assert!(mask & Square::E4.to_bitboard() == 0, "e4 should not be in its own mask");

        assert_eq!(mask.count_ones(), 10, "e4 mask should have 10 bits set");
    }

    #[test]
    fn attacks_empty_board() {
        let attacks = rook_attacks_slow(Square::E4, 0);
        assert!(attacks & Square::E8.to_bitboard() != 0, "e8 reachable on empty board");
        assert!(attacks & Square::E1.to_bitboard() != 0, "e1 reachable on empty board");
        assert!(attacks & Square::A4.to_bitboard() != 0, "a4 reachable on empty board");
        assert!(attacks & Square::H4.to_bitboard() != 0, "h4 reachable on empty board");
        assert!(attacks & Square::E4.to_bitboard() == 0, "e4 itself not in attacks");
    }

    #[test]
    fn attacks_blocked_north_and_east() {
        // Blockers on e6 and g4
        let occupancy = Square::E6.to_bitboard() | Square::G4.to_bitboard();
        let attacks = rook_attacks_slow(Square::E4, occupancy);

        // North: e5 and e6 reachable, e7 blocked
        assert!(attacks & Square::E5.to_bitboard() != 0, "e5 reachable");
        assert!(attacks & Square::E6.to_bitboard() != 0, "e6 reachable (capture the blocker)");
        assert!(attacks & Square::E7.to_bitboard() == 0, "e7 blocked");

        // East: f4 and g4 reachable, h4 blocked
        assert!(attacks & Square::F4.to_bitboard() != 0, "f4 reachable");
        assert!(attacks & Square::G4.to_bitboard() != 0, "g4 reachable (capture the blocker)");
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

    fn moves_for(fen: &str) -> Movelist {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut list = Movelist::new();
        gen_rook_moves(&board, &mut list);
        list
    }

    fn has_move(list: &Movelist, from: Square, to: Square) -> bool {
        list.as_slice().iter().any(|m| m.from() == from && m.to() == to)
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
        assert!(!has_move(&list, Square::E4, Square::E5), "cannot capture own piece on e5");
        assert!(!has_move(&list, Square::E4, Square::G4), "cannot capture own piece on g4");
        assert!(!has_move(&list, Square::E4, Square::H4), "blocked behind own piece on g4");
        assert!(!has_move(&list, Square::E4, Square::E6), "blocked behind own piece on e5");
        assert!(has_move(&list, Square::E4, Square::F4),  "f4 is empty and reachable");
    }

    #[test]
    fn rook_captures_enemy() {
        // Rook on e4, enemy pieces on e6 and b4
        let list = moves_for("8/8/4p3/8/1p2R3/8/8/8 w - - 0 1");
        assert!(has_move(&list, Square::E4, Square::E6), "can capture e6");
        assert!(has_move(&list, Square::E4, Square::B4), "can capture b4");
        assert!(!has_move(&list, Square::E4, Square::E7), "blocked behind enemy on e6");
        assert!(!has_move(&list, Square::E4, Square::A4), "blocked behind enemy on b4");
    }

    #[test]
    fn table_attacks_e4_blocked() {
        let table = RookTable::new();
        let occupancy = Square::E6.to_bitboard() | Square::G4.to_bitboard();
        let attacks = table.attacks(Square::E4, occupancy);

        // North: e5 and e6 reachable, e7 blocked
        assert!(attacks & Square::E5.to_bitboard() != 0, "e5 reachable");
        assert!(attacks & Square::E6.to_bitboard() != 0, "e6 reachable (capture)");
        assert!(attacks & Square::E7.to_bitboard() == 0, "e7 blocked");

        // East: f4 and g4 reachable, h4 blocked
        assert!(attacks & Square::F4.to_bitboard() != 0, "f4 reachable");
        assert!(attacks & Square::G4.to_bitboard() != 0, "g4 reachable (capture)");
        assert!(attacks & Square::H4.to_bitboard() == 0, "h4 blocked");

        // South and West: unobstructed
        assert!(attacks & Square::E1.to_bitboard() != 0, "e1 reachable");
        assert!(attacks & Square::A4.to_bitboard() != 0, "a4 reachable");

        // e4 itself never in attacks
        assert!(attacks & Square::E4.to_bitboard() == 0, "e4 not in own attacks");
    }

    #[test]
    fn table_matches_slow_on_empty_board() {
        let table = RookTable::new();
        for sq in 0..64u8 {
            let square = Square(sq);
            let expected = rook_attacks_slow(square, 0);
            let got      = table.attacks(square, 0);
            assert_eq!(got, expected, "empty board mismatch on {}", square);
        }
    }

    #[test]
    fn table_matches_slow_with_blockers() {
        let table = RookTable::new();
        let occupancy = Square::E6.to_bitboard() | Square::G4.to_bitboard() | Square::B1.to_bitboard();

        for sq in 0..64u8 {
            let square = Square(sq);
            let expected = rook_attacks_slow(square, occupancy);
            let got      = table.attacks(square, occupancy);
            assert_eq!(got, expected, "blocker mismatch on {}", square);
        }
    }
}
