use crate::engine::board::Board;
use crate::engine::mv::Movelist;
use crate::engine::square::Square;

mod pawn;

use pawn::gen_pawn_moves;

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
    gen_pawn_moves( board, &mut movelist);
    movelist
}
