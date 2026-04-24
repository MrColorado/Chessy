use crate::engine::square::Square;
use crate::engine::piece::Piece;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<Piece>,
}

impl Move {
    pub fn new(from: Square, to: Square, promotion: Option<Piece>) -> Move {
        Move { from, to, promotion }
    }

    pub fn from(&self) -> Square { self.from }
    pub fn to(&self) -> Square { self.to }
    pub fn promotion(&self) -> Option<Piece> { self.promotion }
}

#[derive(Debug, Clone)]
pub struct Movelist {
    moves: [Move; 256],
    count: usize,
}

impl Movelist {
    pub fn new() -> Movelist {
        Movelist {
            moves: [Move::new(Square::A1, Square::A1, None); 256],
            count: 0,
        }
    }

    pub fn add_move(&mut self, mv: Move) {
        if self.count < 256 {
            self.moves[self.count] = mv;
            self.count += 1;
        }
    }

    pub fn len(&self) -> usize { self.count }

    pub fn as_slice(&self) -> &[Move] { &self.moves[..self.count] }
}
