use crate::engine::square::Square;

type BitBoard = u64;

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
        // TODO create board from position later

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
}
