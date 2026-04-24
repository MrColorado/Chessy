use crate::engine::board::BitBoard;
use crate::engine::board::Board;
use crate::engine::mv::Movelist;
use crate::engine::square::Square;

mod pawn;
mod knight;

use pawn::gen_pawn_moves;
use knight::gen_knight_moves;

const RANK_8: BitBoard = 0xFF00000000000000;
const RANK_7: BitBoard = 0x00FF000000000000;
const RANK_2: BitBoard = 0x000000000000FF00;
const RANK_1: BitBoard = 0x00000000000000FF;

const NOT_A_FILE: BitBoard = 0xFEFEFEFEFEFEFEFE;
const NOT_H_FILE: BitBoard = 0x7F7F7F7F7F7F7F7F;
const NOT_GH_FILE: BitBoard = 0x3F3F3F3F3F3F3F3F;
const NOT_AB_FILE: BitBoard = 0xFCFCFCFCFCFCFCFC;

fn extract_pieces_from_bitboard(mut bitboard: u64) -> Vec<Square> {
    let mut pieces = Vec::new();
    while bitboard != 0 {
        let index = bitboard.trailing_zeros() as usize;
        pieces.push(Square(index as u8));
        bitboard &= bitboard - 1;
    }
    pieces
}

pub fn gen_moves(board: &Board) -> Movelist {
    let mut movelist = Movelist::new();
    gen_pawn_moves(board, &mut movelist);
    gen_knight_moves(board, &mut movelist);
    movelist
}
