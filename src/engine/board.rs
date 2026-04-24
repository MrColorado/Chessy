use crate::engine::square::Square;
use crate::engine::piece::{Piece, Color};
use crate::engine::mv::Move;

pub type BitBoard = u64;

#[derive(Debug, Default, Clone, Copy)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const WHITE_KINGSIDE:  u8 = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE:  u8 = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;
}

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
            white_pawns:   0x000000000000FF00,
            white_knights: 0x0000000000000042,
            white_bishops: 0x0000000000000024,
            white_rooks:   0x0000000000000081,
            white_queens:  0x0000000000000008,
            white_kings:   0x0000000000000010,

            black_pawns:   0x00FF000000000000,
            black_knights: 0x4200000000000000,
            black_bishops: 0x2400000000000000,
            black_rooks:   0x8100000000000000,
            black_queens:  0x0800000000000000,     
            black_kings:   0x1000000000000000,

            halfmove_clock: 0,
            fullmove_number: 0,
            white_to_move: true,
            en_passant_square: None,
            castling_rights: CastlingRights(
                CastlingRights::WHITE_KINGSIDE |
                CastlingRights::WHITE_QUEENSIDE |
                CastlingRights::BLACK_KINGSIDE |
                CastlingRights::BLACK_QUEENSIDE
            ),
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let words = fen.split_whitespace().collect::<Vec<_>>();
        if words.len() != 6 {
            return Err(format!("Invalid FEN: expected 6 fields, got {}", words.len()));
        }

        let ranks = words[0].split('/').collect::<Vec<_>>();
        if ranks.len() != 8 {
            return Err(format!("Invalid FEN: expected 8 ranks, got {}", ranks.len()));
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
                        'P' => board.white_pawns   |= bit,
                        'N' => board.white_knights |= bit,
                        'B' => board.white_bishops |= bit,
                        'R' => board.white_rooks   |= bit,
                        'Q' => board.white_queens  |= bit,
                        'K' => board.white_kings   |= bit,
                        'p' => board.black_pawns   |= bit,
                        'n' => board.black_knights |= bit,
                        'b' => board.black_bishops |= bit,
                        'r' => board.black_rooks   |= bit,
                        'q' => board.black_queens  |= bit,
                        'k' => board.black_kings   |= bit,
                        _   => return Err(format!("Invalid FEN: unknown piece '{}'", ch)),
                    }
                    file += 1;
                }
            }
        }

        board.white_to_move = words[1] == "w";
        board.castling_rights = CastlingRights(
            words[2].chars().fold(0u8, |rights, ch| match ch {
                'K' => rights | CastlingRights::WHITE_KINGSIDE,
                'Q' => rights | CastlingRights::WHITE_QUEENSIDE,
                'k' => rights | CastlingRights::BLACK_KINGSIDE,
                'q' => rights | CastlingRights::BLACK_QUEENSIDE,
                _   => rights,
            })
        );
        board.en_passant_square = if words[3] != "-" {
           Some(Square::from_str(words[3]).ok_or( "Invalid FEN: invalid en passant square")?)
        } else {
            None
        };
        board.halfmove_clock = words[4].parse::<u8>().map_err(|_| "Invalid FEN: invalid halfmove clock")?;
        board.fullmove_number = words[5].parse::<u8>().map_err(|_| "Invalid FEN: invalid fullmove number")?;

        Ok(board)
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
        self.white_pawns | self.white_knights | self.white_bishops | self.white_rooks | self.white_queens | self.white_kings |
        self.black_pawns | self.black_knights | self.black_bishops | self.black_rooks | self.black_queens | self.black_kings
    }

    pub fn get_empty_squares(&self) -> BitBoard {
        !self.get_occupied_squares()
    }

    pub fn get_occupied_squares_for_color(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.white_pawns | self.white_knights | self.white_bishops | self.white_rooks | self.white_queens | self.white_kings,
            Color::Black => self.black_pawns | self.black_knights | self.black_bishops | self.black_rooks | self.black_queens | self.black_kings,
        }
    }

    pub fn get_piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        let bit = 1u64 << square.to_index();
        if self.white_pawns & bit != 0 { return Some((Piece::Pawn, Color::White)); }
        if self.white_knights & bit != 0 { return Some((Piece::Knight, Color::White)); }
        if self.white_bishops & bit != 0 { return Some((Piece::Bishop, Color::White)); }
        if self.white_rooks & bit != 0 { return Some((Piece::Rook, Color::White)); }
        if self.white_queens & bit != 0 { return Some((Piece::Queen, Color::White)); }
        if self.white_kings & bit != 0 { return Some((Piece::King, Color::White)); }

        if self.black_pawns & bit != 0 { return Some((Piece::Pawn, Color::Black)); }
        if self.black_knights & bit != 0 { return Some((Piece::Knight, Color::Black)); }
        if self.black_bishops & bit != 0 { return Some((Piece::Bishop, Color::Black)); }
        if self.black_rooks & bit != 0 { return Some((Piece::Rook, Color::Black)); }
        if self.black_queens & bit != 0 { return Some((Piece::Queen, Color::Black)); }
        if self.black_kings & bit != 0 { return Some((Piece::King, Color::Black)); }

        None
    }

    pub fn make_move(&mut self, _mv: Move) {
        // TODO implement move making later
    }

    pub fn unmake_move(&mut self, _mv: Move) {
        // TODO implement move unmaking later
    }
}
