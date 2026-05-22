use crate::engine::board;
use crate::engine::board::BitBoard;
use crate::engine::board::Board;
use crate::engine::mv::{Move, Movelist};
use crate::engine::piece::{Color, Piece};
use crate::engine::square::Direction;
use crate::engine::square::{ALL_DIRS, ALL_SQUARES, Square};

mod bishop;
mod king;
mod knight;
mod magic;
mod pawn;
mod queen;
mod rook;

use bishop::{bishop_attack_from, gen_bishop_attacks, gen_bishop_moves};
use king::{gen_king_attacks, gen_king_moves};
use knight::{gen_knight_attacks, gen_knight_moves, knights_attack_from};
use pawn::{gen_pawn_attacks, gen_pawn_moves, pawn_attack_from};
use queen::{gen_queen_attacks, gen_queen_moves};
use rook::{gen_rook_attacks, gen_rook_moves, rook_attack_from};

const RANK_8: BitBoard = 0xFF00000000000000;
const RANK_7: BitBoard = 0x00FF000000000000;
const RANK_2: BitBoard = 0x000000000000FF00;
const RANK_1: BitBoard = 0x00000000000000FF;

const NOT_A_FILE: BitBoard = 0xFEFEFEFEFEFEFEFE;
const NOT_H_FILE: BitBoard = 0x7F7F7F7F7F7F7F7F;
const NOT_GH_FILE: BitBoard = 0x3F3F3F3F3F3F3F3F;
const NOT_AB_FILE: BitBoard = 0xFCFCFCFCFCFCFCFC;

const RAYS: [[u64; 8]; 64] = compute_rays();

const fn compute_rays() -> [[u64; 8]; 64] {
    let mut table = [[0u64; 8]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mut dir = 0;
        while dir < 8 {
            let mut cur = ALL_SQUARES[sq];
            while let Some(next) = cur.step(ALL_DIRS[dir]) {
                if next.step(ALL_DIRS[dir]).is_none() {
                    break;
                }
                table[sq][dir] |= next.to_bitboard();
                cur = next;
            }
            dir += 1;
        }
        sq += 1;
    }
    table
}

struct BitBoardIter(u64);

impl Iterator for BitBoardIter {
    type Item = Square;
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }
        let index = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(Square(index))
    }
}

fn extract_pieces_from_bitboard(bitboard: u64) -> BitBoardIter {
    BitBoardIter(bitboard)
}

pub(super) fn add_quiet_moves(
    from: Square,
    targets: BitBoard,
    piece: Piece,
    board: &Board,
    movelist: &mut Movelist,
) {
    let quiets = targets & !board.get_occupied_squares();
    for dest in extract_pieces_from_bitboard(quiets) {
        movelist.add_move(Move::quiet(from, dest, piece));
    }
}

pub(super) fn add_capture_moves(
    from: Square,
    targets: BitBoard,
    piece: Piece,
    board: &Board,
    movelist: &mut Movelist,
) {
    let captures = targets & board.get_occupied_squares_for_color(board.get_opponent_color());
    for dest in extract_pieces_from_bitboard(captures) {
        movelist.add_move(Move::capture(from, dest, piece));
    }
}

pub fn gen_attacks_for_color(board: &Board, color: Color) -> BitBoard {
    let occupancy = board.get_occupied_squares();
    let mut attacks = 0u64;
    attacks |= gen_pawn_attacks(board, color);
    attacks |= gen_knight_attacks(board, color);
    attacks |= gen_king_attacks(board, color);
    attacks |= gen_rook_attacks(board, color, occupancy);
    attacks |= gen_bishop_attacks(board, color, occupancy);
    attacks |= gen_queen_attacks(board, color, occupancy);
    attacks
}

pub fn gen_moves(board: &Board) -> Movelist {
    let mut movelist = Movelist::new();
    gen_pawn_moves(board, &mut movelist);
    gen_knight_moves(board, &mut movelist);
    gen_king_moves(board, &mut movelist);
    gen_rook_moves(board, &mut movelist);
    gen_bishop_moves(board, &mut movelist);
    gen_queen_moves(board, &mut movelist);
    movelist
}

pub fn compute_danger_squares(board: &Board, color: Color) -> BitBoard {
    let occ_no_king =
        board.get_occupied_squares() & !board.get_bitboard_for_piece(Piece::King, color);
    let attacker_color = color.flip();
    let mut dangers = 0u64;
    dangers |= gen_pawn_attacks(board, attacker_color);
    dangers |= gen_knight_attacks(board, attacker_color);
    dangers |= gen_king_attacks(board, attacker_color);
    dangers |= gen_bishop_attacks(board, attacker_color, occ_no_king);
    dangers |= gen_rook_attacks(board, attacker_color, occ_no_king);
    dangers |= gen_queen_attacks(board, attacker_color, occ_no_king);
    dangers
}

pub fn compute_checker(board: &Board, color: Color) -> BitBoard {
    let opponent_color = color.flip();
    let occupancy = board.get_occupied_squares();
    let king_bb = board.get_king_square(color);
    let mut checkers = 0u64;

    checkers |= pawn_attack_from(king_bb, color)
        & board.get_bitboard_for_piece(Piece::Pawn, opponent_color);
    checkers |=
        knights_attack_from(king_bb) & board.get_bitboard_for_piece(Piece::Knight, opponent_color);
    checkers |= bishop_attack_from(king_bb, occupancy)
        & (board.get_bitboard_for_piece(Piece::Bishop, opponent_color)
            | board.get_bitboard_for_piece(Piece::Queen, opponent_color));
    checkers |= rook_attack_from(king_bb, occupancy)
        & (board.get_bitboard_for_piece(Piece::Rook, opponent_color)
            | board.get_bitboard_for_piece(Piece::Queen, opponent_color));

    checkers
}

pub fn compute_pinned(board: &Board, color: Color) -> (u64, [u64; 64]) {
    let mut pin_mask = [u64::MAX; 64];
    let mut pinned = 0u64;
    let enemie_color = color.flip();
    let king_rays = RAYS[board.get_king_square(color).to_index()];
    let enemies_bishops = board.get_bitboard_for_piece(Piece::Bishop, enemie_color)
        | board.get_bitboard_for_piece(Piece::Queen, enemie_color);
    let enemies_rooks = board.get_bitboard_for_piece(Piece::Rook, enemie_color)
        | board.get_bitboard_for_piece(Piece::Queen, enemie_color);
    let occupancy =
        board.get_occupied_squares() & !board.get_bitboard_for_piece(Piece::King, color);
    let allies = board.get_occupied_squares_for_color(color);

    for ray_idx in 0..8 {
        let ray = king_rays[ray_idx];
        let is_diagonal = ray_idx % 2 == 1;
        let attacker_bb = if is_diagonal {
            enemies_bishops
        } else {
            enemies_rooks
        };

        // No piece are pinned we can continue
        if ray & attacker_bb == 0 {
            continue;
        }

        let (pin_ray, pinned_piece_pos) =
            compute_pinned_for_direction(ray, occupancy, attacker_bb, allies, ALL_DIRS[ray_idx]);
        if pin_ray != 0 {
            pin_mask[pinned_piece_pos] = pin_ray;
            pinned |= 1u64 << pinned_piece_pos;
        }
    }

    (pinned, pin_mask)
}

fn compute_pinned_for_direction(
    ray: u64,
    occupancy_bb: u64,
    attacker_bb: u64,
    allies_bb: u64,
    dir: Direction,
) -> (u64, usize) {
    let mut blockers = occupancy_bb & ray;
    let is_upward = dir.is_upward();

    let first_blocker = if is_upward {
        blockers.trailing_zeros() as usize
    } else {
        63 - blockers.leading_zeros() as usize
    };
    if allies_bb & (1 << first_blocker) == 0 {
        return (0, 0);
    }

    blockers = blockers & !(1 << first_blocker);

    let second_blocker = if is_upward {
        blockers.trailing_zeros() as usize
    } else {
        63 - blockers.leading_zeros() as usize
    };
    if attacker_bb & (1 << second_blocker) == 0 {
        return (0, 0);
    }

    if is_upward {
        (ray & (u64::MAX >> (63 - second_blocker)), first_blocker)
    } else {
        (ray & (u64::MAX << second_blocker), first_blocker)
    }
}

#[cfg(test)]
mod danger_squares_tests {
    use super::*;

    fn dangers(fen: &str, color: Color) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        compute_danger_squares(&board, color)
    }

    #[test]
    fn pawn_attacks() {
        // Black pawn on d5 attacks c4 and e4 — dangerous for White
        let bb = dangers("8/8/8/3p4/8/8/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::C4.to_bitboard() != 0);
        assert!(bb & Square::E4.to_bitboard() != 0);
    }

    #[test]
    fn knight_attacks() {
        // Black knight on d4 attacks b3, b5, c2, c6, e2, e6, f3, f5 — dangerous for White
        let bb = dangers("8/8/8/8/3n4/8/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::B3.to_bitboard() != 0);
        assert!(bb & Square::F5.to_bitboard() != 0);
        assert!(bb & Square::C2.to_bitboard() != 0);
        assert!(bb & Square::E6.to_bitboard() != 0);
    }

    #[test]
    fn bishop_attacks() {
        // Black bishop on c3 attacks b2, a1, d4, e5, f6, g7, h8, b4, a5, d2, e1 — dangerous for White
        let bb = dangers("8/8/8/8/8/2b5/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::B2.to_bitboard() != 0);
        assert!(bb & Square::E5.to_bitboard() != 0);
        assert!(bb & Square::D4.to_bitboard() != 0);
        assert!(bb & Square::A1.to_bitboard() != 0);
    }

    #[test]
    fn rook_attacks() {
        // Black rook on d4 attacks the whole d-file and 4th rank — dangerous for White
        let bb = dangers("8/8/8/8/3r4/8/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::D1.to_bitboard() != 0);
        assert!(bb & Square::D8.to_bitboard() != 0);
        assert!(bb & Square::A4.to_bitboard() != 0);
        assert!(bb & Square::H4.to_bitboard() != 0);
    }

    #[test]
    fn queen_attacks() {
        // Black queen on d4: combines rook + bishop rays — dangerous for White
        let bb = dangers("8/8/8/8/3q4/8/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::D1.to_bitboard() != 0);
        assert!(bb & Square::H8.to_bitboard() != 0);
        assert!(bb & Square::A4.to_bitboard() != 0);
        assert!(bb & Square::G7.to_bitboard() != 0);
    }

    #[test]
    fn king_attacks() {
        // Black king on e8 attacks d8, f8, d7, e7, f7 — dangerous for White
        let bb = dangers("4k3/8/8/8/8/8/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::D7.to_bitboard() != 0);
        assert!(bb & Square::E7.to_bitboard() != 0);
        assert!(bb & Square::F7.to_bitboard() != 0);
        assert!(bb & Square::D8.to_bitboard() != 0);
        assert!(bb & Square::F8.to_bitboard() != 0);
    }

    #[test]
    fn slider_xrays_through_own_king() {
        // White king on e1; black rook on e8. The white king must be excluded from
        // occupancy so the rook's ray continues through e1 — squares behind the
        // king are still marked as dangerous, preventing the king from stepping there.
        let bb = dangers("4r3/8/8/8/8/8/8/4K3 b - - 0 1", Color::White);
        assert!(bb & Square::E1.to_bitboard() != 0);
    }

    #[test]
    fn multiple_pieces_union() {
        // Black rook on a1 and black bishop on h8: both rays should appear — dangerous for White
        let bb = dangers("7b/8/8/8/8/8/8/r3K3 b - - 0 1", Color::White);
        assert!(bb & Square::E1.to_bitboard() != 0); // rook on rank 1
        assert!(bb & Square::G7.to_bitboard() != 0); // bishop diagonal
    }
}

#[cfg(test)]
mod checker_tests {
    use super::*;

    fn checkers(fen: &str, color: Color) -> BitBoard {
        let board = Board::from_fen(fen).expect("valid FEN");
        compute_checker(&board, color)
    }

    #[test]
    fn no_check() {
        let bb = checkers("8/8/8/8/8/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, 0);
    }

    #[test]
    fn knight_check() {
        // Black knight on d3 attacks e1
        let bb = checkers("8/8/8/8/8/3n4/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, Square::D3.to_bitboard());
    }

    #[test]
    fn pawn_check() {
        // Black pawn on d5 attacks e4 (white king on e4)
        let bb = checkers("8/8/8/3p4/4K3/8/8/8 w - - 0 1", Color::White);
        assert_eq!(bb, Square::D5.to_bitboard());
    }

    #[test]
    fn rook_check_file() {
        // Black rook on e8 attacks e1 along the file
        let bb = checkers("4r3/8/8/8/8/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, Square::E8.to_bitboard());
    }

    #[test]
    fn rook_check_rank() {
        // Black rook on a1 attacks e1 along the rank
        let bb = checkers("8/8/8/8/8/8/8/r3K3 w - - 0 1", Color::White);
        assert_eq!(bb, Square::A1.to_bitboard());
    }

    #[test]
    fn bishop_check() {
        // Black bishop on b4 attacks e1 along the diagonal
        let bb = checkers("8/8/8/8/1b6/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, Square::B4.to_bitboard());
    }

    #[test]
    fn queen_check_file() {
        // Black queen on e8 attacks e1 along the file
        let bb = checkers("4q3/8/8/8/8/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, Square::E8.to_bitboard());
    }

    #[test]
    fn queen_check_diagonal() {
        // Black queen on b4 attacks e1 along the diagonal
        let bb = checkers("8/8/8/8/1q6/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, Square::B4.to_bitboard());
    }

    #[test]
    fn double_check() {
        // Black rook on e8 + black knight on d3 both attack e1
        let bb = checkers("4r3/8/8/8/8/3n4/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb.count_ones(), 2);
        assert!(bb & Square::E8.to_bitboard() != 0);
        assert!(bb & Square::D3.to_bitboard() != 0);
    }

    #[test]
    fn black_king_in_check() {
        // White rook on e1 attacks black king on e8
        let bb = checkers("4k3/8/8/8/8/8/8/4R3 b - - 0 1", Color::Black);
        assert_eq!(bb, Square::E1.to_bitboard());
    }

    #[test]
    fn blocker_prevents_check() {
        // Black rook on e8, white pawn on e4 blocks — no check on white king e1
        let bb = checkers("4r3/8/8/8/4P3/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(bb, 0);
    }
}

#[cfg(test)]
mod pinned_tests {
    use super::*;

    // --- compute_pinned_for_direction unit tests ---

    #[test]
    fn no_pin_no_ally_on_ray() {
        // Enemy on ray, no ally between king and enemy → no pin
        let ray = RAYS[Square::E1.to_index()][0]; // N from e1
        let (mask, _) = compute_pinned_for_direction(
            ray,
            Square::E7.to_bitboard(),
            Square::E7.to_bitboard(),
            0,
            Direction::N,
        );
        assert_eq!(mask, 0);
    }

    #[test]
    fn no_pin_first_blocker_is_enemy() {
        // First piece on ray is enemy, not ally → no pin
        let ray = RAYS[Square::E1.to_index()][0]; // N from e1
        let occ = Square::E4.to_bitboard() | Square::E7.to_bitboard();
        let (mask, _) = compute_pinned_for_direction(
            ray,
            occ,
            occ,  // both are attackers
            Square::E7.to_bitboard(), // ally is only e7 (behind attacker e4)
            Direction::N,
        );
        assert_eq!(mask, 0);
    }

    #[test]
    fn no_pin_two_allies_no_attacker() {
        // Two allies on ray but no attacker behind → no pin
        let ray = RAYS[Square::E1.to_index()][0]; // N from e1
        let occ = Square::E4.to_bitboard() | Square::E7.to_bitboard();
        let (mask, _) = compute_pinned_for_direction(
            ray,
            occ,
            0,   // no attacker
            occ, // both are allies
            Direction::N,
        );
        assert_eq!(mask, 0);
    }

    #[test]
    fn pin_north() {
        // King e1, ally pawn e5, enemy rook e7 — pin along N ray
        let ray = RAYS[Square::E1.to_index()][0]; // N from e1
        let occ = Square::E5.to_bitboard() | Square::E7.to_bitboard();
        let (mask, pos) = compute_pinned_for_direction(
            ray,
            occ,
            Square::E7.to_bitboard(),
            Square::E5.to_bitboard(),
            Direction::N,
        );
        assert_eq!(pos, Square::E5.to_index());
        assert_ne!(mask & Square::E4.to_bitboard(), 0); // pinned piece can retreat toward king
        assert_ne!(mask & Square::E6.to_bitboard(), 0); // square between ally and attacker
        assert_ne!(mask & Square::E7.to_bitboard(), 0); // can capture attacker
        assert_eq!(mask & Square::E8.to_bitboard(), 0); // beyond attacker (edge, excluded from ray)
        assert_eq!(mask & Square::D5.to_bitboard(), 0); // off the N ray
    }

    #[test]
    fn pin_northwest_diagonal() {
        // King e1, ally pawn d2, enemy bishop c3 — pin along NW diagonal
        // RAYS[e1][NW=7] = {d2, c3, b4}  (a5 is edge, excluded)
        let ray = RAYS[Square::E1.to_index()][7]; // NW from e1
        let occ = Square::D2.to_bitboard() | Square::C3.to_bitboard();
        let (mask, pos) = compute_pinned_for_direction(
            ray,
            occ,
            Square::C3.to_bitboard(),
            Square::D2.to_bitboard(),
            Direction::NW,
        );
        assert_eq!(pos, Square::D2.to_index());
        assert_ne!(mask & Square::C3.to_bitboard(), 0); // can capture bishop
        assert_eq!(mask & Square::B4.to_bitboard(), 0); // beyond attacker
    }

    // --- compute_pinned integration tests via FEN ---

    fn pinned(fen: &str, color: Color) -> (u64, [u64; 64]) {
        let board = Board::from_fen(fen).expect("valid FEN");
        compute_pinned(&board, color)
    }

    #[test]
    fn no_pins() {
        let (pinned_bb, pin_mask) = pinned("8/8/8/8/8/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(pinned_bb, 0);
        assert!(pin_mask.iter().all(|&m| m == u64::MAX));
    }

    #[test]
    fn rook_pin_north() {
        // King e1, white pawn e5, black rook e7 — pawn pinned on N ray
        // Note: e8 is excluded from RAYS, so attacker must be on e7
        let (pinned_bb, pin_mask) = pinned("8/4r3/8/4P3/8/8/8/4K3 w - - 0 1", Color::White);
        assert_ne!(pinned_bb & Square::E5.to_bitboard(), 0);
        assert_ne!(pin_mask[Square::E5.to_index()] & Square::E7.to_bitboard(), 0); // can capture
        assert_ne!(pin_mask[Square::E5.to_index()] & Square::E6.to_bitboard(), 0); // intermediate square
        assert_eq!(pin_mask[Square::E5.to_index()] & Square::D5.to_bitboard(), 0); // off-ray
    }

    #[test]
    fn bishop_pin_northwest() {
        // King e1, white pawn d2, black bishop c3
        let (pinned_bb, pin_mask) = pinned("8/8/8/8/8/2b5/3P4/4K3 w - - 0 1", Color::White);
        assert_ne!(pinned_bb & Square::D2.to_bitboard(), 0);
        assert_ne!(pin_mask[Square::D2.to_index()] & Square::C3.to_bitboard(), 0); // can capture
        assert_eq!(pin_mask[Square::D2.to_index()] & Square::E3.to_bitboard(), 0); // off-ray
    }

    #[test]
    fn two_simultaneous_pins() {
        // King e1, pawns d2 (NW pin by bishop c3) and e2 (N pin by rook e7)
        let (pinned_bb, pin_mask) = pinned("8/4r3/8/8/8/2b5/3PP3/4K3 w - - 0 1", Color::White);
        assert_ne!(pinned_bb & Square::D2.to_bitboard(), 0);
        assert_ne!(pinned_bb & Square::E2.to_bitboard(), 0);
        assert_ne!(pin_mask[Square::D2.to_index()] & Square::C3.to_bitboard(), 0);
        assert_ne!(pin_mask[Square::E2.to_index()] & Square::E7.to_bitboard(), 0);
    }

    #[test]
    fn two_allies_block_pin() {
        // Two white pawns e4 and e5 between king e1 and black rook e7 → no pin
        let (pinned_bb, _) = pinned("8/4r3/8/4P3/4P3/8/8/4K3 w - - 0 1", Color::White);
        assert_eq!(pinned_bb & Square::E4.to_bitboard(), 0);
        assert_eq!(pinned_bb & Square::E5.to_bitboard(), 0);
    }
}
