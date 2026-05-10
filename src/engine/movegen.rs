use crate::engine::board::BitBoard;
use crate::engine::board::Board;
use crate::engine::mv::{Move, Movelist};
use crate::engine::square::Square;
use crate::engine::piece::{Color, Piece};

mod magic;
mod king;
mod knight;
mod pawn;
mod rook;
mod bishop;
mod queen;

use king::{gen_king_moves, gen_king_attacks};
use knight::{gen_knight_attacks, gen_knight_moves, knights_attack_from};
use pawn::{gen_pawn_attacks, gen_pawn_moves, pawn_attack_from};
use rook::{gen_rook_attacks, gen_rook_moves, rook_attack_from};
use bishop::{gen_bishop_attacks, gen_bishop_moves, bishop_attack_from};
use queen::{gen_queen_attacks, gen_queen_moves};

const RANK_8: BitBoard = 0xFF00000000000000;
const RANK_7: BitBoard = 0x00FF000000000000;
const RANK_2: BitBoard = 0x000000000000FF00;
const RANK_1: BitBoard = 0x00000000000000FF;

const NOT_A_FILE: BitBoard = 0xFEFEFEFEFEFEFEFE;
const NOT_H_FILE: BitBoard = 0x7F7F7F7F7F7F7F7F;
const NOT_GH_FILE: BitBoard = 0x3F3F3F3F3F3F3F3F;
const NOT_AB_FILE: BitBoard = 0xFCFCFCFCFCFCFCFC;

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
    let mut attacks = 0u64;
    attacks |= gen_pawn_attacks(board, color);
    attacks |= gen_knight_attacks(board, color);
    attacks |= gen_rook_attacks(board, color);
    attacks |= gen_bishop_attacks(board, color);
    attacks |= gen_queen_attacks(board, color);
    attacks |= gen_king_attacks(board, color);
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

pub fn compute_checker(board: &Board, color: Color) -> BitBoard {
    let opponent_color = color.flip();
    let occupancy = board.get_occupied_squares();
    let king_bb = board.get_king_square(color);
    let mut checkers = 0u64;

    checkers |= knights_attack_from(king_bb) & board.get_bitboard_for_piece(Piece::Knight, opponent_color);
    checkers |= pawn_attack_from(king_bb, color) & board.get_bitboard_for_piece(Piece::Pawn, opponent_color);
    checkers |= rook_attack_from(king_bb, occupancy) & 
        (board.get_bitboard_for_piece(Piece::Rook, opponent_color) | board.get_bitboard_for_piece(Piece::Queen, opponent_color));
    checkers |= bishop_attack_from(king_bb, occupancy) & 
        (board.get_bitboard_for_piece(Piece::Bishop, opponent_color) | board.get_bitboard_for_piece(Piece::Queen, opponent_color));

    checkers
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
