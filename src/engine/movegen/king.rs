use crate::engine::board::{BitBoard, Board};
use crate::engine::mv::{Move, Movelist};
use crate::engine::piece::Piece;
use crate::engine::square::Square;

use super::extract_pieces_from_bitboard;
use super::{NOT_A_FILE, NOT_H_FILE};

pub(super) fn gen_king_moves(board: &Board, movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    let king = board.get_bitboard_for_piece(Piece::King, color);
    for pos in extract_pieces_from_bitboard(king) {
        let targets = king_targets(pos.to_bitboard());
        king_moves(pos, targets, board, movelist);
        king_attacks(pos, targets, board, movelist);
    }
    castle_moves(board, movelist); // TODO will need to check if king is in check, and if squares it passes through are attacked
}

fn king_targets(pos: BitBoard) -> BitBoard {
    let mut target = pos << 8; // Up
    target |= pos >> 8; // Down
    target |= (pos << 1) & NOT_A_FILE; // Right
    target |= (pos >> 1) & NOT_H_FILE; // Left
    target |= (pos << 9) & NOT_A_FILE; // Up Right
    target |= (pos << 7) & NOT_H_FILE; // Up Left
    target |= (pos >> 7) & NOT_A_FILE; // Down Right
    target |= (pos >> 9) & NOT_H_FILE; // Down Left
    return target;
}

fn king_moves(pos: Square, targets: BitBoard, board: &Board, movelist: &mut Movelist) {
    let occupied = board.get_occupied_squares();
    let mv_possible = targets & !occupied;
    for dest in extract_pieces_from_bitboard(mv_possible) {
        movelist.add_move(Move::quiet(pos, dest));
    }
}

fn king_attacks(pos: Square, targets: BitBoard, board: &Board, movelist: &mut Movelist) {
    let attacks = targets & board.get_occupied_squares_for_color(board.get_opponent_color());
    for dest in extract_pieces_from_bitboard(attacks) {
        movelist.add_move(Move::capture(pos, dest));
    }
}

fn castle_moves(board: &Board, movelist: &mut Movelist) {
    let color = board.get_current_player_color();
    if board.can_castle_kingside(color) {
        movelist.add_move(Move::kingside_castle(color));
    }
    if board.can_castle_queenside(color) {
        movelist.add_move(Move::queenside_castle(color));
    }
}
