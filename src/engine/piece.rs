#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    // Return the opposite color.
    #[inline]
    pub fn flip(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    // Returns true if this color is White.
    #[inline]
    pub fn is_white(self) -> bool {
        self == Color::White
    }

    // Index into a 2-element array: White=0, Black=1.
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

// Michniewski PST — White perspective, index 0 = a1, 63 = h8 (rank1..rank8, fileA..fileH)
#[rustfmt::skip]
static PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10,-20,-20, 10, 10,  5,
     5, -5,-10,  0,  0,-10, -5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
static KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
static BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
static ROOK_TABLE: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     5, 10, 10, 10, 10, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
static QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
      0,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
static KING_TABLE: [i32; 64] = [
     20, 30, 10,  0,  0, 10, 30, 20,
     20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const ALL: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    // Index 0–5, used for indexing into per-piece arrays.
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Piece::Pawn   => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook   => 3,
            Piece::Queen  => 4,
            Piece::King   => 5,
        }
    }

    // Standard material values in centipawns (pawn = 100). // TODO let's tune these values later.
    pub fn value(self) -> i32 {
        match self {
            Piece::Pawn   => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook   => 500,
            Piece::Queen  => 900,
            Piece::King   => 20_000, 
        }
    }

    pub fn pst_bonus(self, color: Color, square: crate::engine::square::Square) -> i32 {
        let idx = match color {
            Color::White => square.to_index(),
            Color::Black => {
                let rank = square.to_index() / 8;
                let file = square.to_index() % 8;
                (7 - rank) * 8 + file
            }
        };
        match self {
            Piece::Pawn   => PAWN_TABLE[idx],
            Piece::Knight => KNIGHT_TABLE[idx],
            Piece::Bishop => BISHOP_TABLE[idx],
            Piece::Rook   => ROOK_TABLE[idx],
            Piece::Queen  => QUEEN_TABLE[idx],
            Piece::King   => KING_TABLE[idx],
        }
    }

    // One-character symbol, uppercase for display purposes.
    pub fn symbol(self) -> char {
        match self {
            Piece::Pawn   => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook   => 'R',
            Piece::Queen  => 'Q',
            Piece::King   => 'K',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColoredPiece {
    pub piece: Piece,
    pub color: Color,
}

impl ColoredPiece {
    #[inline]
    pub fn new(piece: Piece, color: Color) -> ColoredPiece {
        ColoredPiece { piece, color }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::square::Square;

    #[test]
    fn flip_color() {
        assert_eq!(Color::White.flip(), Color::Black);
        assert_eq!(Color::Black.flip(), Color::White);
    }

    #[test]
    fn piece_indices_are_unique() {
        let mut seen = [false; 6];
        for p in Piece::ALL {
            let i = p.index();
            assert!(!seen[i], "duplicate index {i} for {:?}", p);
            seen[i] = true;
        }
    }

    #[test]
    fn king_is_most_valuable() {
        for p in Piece::ALL {
            if p != Piece::King {
                assert!(Piece::King.value() > p.value());
            }
        }
    }
}
