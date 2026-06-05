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
    pub prev_hash: u64,
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

const KILLER_PRIMARY_SCORE: i32 = -1_000;
const KILLER_SECONDARY_SCORE: i32 = -1_001;
const TT_MOVE_SCORE: i32 = 1_000_000;

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
    scores: [i32; 256],
    count: usize,
}

impl Movelist {
    pub fn new() -> Movelist {
        Movelist {
            moves: [Move::default(); 256],
            scores: [0; 256],
            count: 0,
        }
    }

    pub fn add_move(&mut self, mv: Move) {
        if self.count >= 256 { return; }
        self.scores[self.count] = mv.mvv_lva_score();
        self.moves[self.count] = mv;
        self.count += 1;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.count]
    }

    pub fn rescore_tt_move(&mut self, tt_move: Move) {
        for i in 0..self.count {
            if self.moves[i] == tt_move {
                self.scores[i] = TT_MOVE_SCORE;
                return;
            }
        }
    }

    pub fn rescore_with_killers(&mut self, killers: &[Option<Move>; 2]) {
        for i in 0..self.count {
            if self.scores[i] != i32::MIN {
                continue; // capture: on garde le score MVV-LVA
            }
            let mv = self.moves[i];
            if Some(mv) == killers[0] {
                self.scores[i] = KILLER_PRIMARY_SCORE;
            } else if Some(mv) == killers[1] {
                self.scores[i] = KILLER_SECONDARY_SCORE;
            }
        }
    }

    pub fn sort(&mut self) {
        let moves = &mut self.moves[..self.count];
        let scores = &mut self.scores[..self.count];
        // insertion sort sur les deux tableaux parallèles
        for i in 1..moves.len() {
            let mv = moves[i];
            let score = scores[i];
            let mut j = i;
            while j > 0 && scores[j - 1] < score {
                moves[j] = moves[j - 1];
                scores[j] = scores[j - 1];
                j -= 1;
            }
            moves[j] = mv;
            scores[j] = score;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut list: Movelist) -> Vec<Move> {
        list.sort();
        list.as_slice().to_vec()
    }

    #[test]
    fn quiet_moves_added_last() {
        let mut list = Movelist::new();
        list.add_move(Move::quiet(Square::A1, Square::A2, Piece::Pawn));
        list.add_move(Move::capture(Square::B1, Square::B2, Piece::Pawn, Piece::Queen));
        let ordered = sorted(list);
        assert_eq!(ordered[0].capture, Some(Piece::Queen));
        assert_eq!(ordered[1].capture, None);
    }

    #[test]
    fn captures_sorted_by_mvv_lva() {
        let mut list = Movelist::new();
        list.add_move(Move::capture(Square::A1, Square::A2, Piece::Queen, Piece::Pawn));
        list.add_move(Move::capture(Square::B1, Square::B2, Piece::Pawn, Piece::Queen));
        list.add_move(Move::capture(Square::C1, Square::C2, Piece::Knight, Piece::Rook));
        let ordered = sorted(list);
        // Best: Pawn x Queen, then Knight x Rook, then Queen x Pawn
        assert_eq!(ordered[0].piece, Piece::Pawn);
        assert_eq!(ordered[1].piece, Piece::Knight);
        assert_eq!(ordered[2].piece, Piece::Queen);
    }

    #[test]
    fn multiple_quiet_moves_all_returned() {
        let mut list = Movelist::new();
        list.add_move(Move::quiet(Square::A1, Square::A2, Piece::Pawn));
        list.add_move(Move::quiet(Square::B1, Square::B2, Piece::Pawn));
        list.add_move(Move::quiet(Square::C1, Square::C2, Piece::Pawn));
        assert_eq!(list.len(), 3);
        for mv in sorted(list) {
            assert_eq!(mv.capture, None);
        }
    }

    #[test]
    fn killer_moves_ordered_before_other_quiets() {
        let mut list = Movelist::new();
        let q1 = Move::quiet(Square::A1, Square::A2, Piece::Pawn);
        let killer = Move::quiet(Square::B1, Square::B2, Piece::Pawn);
        let q3 = Move::quiet(Square::C1, Square::C2, Piece::Pawn);
        let cap = Move::capture(Square::D1, Square::D2, Piece::Rook, Piece::Bishop);
        list.add_move(q1);
        list.add_move(killer);
        list.add_move(q3);
        list.add_move(cap);
        list.rescore_with_killers(&[Some(killer), None]);
        let ordered = sorted(list);
        assert_eq!(ordered[0], cap, "capture must come first");
        assert_eq!(ordered[1], killer, "killer must come before non-killer quiets");
        assert!(ordered[2].captured().is_none() && ordered[3].captured().is_none());
    }

    #[test]
    fn killer_primary_before_secondary() {
        let mut list = Movelist::new();
        let primary = Move::quiet(Square::A1, Square::A2, Piece::Pawn);
        let secondary = Move::quiet(Square::B1, Square::B2, Piece::Pawn);
        list.add_move(secondary);
        list.add_move(primary);
        list.rescore_with_killers(&[Some(primary), Some(secondary)]);
        let ordered = sorted(list);
        assert_eq!(ordered[0], primary);
        assert_eq!(ordered[1], secondary);
    }

    #[test]
    fn killers_not_applied_to_captures() {
        let mut list = Movelist::new();
        let cap = Move::capture(Square::A1, Square::A2, Piece::Pawn, Piece::Queen);
        list.add_move(cap);
        // même si on passe la capture comme "killer", son score MVV-LVA doit rester intact
        list.rescore_with_killers(&[Some(cap), None]);
        assert_eq!(list.scores[0], cap.mvv_lva_score());
    }

    #[test]
    fn captures_before_all_quiets() {
        let mut list = Movelist::new();
        list.add_move(Move::quiet(Square::A1, Square::A2, Piece::Pawn));
        list.add_move(Move::quiet(Square::B1, Square::B2, Piece::Pawn));
        list.add_move(Move::capture(Square::C1, Square::C2, Piece::Rook, Piece::Bishop));
        list.add_move(Move::quiet(Square::D1, Square::D2, Piece::Pawn));
        let ordered = sorted(list);
        assert!(ordered[0].capture.is_some(), "first must be capture");
        for mv in &ordered[1..] {
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
