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

const MAILBOX_EMPTY: u8 = 0xFF;

fn mailbox_encode(piece: Piece, color: Color) -> u8 {
    let p = match piece {
        Piece::Pawn   => 0u8,
        Piece::Knight => 1,
        Piece::Bishop => 2,
        Piece::Rook   => 3,
        Piece::Queen  => 4,
        Piece::King   => 5,
    };
    let c = match color { Color::White => 0u8, Color::Black => 1 };
    p * 2 + c
}

fn mailbox_decode(v: u8) -> (Piece, Color) {
    let piece = match v / 2 {
        0 => Piece::Pawn,
        1 => Piece::Knight,
        2 => Piece::Bishop,
        3 => Piece::Rook,
        4 => Piece::Queen,
        _ => Piece::King,
    };
    let color = if v % 2 == 0 { Color::White } else { Color::Black };
    (piece, color)
}

#[derive(Debug, Clone, Copy)]
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

    mailbox: [u8; 64],

    halfmove_clock: u8,
    fullmove_number: u8,
    mg_eval: i32,
    eg_eval: i32,
    game_phase: i32,
    white_to_move: bool,
    en_passant_square: Option<Square>,
    castling_rights: CastlingRights,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_kings: 0,
            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_kings: 0,
            mailbox: [MAILBOX_EMPTY; 64],
            halfmove_clock: 0,
            fullmove_number: 0,
            mg_eval: 0,
            eg_eval: 0,
            game_phase: 0,
            white_to_move: true,
            en_passant_square: None,
            castling_rights: CastlingRights(0),
        }
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
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

            mailbox: [MAILBOX_EMPTY; 64],
            halfmove_clock: 0,
            fullmove_number: 0,
            mg_eval: 0,
            eg_eval: 0,
            game_phase: 0,
            white_to_move: true,
            en_passant_square: None,
            castling_rights: CastlingRights(
                CastlingRights::WHITE_KINGSIDE
                    | CastlingRights::WHITE_QUEENSIDE
                    | CastlingRights::BLACK_KINGSIDE
                    | CastlingRights::BLACK_QUEENSIDE,
            ),
        };
        board.rebuild_mailbox();
        board.game_phase = board.compute_game_phase();
        let (mg, eg) = board.evaluate();
        board.mg_eval = mg;
        board.eg_eval = eg;
        board
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

        board.rebuild_mailbox();
        board.game_phase = board.compute_game_phase();
        let (mg, eg) = board.evaluate();
        board.mg_eval = mg;
        board.eg_eval = eg;

        Ok(board)
    }

    fn rebuild_mailbox(&mut self) {
        self.mailbox = [MAILBOX_EMPTY; 64];
        let pieces = [
            (Piece::Pawn,   Color::White, self.white_pawns),
            (Piece::Knight, Color::White, self.white_knights),
            (Piece::Bishop, Color::White, self.white_bishops),
            (Piece::Rook,   Color::White, self.white_rooks),
            (Piece::Queen,  Color::White, self.white_queens),
            (Piece::King,   Color::White, self.white_kings),
            (Piece::Pawn,   Color::Black, self.black_pawns),
            (Piece::Knight, Color::Black, self.black_knights),
            (Piece::Bishop, Color::Black, self.black_bishops),
            (Piece::Rook,   Color::Black, self.black_rooks),
            (Piece::Queen,  Color::Black, self.black_queens),
            (Piece::King,   Color::Black, self.black_kings),
        ];
        for (piece, color, mut bb) in pieces {
            while bb != 0 {
                let idx = bb.trailing_zeros() as usize;
                self.mailbox[idx] = mailbox_encode(piece, color);
                bb &= bb - 1;
            }
        }
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

    pub fn get_board_evaluation(&self) -> i32 {
        let mg_phase = self.game_phase;
        let eg_phase = 128 - mg_phase;
        let score = (self.mg_eval * mg_phase + self.eg_eval * eg_phase) >> 7;
        if self.get_current_player_color() == Color::White { score } else { -score }
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        let king_bb = self.get_bitboard_for_piece(Piece::King, color);
        Square(king_bb.trailing_zeros() as u8)
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
        let v = self.mailbox[square.to_index()];
        if v == MAILBOX_EMPTY { None } else { Some(mailbox_decode(v)) }
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
        let piece = mv.piece();
        let captured = mv.captured();
        let is_capture = matches!(mv.kind(), MoveKind::Capture | MoveKind::PromotionCapture(_));

        let irreversibile_state = IrreversibleState {
            castling_rights: self.castling_rights,
            en_passant_square: self.en_passant_square,
            halfmove_clock: self.halfmove_clock,
            captured_piece: captured,
        };

        let color = self.get_current_player_color();
        let opponent = self.get_opponent_color();
        let (dmg, deg) = Self::pst_delta(mv, captured, color, opponent);
        self.mg_eval += dmg;
        self.eg_eval += deg;

        // update game_phase: captures remove material, promotions swap pawn (weight=0) for piece
        if let Some(cap) = captured {
            self.game_phase = (self.game_phase - cap.phase_weight()).max(0);
        }
        if let MoveKind::Promotion(promo) | MoveKind::PromotionCapture(promo) = mv.kind() {
            self.game_phase = (self.game_phase + promo.phase_weight()).min(128);
        }

        match mv.kind() {
            MoveKind::Promotion(_) => {
                self.remove_piece(Piece::Pawn, color, mv.from());
                self.place_piece(mv.promotion().unwrap(), color, mv.to());
            }
            MoveKind::PromotionCapture(_) => {
                self.remove_piece(captured.unwrap(), opponent, mv.to());
                self.remove_piece(Piece::Pawn, color, mv.from());
                self.place_piece(mv.promotion().unwrap(), color, mv.to());
            }
            MoveKind::Capture => {
                self.remove_piece(captured.unwrap(), opponent, mv.to());
                self.move_piece(piece, color, mv.from(), mv.to());
            }
            MoveKind::KingsideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::H1, Square::F1),
                    Color::Black => (Square::H8, Square::F8),
                };
                self.move_piece(Piece::King, color, mv.from(), mv.to());
                self.move_piece(Piece::Rook, color, rook_from, rook_to);
            }
            MoveKind::QueensideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::A1, Square::D1),
                    Color::Black => (Square::A8, Square::D8),
                };
                self.move_piece(Piece::King, color, mv.from(), mv.to());
                self.move_piece(Piece::Rook, color, rook_from, rook_to);
            }
            MoveKind::EnPassant => {
                self.move_piece(Piece::Pawn, color, mv.from(), mv.to());
                let captured_pawn_square = match color {
                    Color::White => Square((mv.to().to_index() - 8) as u8),
                    Color::Black => Square((mv.to().to_index() + 8) as u8),
                };
                self.remove_piece(Piece::Pawn, opponent, captured_pawn_square);
            }
            _ => {
                self.move_piece(piece, color, mv.from(), mv.to());
            }
        }

        self.castling_rights.0 &= CASTLING_RIGHTS_MASK[mv.from().to_index() as usize]
            & CASTLING_RIGHTS_MASK[mv.to().to_index() as usize];

        self.halfmove_clock = if piece == Piece::Pawn || is_capture {
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

    pub fn unmake_move(&mut self, mv: Move, state: IrreversibleState) { 
        self.white_to_move = !self.white_to_move;
        self.en_passant_square = state.en_passant_square;
        self.halfmove_clock = state.halfmove_clock;
        self.castling_rights = state.castling_rights;
        self.fullmove_number -= if self.white_to_move { 0 } else { 1 };

        let color = self.get_current_player_color();
        let opponent = self.get_opponent_color();
        let piece = mv.piece();

        match mv.kind() {
            MoveKind::Promotion(_) => {
                self.remove_piece(mv.promotion().unwrap(), color, mv.to());
                self.place_piece(Piece::Pawn, color, mv.from());
            }
            MoveKind::PromotionCapture(_) => {
                self.remove_piece(mv.promotion().unwrap(), color, mv.to());
                self.place_piece(Piece::Pawn, color, mv.from());
                self.place_piece(state.captured_piece.unwrap(), opponent, mv.to());
            }
            MoveKind::Capture => {
                self.move_piece(piece, color, mv.to(), mv.from());
                self.place_piece(state.captured_piece.unwrap(), opponent, mv.to());
            }
            MoveKind::KingsideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::H1, Square::F1),
                    Color::Black => (Square::H8, Square::F8),
                };
                self.move_piece(Piece::King, color, mv.to(), mv.from());
                self.move_piece(Piece::Rook, color, rook_to, rook_from);
            }
            MoveKind::QueensideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::A1, Square::D1),
                    Color::Black => (Square::A8, Square::D8),
                };
                self.move_piece(Piece::King, color, mv.to(), mv.from());
                self.move_piece(Piece::Rook, color, rook_to, rook_from);
            }
            MoveKind::EnPassant => {
                self.move_piece(Piece::Pawn, color, mv.to(), mv.from());
                let cap_sq = match color {
                    Color::White => Square((mv.to().to_index() - 8) as u8),
                    Color::Black => Square((mv.to().to_index() + 8) as u8),
                };
                self.place_piece(Piece::Pawn, opponent, cap_sq);
            }
            _ => {
                self.move_piece(piece, color, mv.to(), mv.from());
            }
        }

        // restore game_phase (inverse of make_move)
        if let Some(cap) = mv.captured() {
            self.game_phase = (self.game_phase + cap.phase_weight()).min(128);
        }
        if let MoveKind::Promotion(promo) | MoveKind::PromotionCapture(promo) = mv.kind() {
            self.game_phase = (self.game_phase - promo.phase_weight()).max(0);
        }

        // undo eval deltas (mirror of make_move)
        let (dmg, deg) = Self::pst_delta(mv, state.captured_piece, color, opponent);
        self.mg_eval -= dmg;
        self.eg_eval -= deg;
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
        self.mailbox[from.to_index()] = MAILBOX_EMPTY;
        self.mailbox[to.to_index()] = mailbox_encode(piece, color);
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
        self.mailbox[square.to_index()] = mailbox_encode(piece, color);
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
        self.mailbox[square.to_index()] = MAILBOX_EMPTY;
    }

    pub fn make_squares_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<Piece>,
    ) -> Result<IrreversibleState, String> {
        let (piece, _) = self
            .get_piece_at(from)
            .ok_or_else(|| format!("no piece at {from}"))?;

        let color = self.get_current_player_color();

        let mv = if let Some(promo) = promotion {
            if let Some(captured) =  self.get_piece_at(to) {
                Move::promote_capture(from, to, promo, captured.0)
            } else {
                Move::promote(from, to, promo)
            }
        } else if piece == Piece::King && (to.file() as i32 - from.file() as i32).abs() == 2 {
            if to.file() > from.file() {
                Move::kingside_castle(color)
            } else {
                Move::queenside_castle(color)
            }
        } else if piece == Piece::Pawn && Some(to) == self.en_passant_square {
            Move::en_passant(from, to)
        } else if piece == Piece::Pawn && (to.to_index() as i32 - from.to_index() as i32).abs() == 16 {
            Move::double_pawn_push(from, to)
        } else if let Some(captured) =  self.get_piece_at(to) {
            Move::capture(from, to, piece, captured.0)
        } else {
            Move::quiet(from, to, piece)
        };

        Ok(self.make_move(mv))
    }

    fn compute_game_phase(&self) -> i32 {
        let mut phase = 0;
        for color in [Color::White, Color::Black] {
            for piece in Piece::ALL {
                let count = self.get_bitboard_for_piece(piece, color).count_ones() as i32;
                phase += count * piece.phase_weight();
            }
        }
        phase.min(128)
    }

    fn evaluate(&self) -> (i32, i32) {
        let mut mg = 0i32;
        let mut eg = 0i32;
        for color in [Color::White, Color::Black] {
            let sign = if color == Color::White { 1 } else { -1 };
            for piece in Piece::ALL {
                let mut bb = self.get_bitboard_for_piece(piece, color);
                while bb != 0 {
                    let sq = Square(bb.trailing_zeros() as u8);
                    mg += sign * piece.mg_table(color, sq);
                    eg += sign * piece.eg_table(color, sq);
                    bb &= bb - 1;
                }
            }
        }
        (mg, eg)
    }

    fn pst_delta(mv: Move, captured: Option<Piece>, color: Color, opponent: Color) -> (i32, i32) {
        let (mg_t, eg_t) = crate::engine::piece::mg_eg_tables();
        let mgt = |p: Piece, c: Color, sq: Square| mg_t[p.index()][c.index()][sq.to_index()];
        let egt = |p: Piece, c: Color, sq: Square| eg_t[p.index()][c.index()][sq.to_index()];
        let sign = if color == Color::White { 1 } else { -1 };
        let piece = mv.piece();
        let (mg, eg) = match mv.kind() {
            MoveKind::Promotion(promo) => (
                mgt(promo, color, mv.to()) - mgt(Piece::Pawn, color, mv.from()),
                egt(promo, color, mv.to()) - egt(Piece::Pawn, color, mv.from()),
            ),
            MoveKind::PromotionCapture(promo) => {
                let cap = captured.unwrap();
                (
                    mgt(promo, color, mv.to()) - mgt(Piece::Pawn, color, mv.from()) + mgt(cap, opponent, mv.to()),
                    egt(promo, color, mv.to()) - egt(Piece::Pawn, color, mv.from()) + egt(cap, opponent, mv.to()),
                )
            }
            MoveKind::Capture => {
                let cap = captured.unwrap();
                (
                    mgt(piece, color, mv.to()) - mgt(piece, color, mv.from()) + mgt(cap, opponent, mv.to()),
                    egt(piece, color, mv.to()) - egt(piece, color, mv.from()) + egt(cap, opponent, mv.to()),
                )
            }
            MoveKind::EnPassant => {
                let cap_sq = match color {
                    Color::White => Square((mv.to().to_index() - 8) as u8),
                    Color::Black => Square((mv.to().to_index() + 8) as u8),
                };
                (
                    mgt(piece, color, mv.to()) - mgt(piece, color, mv.from()) + mgt(Piece::Pawn, opponent, cap_sq),
                    egt(piece, color, mv.to()) - egt(piece, color, mv.from()) + egt(Piece::Pawn, opponent, cap_sq),
                )
            }
            MoveKind::KingsideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::H1, Square::F1),
                    Color::Black => (Square::H8, Square::F8),
                };
                (
                    mgt(piece, color, mv.to()) - mgt(piece, color, mv.from())
                        + mgt(Piece::Rook, color, rook_to) - mgt(Piece::Rook, color, rook_from),
                    egt(piece, color, mv.to()) - egt(piece, color, mv.from())
                        + egt(Piece::Rook, color, rook_to) - egt(Piece::Rook, color, rook_from),
                )
            }
            MoveKind::QueensideCastle => {
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::A1, Square::D1),
                    Color::Black => (Square::A8, Square::D8),
                };
                (
                    mgt(piece, color, mv.to()) - mgt(piece, color, mv.from())
                        + mgt(Piece::Rook, color, rook_to) - mgt(Piece::Rook, color, rook_from),
                    egt(piece, color, mv.to()) - egt(piece, color, mv.from())
                        + egt(Piece::Rook, color, rook_to) - egt(Piece::Rook, color, rook_from),
                )
            }
            _ => (
                mgt(piece, color, mv.to()) - mgt(piece, color, mv.from()),
                egt(piece, color, mv.to()) - egt(piece, color, mv.from()),
            ),
        };
        (sign * mg, sign * eg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::mv::Move;
    use crate::engine::square::Square;

    #[test]
    fn evaluate_starting_position_is_zero() {
        let board = Board::new();
        assert_eq!(board.get_board_evaluation(), 0);
    }

    #[test]
    fn evaluate_white_up_a_pawn() {
        let board = Board::from_fen("rnbqkbnr/ppppppp1/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert!(board.get_board_evaluation() > 0);
    }

    #[test]
    fn evaluate_black_up_a_queen() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1").unwrap();
        assert!(board.get_board_evaluation() < 0);
    }

    #[test]
    fn make_unmake_capture_restores_eval() {
        let mut board = Board::from_fen("8/8/3p4/4P3/8/8/8/4K2k w - - 0 1").unwrap();
        let mv = Move::capture(Square::E5, Square::D6, Piece::Pawn, Piece::Pawn);
        let before = board.get_board_evaluation();
        let state = board.make_move(mv);
        board.unmake_move(mv, state);
        assert_eq!(board.get_board_evaluation(), before);
    }

    #[test]
    fn make_unmake_quiet_restores_eval() {
        let mut board = Board::from_fen("8/8/8/8/8/8/4P3/4K2k w - - 0 1").unwrap();
        let mv = Move::quiet(Square::E2, Square::E4, Piece::Pawn);
        let before = board.get_board_evaluation();
        let state = board.make_move(mv);
        board.unmake_move(mv, state);
        assert_eq!(board.get_board_evaluation(), before);
    }

    #[test]
    fn make_move_eval_matches_full_eval() {
        let mut board = Board::from_fen("8/8/3p4/4P3/8/8/8/4K2k w - - 0 1").unwrap();
        let mv = Move::capture(Square::E5, Square::D6, Piece::Pawn, Piece::Pawn);
        board.make_move(mv);
        let incremental = board.get_board_evaluation();
        let (mg, eg) = board.evaluate();
        let mg_phase = board.game_phase;
        let eg_phase = 128 - mg_phase;
        let full = (mg * mg_phase + eg * eg_phase) >> 7;
        let full_from_white = if board.get_current_player_color() == Color::White { full } else { -full };
        assert_eq!(incremental, full_from_white);
    }
}
