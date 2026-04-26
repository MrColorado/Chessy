use crate::engine::board::BitBoard;
use crate::engine::board::Board;
use crate::engine::mv::{Move, Movelist};
use crate::engine::square::Square;

mod magic;
mod king;
mod knight;
mod pawn;
mod rook;
mod bishop;
mod queen;

use king::gen_king_moves;
use knight::{gen_knight_attacks, gen_knight_moves};
use pawn::{gen_pawn_attacks, gen_pawn_moves};
use rook::{gen_rook_attacks, gen_rook_moves};
use bishop::{gen_bishop_attacks, gen_bishop_moves};
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
    board: &Board,
    movelist: &mut Movelist,
) {
    let quiets = targets & !board.get_occupied_squares();
    for dest in extract_pieces_from_bitboard(quiets) {
        movelist.add_move(Move::quiet(from, dest));
    }
}

pub(super) fn add_capture_moves(
    from: Square,
    targets: BitBoard,
    board: &Board,
    movelist: &mut Movelist,
) {
    let captures = targets & board.get_occupied_squares_for_color(board.get_opponent_color());
    for dest in extract_pieces_from_bitboard(captures) {
        movelist.add_move(Move::capture(from, dest));
    }
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

pub fn gen_attacks(board: &Board) -> BitBoard {
    let mut attacks = 0u64;
    attacks |= gen_pawn_attacks(board);
    attacks |= gen_knight_attacks(board);
    attacks |= gen_rook_attacks(board);
    attacks |= gen_bishop_attacks(board);
    attacks |= gen_queen_attacks(board);
    attacks
}
