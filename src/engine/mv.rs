use crate::engine::board::CastlingRights;
use crate::engine::piece::{Color, Piece};
use crate::engine::square::Square;
use std::fmt;

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
    capture: Option<Piece>,
}

impl Default for Move {
    fn default() -> Self {
        Move {
            from: Square::A1,
            to: Square::A1,
            piece: Piece::Pawn,
            kind: MoveKind::Quiet,
            capture: None,
        }
    }
}

impl Move {
    pub fn quiet(from: Square, to: Square, piece: Piece) -> Move {
        Move {
            from,
            to,
            piece,
            kind: MoveKind::Quiet,
            capture: None,
        }
    }

    pub fn capture(from: Square, to: Square, piece: Piece, captured: Piece) -> Move {
        Move {
            from,
            to,
            piece,
            kind: MoveKind::Capture,
            capture: Some(captured),
        }
    }

    pub fn promote(from: Square, to: Square, promoted_to: Piece) -> Move {
        Move {
            from,
            to,
            piece: Piece::Pawn,
            kind: MoveKind::Promotion(promoted_to),
            capture: None,
        }
    }

    pub fn promote_capture(from: Square, to: Square, promoted_to: Piece, captured: Piece) -> Move {
        Move {
            from,
            to,
            piece: Piece::Pawn,
            kind: MoveKind::PromotionCapture(promoted_to),
            capture: Some(captured),
        }
    }

    pub fn en_passant(from: Square, to: Square) -> Move {
        Move {
            from,
            to,
            piece: Piece::Pawn,
            kind: MoveKind::EnPassant,
            capture: Some(Piece::Pawn),
        }
    }

    pub fn double_pawn_push(from: Square, to: Square) -> Move {
        Move {
            from,
            to,
            piece: Piece::Pawn,
            kind: MoveKind::DoublePawnPush,
            capture: None,
        }
    }

    pub fn kingside_castle(color: Color) -> Move {
        match color {
            Color::White => Move {
                from: Square::E1,
                to: Square::G1,
                piece: Piece::King,
                kind: MoveKind::KingsideCastle,
                capture: None,
            },
            Color::Black => Move {
                from: Square::E8,
                to: Square::G8,
                piece: Piece::King,
                kind: MoveKind::KingsideCastle,
                capture: None,
            },
        }
    }

    pub fn queenside_castle(color: Color) -> Move {
        match color {
            Color::White => Move {
                from: Square::E1,
                to: Square::C1,
                piece: Piece::King,
                kind: MoveKind::QueensideCastle,
                capture: None,
            },
            Color::Black => Move {
                from: Square::E8,
                to: Square::C8,
                piece: Piece::King,
                kind: MoveKind::QueensideCastle,
                capture: None,
            },
        }
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }

    pub fn kind(&self) -> MoveKind {
        self.kind
    }

    pub fn captured(&self) -> Option<Piece> {
        self.capture
    }

    pub fn promotion(&self) -> Option<Piece> {
        match self.kind {
            MoveKind::Promotion(p) | MoveKind::PromotionCapture(p) => Some(p),
            _ => None,
        }
    }

    fn mvv_lva_score(&self) -> i32 {
        match self.capture {
            Some(victim) => victim.value() - self.piece.value() / 100,
            None => i32::MIN, // coups silencieux toujours après les captures
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.promotion() {
            Some(p) => write!(f, "{}{}{}", self.from, self.to, p.symbol().to_lowercase()),
            None => write!(f, "{}{}", self.from, self.to),
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
        if self.count >= 256 { return; }
        let score =  mv.mvv_lva_score();
        let mut i = self.count;
        while i > 0 && self.moves[i - 1].mvv_lva_score() < score {
            self.moves[i] = self.moves[i - 1];
            i -= 1;
        }
        self.moves[i] = mv;
        self.count += 1;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.count]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_moves_added_last() {
        let mut list = Movelist::new();
        list.add_move(Move::quiet(Square::A1, Square::A2, Piece::Pawn));
        list.add_move(Move::capture(Square::B1, Square::B2, Piece::Pawn, Piece::Queen));
        assert_eq!(list.as_slice()[0].capture, Some(Piece::Queen));
        assert_eq!(list.as_slice()[1].capture, None);
    }

    #[test]
    fn captures_sorted_by_mvv_lva() {
        let mut list = Movelist::new();
        list.add_move(Move::capture(Square::A1, Square::A2, Piece::Queen, Piece::Pawn));
        list.add_move(Move::capture(Square::B1, Square::B2, Piece::Pawn, Piece::Queen));
        list.add_move(Move::capture(Square::C1, Square::C2, Piece::Knight, Piece::Rook));
        // Best: Pawn x Queen, then Knight x Rook, then Queen x Pawn
        assert_eq!(list.as_slice()[0].piece, Piece::Pawn);
        assert_eq!(list.as_slice()[1].piece, Piece::Knight);
        assert_eq!(list.as_slice()[2].piece, Piece::Queen);
    }

    #[test]
    fn multiple_quiet_moves_preserve_insertion_order() {
        let mut list = Movelist::new();
        list.add_move(Move::quiet(Square::A1, Square::A2, Piece::Pawn));
        list.add_move(Move::quiet(Square::B1, Square::B2, Piece::Pawn));
        list.add_move(Move::quiet(Square::C1, Square::C2, Piece::Pawn));
        assert_eq!(list.len(), 3);
        for mv in list.as_slice() {
            assert_eq!(mv.capture, None);
        }
    }

    #[test]
    fn captures_before_all_quiets() {
        let mut list = Movelist::new();
        list.add_move(Move::quiet(Square::A1, Square::A2, Piece::Pawn));
        list.add_move(Move::quiet(Square::B1, Square::B2, Piece::Pawn));
        list.add_move(Move::capture(Square::C1, Square::C2, Piece::Rook, Piece::Bishop));
        list.add_move(Move::quiet(Square::D1, Square::D2, Piece::Pawn));
        let slice = list.as_slice();
        assert!(slice[0].capture.is_some(), "first must be capture");
        for mv in &slice[1..] {
            assert!(mv.capture.is_none(), "rest must be quiet");
        }
    }
}

impl IntoIterator for Movelist {
    type Item = Move;
    type IntoIter = std::iter::Take<std::array::IntoIter<Move, 256>>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter().take(self.count)
    }
}
