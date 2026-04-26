use crate::engine::board::CastlingRights;
use crate::engine::piece::{Color, Piece};
use crate::engine::square::Square;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum MoveKind {
    #[default]
    Quiet,
    Capture,
    KingsideCastle,
    QueensideCastle,
    EnPassant,
    DoublePawnPush,
    Promotion(Piece),
    PromotionCapture(Piece),
}

#[derive(Debug, Clone, Copy)]
pub struct IrreversibleState {
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u8,
    pub captured_piece: Option<Piece>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    from: Square,
    to: Square,
    piece: Piece,
    kind: MoveKind,
}

impl Default for Move {
    fn default() -> Self {
        Move { from: Square::A1, to: Square::A1, piece: Piece::Pawn, kind: MoveKind::Quiet }
    }
}

impl Move {
    pub fn quiet(from: Square, to: Square, piece: Piece) -> Move {
        Move { from, to, piece, kind: MoveKind::Quiet }
    }

    pub fn capture(from: Square, to: Square, piece: Piece) -> Move {
        Move { from, to, piece, kind: MoveKind::Capture }
    }

    pub fn promote(from: Square, to: Square, promoted_to: Piece) -> Move {
        Move { from, to, piece: Piece::Pawn, kind: MoveKind::Promotion(promoted_to) }
    }

    pub fn promote_capture(from: Square, to: Square, promoted_to: Piece) -> Move {
        Move { from, to, piece: Piece::Pawn, kind: MoveKind::PromotionCapture(promoted_to) }
    }

    pub fn en_passant(from: Square, to: Square) -> Move {
        Move { from, to, piece: Piece::Pawn, kind: MoveKind::EnPassant }
    }

    pub fn double_pawn_push(from: Square, to: Square) -> Move {
        Move { from, to, piece: Piece::Pawn, kind: MoveKind::DoublePawnPush }
    }

    pub fn kingside_castle(color: Color) -> Move {
        match color {
            Color::White => Move { from: Square::E1, to: Square::G1, piece: Piece::King, kind: MoveKind::KingsideCastle },
            Color::Black => Move { from: Square::E8, to: Square::G8, piece: Piece::King, kind: MoveKind::KingsideCastle },
        }
    }

    pub fn queenside_castle(color: Color) -> Move {
        match color {
            Color::White => Move { from: Square::E1, to: Square::C1, piece: Piece::King, kind: MoveKind::QueensideCastle },
            Color::Black => Move { from: Square::E8, to: Square::C8, piece: Piece::King, kind: MoveKind::QueensideCastle },
        }
    }

    pub fn from(&self) -> Square { self.from }
    pub fn to(&self) -> Square { self.to }
    pub fn piece(&self) -> Piece { self.piece }
    pub fn kind(&self) -> MoveKind { self.kind }

    pub fn promotion(&self) -> Option<Piece> {
        match self.kind {
            MoveKind::Promotion(p) | MoveKind::PromotionCapture(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Movelist {
    moves: [Move; 256],
    count: usize,
}

impl Movelist {
    pub fn new() -> Movelist {
        Movelist {
            moves: [Move::default(); 256],
            count: 0,
        }
    }

    pub fn add_move(&mut self, mv: Move) {
        if self.count < 256 {
            self.moves[self.count] = mv;
            self.count += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.count]
    }
}
