use std::default;

use crate::engine::mv::{IrreversibleState, Move, MoveKind};
use crate::engine::piece::{Color, Piece};
use crate::engine::square::Square;

pub type BitBoard = u64;

#[derive(Debug, Default, Clone, Copy)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const WHITE_KINGSIDE: u8 = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE: u8 = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.0 & CastlingRights::WHITE_KINGSIDE != 0,
            Color::Black => self.0 & CastlingRights::BLACK_KINGSIDE != 0,
        }
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.0 & CastlingRights::WHITE_QUEENSIDE != 0,
            Color::Black => self.0 & CastlingRights::BLACK_QUEENSIDE != 0,
        }
    }
}

const CASTLING_RIGHTS_MASK: [u8; 64] = {
    let mut m = [0b1111u8; 64];
    m[Square::E1.0 as usize] &= !(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
    m[Square::H1.0 as usize] &= !CastlingRights::WHITE_KINGSIDE;
    m[Square::A1.0 as usize] &= !CastlingRights::WHITE_QUEENSIDE;
    m[Square::E8.0 as usize] &= !(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
    m[Square::H8.0 as usize] &= !CastlingRights::BLACK_KINGSIDE;
    m[Square::A8.0 as usize] &= !CastlingRights::BLACK_QUEENSIDE;
    m
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Board {
    white_pawns: BitBoard,
    white_knights: BitBoard,
    white_bishops: BitBoard,
    white_rooks: BitBoard,
    white_queens: BitBoard,
    white_kings: BitBoard,

    black_pawns: BitBoard,
    black_knights: BitBoard,
    black_bishops: BitBoard,
    black_rooks: BitBoard,
    black_queens: BitBoard,
    black_kings: BitBoard,

    halfmove_clock: u8,
    fullmove_number: u8,
    white_to_move: bool,
    en_passant_square: Option<Square>,
    castling_rights: CastlingRights,
}

impl Board {
    pub fn new() -> Board {
        Board {
            white_pawns: 0x000000000000FF00,
            white_knights: 0x0000000000000042,
            white_bishops: 0x0000000000000024,
            white_rooks: 0x0000000000000081,
            white_queens: 0x0000000000000008,
            white_kings: 0x0000000000000010,

            black_pawns: 0x00FF000000000000,
            black_knights: 0x4200000000000000,
            black_bishops: 0x2400000000000000,
            black_rooks: 0x8100000000000000,
            black_queens: 0x0800000000000000,
            black_kings: 0x1000000000000000,

            halfmove_clock: 0,
            fullmove_number: 0,
            white_to_move: true,
            en_passant_square: None,
            castling_rights: CastlingRights(
                CastlingRights::WHITE_KINGSIDE
                    | CastlingRights::WHITE_QUEENSIDE
                    | CastlingRights::BLACK_KINGSIDE
                    | CastlingRights::BLACK_QUEENSIDE,
            ),
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let words = fen.split_whitespace().collect::<Vec<_>>();
        if words.len() != 6 {
            return Err(format!(
                "Invalid FEN: expected 6 fields, got {}",
                words.len()
            ));
        }

        let ranks = words[0].split('/').collect::<Vec<_>>();
        if ranks.len() != 8 {
            return Err(format!(
                "Invalid FEN: expected 8 ranks, got {}",
                ranks.len()
            ));
        }

        let mut board = Board::default();

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx;
            let mut file = 0usize;
            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    file += (ch as u8 - b'0') as usize;
                } else {
                    let bit = 1u64 << (rank * 8 + file);
                    match ch {
                        'P' => board.white_pawns |= bit,
                        'N' => board.white_knights |= bit,
                        'B' => board.white_bishops |= bit,
                        'R' => board.white_rooks |= bit,
                        'Q' => board.white_queens |= bit,
                        'K' => board.white_kings |= bit,
                        'p' => board.black_pawns |= bit,
                        'n' => board.black_knights |= bit,
                        'b' => board.black_bishops |= bit,
                        'r' => board.black_rooks |= bit,
                        'q' => board.black_queens |= bit,
                        'k' => board.black_kings |= bit,
                        _ => return Err(format!("Invalid FEN: unknown piece '{}'", ch)),
                    }
                    file += 1;
                }
            }
        }

        board.white_to_move = words[1] == "w";
        board.castling_rights = CastlingRights(words[2].chars().fold(0u8, |rights, ch| match ch {
            'K' => rights | CastlingRights::WHITE_KINGSIDE,
            'Q' => rights | CastlingRights::WHITE_QUEENSIDE,
            'k' => rights | CastlingRights::BLACK_KINGSIDE,
            'q' => rights | CastlingRights::BLACK_QUEENSIDE,
            _ => rights,
        }));
        board.en_passant_square = if words[3] != "-" {
            Some(Square::from_str(words[3]).ok_or("Invalid FEN: invalid en passant square")?)
        } else {
            None
        };
        board.halfmove_clock = words[4]
            .parse::<u8>()
            .map_err(|_| "Invalid FEN: invalid halfmove clock")?;
        board.fullmove_number = words[5]
            .parse::<u8>()
            .map_err(|_| "Invalid FEN: invalid fullmove number")?;

        Ok(board)
    }

    pub fn get_current_player_color(&self) -> Color {
        if self.white_to_move {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn get_opponent_color(&self) -> Color {
        if self.white_to_move {
            Color::Black
        } else {
            Color::White
        }
    }

    pub fn get_white_to_move(&self) -> bool {
        self.white_to_move
    }

    pub fn get_castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    pub fn get_en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    pub fn get_halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    pub fn get_fullmove_number(&self) -> u8 {
        self.fullmove_number
    }

    pub fn get_bitboard_for_piece(&self, piece: Piece, color: Color) -> BitBoard {
        match (piece, color) {
            (Piece::Pawn, Color::White) => self.white_pawns,
            (Piece::Knight, Color::White) => self.white_knights,
            (Piece::Bishop, Color::White) => self.white_bishops,
            (Piece::Rook, Color::White) => self.white_rooks,
            (Piece::Queen, Color::White) => self.white_queens,
            (Piece::King, Color::White) => self.white_kings,

            (Piece::Pawn, Color::Black) => self.black_pawns,
            (Piece::Knight, Color::Black) => self.black_knights,
            (Piece::Bishop, Color::Black) => self.black_bishops,
            (Piece::Rook, Color::Black) => self.black_rooks,
            (Piece::Queen, Color::Black) => self.black_queens,
            (Piece::King, Color::Black) => self.black_kings,
        }
    }

    pub fn get_occupied_squares(&self) -> BitBoard {
        self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_kings
            | self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_kings
    }

    pub fn get_empty_squares(&self) -> BitBoard {
        !self.get_occupied_squares()
    }

    pub fn get_occupied_squares_for_color(&self, color: Color) -> BitBoard {
        match color {
            Color::White => {
                self.white_pawns
                    | self.white_knights
                    | self.white_bishops
                    | self.white_rooks
                    | self.white_queens
                    | self.white_kings
            }
            Color::Black => {
                self.black_pawns
                    | self.black_knights
                    | self.black_bishops
                    | self.black_rooks
                    | self.black_queens
                    | self.black_kings
            }
        }
    }

    pub fn get_piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        let bit = 1u64 << square.to_index();
        if self.white_pawns & bit != 0 {
            return Some((Piece::Pawn, Color::White));
        }
        if self.white_knights & bit != 0 {
            return Some((Piece::Knight, Color::White));
        }
        if self.white_bishops & bit != 0 {
            return Some((Piece::Bishop, Color::White));
        }
        if self.white_rooks & bit != 0 {
            return Some((Piece::Rook, Color::White));
        }
        if self.white_queens & bit != 0 {
            return Some((Piece::Queen, Color::White));
        }
        if self.white_kings & bit != 0 {
            return Some((Piece::King, Color::White));
        }

        if self.black_pawns & bit != 0 {
            return Some((Piece::Pawn, Color::Black));
        }
        if self.black_knights & bit != 0 {
            return Some((Piece::Knight, Color::Black));
        }
        if self.black_bishops & bit != 0 {
            return Some((Piece::Bishop, Color::Black));
        }
        if self.black_rooks & bit != 0 {
            return Some((Piece::Rook, Color::Black));
        }
        if self.black_queens & bit != 0 {
            return Some((Piece::Queen, Color::Black));
        }
        if self.black_kings & bit != 0 {
            return Some((Piece::King, Color::Black));
        }

        None
    }

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        if !self.castling_rights.can_castle_kingside(color) {
            return false;
        }
        let empty_mask = match color {
            Color::White => Square::F1.to_bitboard() | Square::G1.to_bitboard(),
            Color::Black => Square::F8.to_bitboard() | Square::G8.to_bitboard(),
        };
        self.get_occupied_squares() & empty_mask == 0
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        if !self.castling_rights.can_castle_queenside(color) {
            return false;
        }
        let empty_mask = match color {
            Color::White => {
                Square::B1.to_bitboard() | Square::C1.to_bitboard() | Square::D1.to_bitboard()
            }
            Color::Black => {
                Square::B8.to_bitboard() | Square::C8.to_bitboard() | Square::D8.to_bitboard()
            }
        };
        self.get_occupied_squares() & empty_mask == 0
    }

    pub fn make_move(&mut self, mv: Move) -> IrreversibleState {
        let piece_from = self.get_piece_at(mv.from()).unwrap();
        let piece_to = self.get_piece_at(mv.to());

        let irreversibile_state = IrreversibleState {
            castling_rights: self.castling_rights,
            en_passant_square: self.en_passant_square,
            halfmove_clock: self.halfmove_clock,
            captured_piece: piece_to.map(|(piece, _)| piece),
        };

        let color = self.get_current_player_color();
        let oponemt_color = self.get_opponent_color();
        self.move_piece(piece_from.0, piece_from.1, mv.from(), mv.to());
        match mv.kind() {
            MoveKind::Promotion(_) => {
                self.remove_piece(Piece::Pawn, color, mv.to());
                self.place_piece(mv.promotion().unwrap(), color, mv.to());
            }
            MoveKind::Capture => {
                self.remove_piece(piece_to.unwrap().0, oponemt_color, mv.to());
            }
            MoveKind::PromotionCapture(_) => {
                self.remove_piece(piece_to.unwrap().0, oponemt_color, mv.to());
                self.remove_piece(Piece::Pawn, color, mv.to());
                self.place_piece(mv.promotion().unwrap(), color, mv.to());
            }
            MoveKind::KingsideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::H1, Square::F1),
                    Color::Black => (Square::H8, Square::F8),
                };
                self.move_piece(Piece::Rook, color, rook_from, rook_to);
            }
            MoveKind::QueensideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::A1, Square::D1),
                    Color::Black => (Square::A8, Square::D8),
                };
                self.move_piece(Piece::Rook, color, rook_from, rook_to);
            }
            MoveKind::EnPassant => {
                let captured_pawn_square = match color {
                    Color::White => Square((mv.to().to_index() - 8) as u8),
                    Color::Black => Square((mv.to().to_index() + 8) as u8),
                };
                self.remove_piece(Piece::Pawn, oponemt_color, captured_pawn_square);
            }
            default => {
                // No additional board updates needed for double pawn push
            }
        }

        self.castling_rights.0 &= CASTLING_RIGHTS_MASK[mv.from().to_index() as usize]
            & CASTLING_RIGHTS_MASK[mv.to().to_index() as usize];

        self.halfmove_clock = if piece_from.0 == Piece::Pawn || mv.kind() == MoveKind::Capture {
            0
        } else {
            self.halfmove_clock + 1
        };
        self.en_passant_square = if mv.kind() == MoveKind::DoublePawnPush {
            Some(Square(
                ((mv.from().to_index() + mv.to().to_index()) / 2) as u8,
            ))
        } else {
            None
        };
        self.fullmove_number += if self.white_to_move { 0 } else { 1 };
        self.white_to_move = !self.white_to_move;

        irreversibile_state
    }

    pub fn unmake_move(&mut self, _mv: Move) {
        self.castling_rights.0 &= CASTLING_RIGHTS_MASK[_mv.from().to_index() as usize]
            & CASTLING_RIGHTS_MASK[_mv.to().to_index() as usize];
        // TODO implement move unmaking later
    }

    fn move_piece(&mut self, piece: Piece, color: Color, from: Square, to: Square) {
        let from_bb = from.to_bitboard();
        let to_bb = to.to_bitboard();

        match (piece, color) {
            (Piece::Pawn,   Color::White) => self.white_pawns   = (self.white_pawns   & !from_bb) | to_bb,
            (Piece::Knight, Color::White) => self.white_knights = (self.white_knights & !from_bb) | to_bb,
            (Piece::Bishop, Color::White) => self.white_bishops = (self.white_bishops & !from_bb) | to_bb,
            (Piece::Rook,   Color::White) => self.white_rooks   = (self.white_rooks   & !from_bb) | to_bb,
            (Piece::Queen,  Color::White) => self.white_queens  = (self.white_queens  & !from_bb) | to_bb,
            (Piece::King,   Color::White) => self.white_kings   = (self.white_kings   & !from_bb) | to_bb,
            (Piece::Pawn,   Color::Black) => self.black_pawns   = (self.black_pawns   & !from_bb) | to_bb,
            (Piece::Knight, Color::Black) => self.black_knights = (self.black_knights & !from_bb) | to_bb,
            (Piece::Bishop, Color::Black) => self.black_bishops = (self.black_bishops & !from_bb) | to_bb,
            (Piece::Rook,   Color::Black) => self.black_rooks   = (self.black_rooks   & !from_bb) | to_bb,
            (Piece::Queen,  Color::Black) => self.black_queens  = (self.black_queens  & !from_bb) | to_bb,
            (Piece::King,   Color::Black) => self.black_kings   = (self.black_kings   & !from_bb) | to_bb,
        }
    }

    fn place_piece(&mut self, piece: Piece, color: Color, square: Square) {
        let bit = square.to_bitboard();
        match (piece, color) {
            (Piece::Pawn,   Color::White) => self.white_pawns   |= bit,
            (Piece::Knight, Color::White) => self.white_knights |= bit,
            (Piece::Bishop, Color::White) => self.white_bishops |= bit,
            (Piece::Rook,   Color::White) => self.white_rooks   |= bit,
            (Piece::Queen,  Color::White) => self.white_queens  |= bit,
            (Piece::King,   Color::White) => self.white_kings   |= bit,
            (Piece::Pawn,   Color::Black) => self.black_pawns   |= bit,
            (Piece::Knight, Color::Black) => self.black_knights |= bit,
            (Piece::Bishop, Color::Black) => self.black_bishops |= bit,
            (Piece::Rook,   Color::Black) => self.black_rooks   |= bit,
            (Piece::Queen,  Color::Black) => self.black_queens  |= bit,
            (Piece::King,   Color::Black) => self.black_kings   |= bit,
        }
    }

    fn remove_piece(&mut self, piece: Piece, color: Color, square: Square) {
        let bit = square.to_bitboard();
        match (piece, color) {
            (Piece::Pawn, Color::White) => self.white_pawns &= !bit,
            (Piece::Knight, Color::White) => self.white_knights &= !bit,
            (Piece::Bishop, Color::White) => self.white_bishops &= !bit,
            (Piece::Rook, Color::White) => self.white_rooks &= !bit,
            (Piece::Queen, Color::White) => self.white_queens &= !bit,
            (Piece::King, Color::White) => self.white_kings &= !bit,
            (Piece::Pawn, Color::Black) => self.black_pawns &= !bit,
            (Piece::Knight, Color::Black) => self.black_knights &= !bit,
            (Piece::Bishop, Color::Black) => self.black_bishops &= !bit,
            (Piece::Rook, Color::Black) => self.black_rooks &= !bit,
            (Piece::Queen, Color::Black) => self.black_queens &= !bit,
            (Piece::King, Color::Black) => self.black_kings &= !bit,
        }
    }
}
