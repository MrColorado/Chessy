use crate::engine::square::Square;
use std::sync::OnceLock;

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

// PeSTO tables — White perspective, index 0 = a1, 63 = h8 (rank1..rank8, fileA..fileH)
// Source: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
#[rustfmt::skip]
static MG_PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    98,134, 61, 95, 68,126, 34,-11,
    -6,  7, 26, 31, 65, 56, 25,-20,
   -14, 13,  6, 21, 23, 12, 17,-23,
   -27, -2, -5, 12, 17,  6, 10,-25,
   -26, -4, -4,-10,  3,  3, 33,-12,
   -35, -1,-20,-23,-15, 24, 38,-22,
     0,  0,  0,  0,  0,  0,  0,  0,
];
#[rustfmt::skip]
static EG_PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
   178,173,158,134,147,132,165,187,
    94,100, 85, 67, 56, 53, 82, 84,
    32, 24, 13,  5, -2,  4, 17, 17,
    13,  9, -3, -7, -7, -8,  3, -1,
     4,  7, -6,  1,  0, -5, -1, -8,
    13,  8,  8, 10, 13,  0,  2, -7,
     0,  0,  0,  0,  0,  0,  0,  0,
];
#[rustfmt::skip]
static MG_KNIGHT_TABLE: [i32; 64] = [
   -167,-89,-34,-49, 61,-97,-15,-107,
    -73,-41, 72, 36, 23, 62,  7, -17,
    -47, 60, 37, 65, 84,129, 73,  44,
     -9, 17, 19, 53, 37, 69, 18,  22,
    -13,  4, 16, 13, 28, 19, 21,  -8,
    -23, -9, 12, 10, 19, 17, 25, -16,
    -29,-53,-12, -3, -1, 18,-14, -19,
   -105,-21,-58,-33,-17,-28,-19, -23,
];
#[rustfmt::skip]
static EG_KNIGHT_TABLE: [i32; 64] = [
    -58,-38,-13,-28,-31,-27,-63,-99,
    -25, -8,-25, -2, -9,-25,-24,-52,
    -24,-20, 10,  9, -1, -9,-19,-41,
    -17,  3, 22, 22, 22, 11,  8,-18,
    -18, -6, 16, 25, 16, 17,  4,-18,
    -23, -3, -1, 15, 10, -3,-20,-22,
    -42,-20,-10, -5, -2,-20,-23,-44,
    -29,-51,-23,-15,-22,-18,-50,-64,
];
#[rustfmt::skip]
static MG_BISHOP_TABLE: [i32; 64] = [
    -29,  4,-82,-37,-25,-42,  7, -8,
    -26, 16,-18,-13, 30, 59, 18,-47,
    -16, 37, 43, 40, 35, 50, 37, -2,
     -4,  5, 19, 50, 37, 37,  7, -2,
     -6, 13, 13, 26, 34, 12, 10,  4,
      0, 15, 15, 15, 14, 27,  6,  7,
      4, 15, 16,  0,  7, 21, 33,  1,
    -33, -3,-14,-21,-13,-12,-39,-21,
];
#[rustfmt::skip]
static EG_BISHOP_TABLE: [i32; 64] = [
    -14,-21,-11, -8, -7, -9,-17,-24,
     -8, -4,  7,-12, -3,-13, -4,-14,
      2, -8,  0, -1, -2,  6,  0,  4,
     -3,  9, 12,  9, 14, 10,  3,  2,
     -6,  3, 13, 19,  7, 10, -3, -9,
    -12, -3,  8, 10, 13,  3, -7,-15,
    -14,-18, -7, -1,  4, -9,-15,-27,
    -23, -9,-23, -5, -9,-16, -5,-17,
];
#[rustfmt::skip]
static MG_ROOK_TABLE: [i32; 64] = [
     32, 42, 32, 51, 63,  9, 31, 43,
     27, 32, 58, 62, 80, 67, 26, 44,
     -5, 19, 26, 36, 17, 45, 61, 16,
    -24,-11,  7, 26, 24, 35, -8,-20,
    -36,-26,-12, -1,  9, -7,  6,-23,
    -45,-25,-16,-17,  3,  0, -5,-33,
    -44,-16,-20, -9, -1, 11, -6,-71,
    -19,-13,  1, 17, 16,  7,-37,-26,
];
#[rustfmt::skip]
static EG_ROOK_TABLE: [i32; 64] = [
     13, 10, 18, 15, 12, 12,  8,  5,
     11, 13, 13, 11, -3,  3,  8,  3,
      7,  7,  7,  5,  4, -3, -5, -3,
      4,  3, 13,  1,  2,  1, -1,  2,
      3,  5,  8,  4, -5, -6, -8, -11,
     -4,  0, -5, -1, -7,-12, -8,-16,
     -6, -6,  0,  2, -9, -9,-11, -3,
     -9,  2,  3, -1, -5,-13,  4,-20,
];
#[rustfmt::skip]
static MG_QUEEN_TABLE: [i32; 64] = [
    -28,  0, 29, 12, 59, 44, 43, 45,
    -24,-39, -5,  1,-16, 57, 28, 54,
    -13,-17,  7,  8, 29, 56, 47, 57,
    -27,-27,-16,-16, -1, 17, -2,  1,
     -9,-26, -9,-10, -2, -4,  3, -3,
    -14,  2,-11, -2, -5,  2, 14,  5,
    -35, -8, 11,  2,  8, 15, -3,  1,
     -1,-18, -9, 10,-15,-25,-31,-50,
];
#[rustfmt::skip]
static EG_QUEEN_TABLE: [i32; 64] = [
     -9, 22, 22, 27, 27, 19, 10, 20,
    -17, 20, 32, 41, 58, 25, 30,  0,
    -20,  6,  9, 49, 47, 35, 19,  9,
      3, 22, 24, 45, 57, 40, 57, 36,
    -18, 28, 19, 47, 31, 34, 39, 23,
    -16,-27, 15,  6,  9, 17, 10,  5,
    -22,-23,-30,-16,-16,-23,-36,-32,
    -33,-28,-22,-43, -5,-32,-20,-41,
];
#[rustfmt::skip]
static MG_KING_TABLE: [i32; 64] = [
    -65, 23, 16,-15,-56,-34,  2, 13,
     29, -1,-20, -7, -8, -4,-38,-29,
     -9, 24,  2,-16,-20,  6, 22,-22,
    -17,-20,-12,-27,-30,-25,-14,-36,
    -49, -1,-27,-39,-46,-44,-33,-51,
    -14,-14,-22,-46,-44,-30,-15,-27,
      1,  7, -8,-64,-43,-16,  9,  8,
    -15, 36, 12,-54,  8,-28, 24, 14,
];
#[rustfmt::skip]
static EG_KING_TABLE: [i32; 64] = [
    -74,-35,-18,-18,-11, 15,  4,-17,
    -12, 17, 14, 17, 17, 38, 23, 11,
     10, 17, 23, 15, 20, 45, 44, 13,
     -8, 22, 24, 27, 26, 33, 26,  3,
    -18, -4, 21, 24, 27, 23,  9,-11,
    -19, -3, 11, 21, 23, 16,  7, -9,
    -27,-11,  4, 13, 14,  4, -5,-17,
    -53,-34,-21,-11,-28,-14,-24,-43,
];

pub type PstTable = [[[i32; 64]; 2]; 6];

pub fn mg_eg_tables() -> &'static (PstTable, PstTable) {
    static TABLES: OnceLock<(PstTable, PstTable)> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mg_raw: [&[i32; 64]; 6] = [
            &MG_PAWN_TABLE,
            &MG_KNIGHT_TABLE,
            &MG_BISHOP_TABLE,
            &MG_ROOK_TABLE,
            &MG_QUEEN_TABLE,
            &MG_KING_TABLE,
        ];
        let eg_raw: [&[i32; 64]; 6] = [
            &EG_PAWN_TABLE,
            &EG_KNIGHT_TABLE,
            &EG_BISHOP_TABLE,
            &EG_ROOK_TABLE,
            &EG_QUEEN_TABLE,
            &EG_KING_TABLE,
        ];
        let mg_values = [82, 337, 365, 477, 1025, 20_000i32];
        let eg_values = [94, 281, 297, 512, 936, 20_000i32];

        let mut mg = [[[0i32; 64]; 2]; 6];
        let mut eg = [[[0i32; 64]; 2]; 6];
        // PeSTO tables are written visually (rank 8 first, rank 1 last).
        // Our square indexing is rank 1 first (a1=0, h8=63).
        // White reads the table directly (sq); black mirrors by flipping the rank.
        for p in 0..6 {
            for sq in 0..64usize {
                let rank = sq / 8;
                let file = sq % 8;
                let rank_flipped = (7 - rank) * 8 + file;
                mg[p][0][sq] = mg_values[p] + mg_raw[p][rank_flipped]; // White: flip to match PeSTO's a1=56 convention
                eg[p][0][sq] = eg_values[p] + eg_raw[p][rank_flipped];
                mg[p][1][sq] = mg_values[p] + mg_raw[p][sq];           // Black: sq already mirrors correctly
                eg[p][1][sq] = eg_values[p] + eg_raw[p][sq];
            }
        }
        (mg, eg)
    })
}

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
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
    }

    pub fn value(self) -> i32 {
        self.mg_value()
    }

    pub fn mg_value(self) -> i32 {
        match self {
            Piece::Pawn => 82,
            Piece::Knight => 337,
            Piece::Bishop => 365,
            Piece::Rook => 477,
            Piece::Queen => 1025,
            Piece::King => 20_000,
        }
    }

    pub fn eg_value(self) -> i32 {
        match self {
            Piece::Pawn => 94,
            Piece::Knight => 281,
            Piece::Bishop => 297,
            Piece::Rook => 512,
            Piece::Queen => 936,
            Piece::King => 20_000,
        }
    }

    // Phase weight for tapered eval: max total=128 (power of 2, enables >> 7 instead of /24).
    pub fn phase_weight(self) -> i32 {
        match self {
            Piece::Knight => 5,
            Piece::Bishop => 5,
            Piece::Rook => 11,
            Piece::Queen => 22,
            _ => 0,
        }
    }

    pub fn mg_table(self, color: Color, square: Square) -> i32 {
        mg_eg_tables().0[self.index()][color.index()][square.to_index()]
    }

    pub fn eg_table(self, color: Color, square: Square) -> i32 {
        mg_eg_tables().1[self.index()][color.index()][square.to_index()]
    }

    // One-character symbol, uppercase for display purposes.
    pub fn symbol(self) -> char {
        match self {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
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

    // PeSTO MG_KING_TABLE (rank 8 first): rank 1 values are at indices 56..63.
    // g1=index 62 = 24, c1=index 58 = 12 — castled king positions are rewarded.
    // e1=index 60 = 8, d1=index 59 = -54 — center is penalized.
    #[test]
    fn white_king_mg_castled_better_than_center() {
        let g1 = Square::G1;
        let c1 = Square::C1;
        let e1 = Square::E1;
        let white_g1 = Piece::King.mg_table(Color::White, g1);
        let white_c1 = Piece::King.mg_table(Color::White, c1);
        let white_e1 = Piece::King.mg_table(Color::White, e1);
        assert!(white_g1 > white_e1, "g1 ({white_g1}) should be better than e1 ({white_e1}) for white king MG");
        assert!(white_c1 > white_e1, "c1 ({white_c1}) should be better than e1 ({white_e1}) for white king MG");
    }

    // Black king: g8/c8 should be rewarded, e8 penalized — symmetric to white.
    #[test]
    fn black_king_mg_castled_better_than_center() {
        let g8 = Square::G8;
        let c8 = Square::C8;
        let e8 = Square::E8;
        let black_g8 = Piece::King.mg_table(Color::Black, g8);
        let black_c8 = Piece::King.mg_table(Color::Black, c8);
        let black_e8 = Piece::King.mg_table(Color::Black, e8);
        assert!(black_g8 > black_e8, "g8 ({black_g8}) should be better than e8 ({black_e8}) for black king MG");
        assert!(black_c8 > black_e8, "c8 ({black_c8}) should be better than e8 ({black_e8}) for black king MG");
    }

    // Symmetry: white on sq and black on mirrored sq should get the same PST bonus.
    #[test]
    fn pst_white_black_symmetry() {
        for sq in 0..64usize {
            let rank = sq / 8;
            let file = sq % 8;
            let mirrored = (7 - rank) * 8 + file;
            for piece in Piece::ALL {
                let white_mg = Piece::mg_table(piece, Color::White, Square(sq as u8));
                let black_mg = Piece::mg_table(piece, Color::Black, Square(mirrored as u8));
                assert_eq!(white_mg, black_mg, "{:?} MG white on {sq} vs black on {mirrored}", piece);
                let white_eg = Piece::eg_table(piece, Color::White, Square(sq as u8));
                let black_eg = Piece::eg_table(piece, Color::Black, Square(mirrored as u8));
                assert_eq!(white_eg, black_eg, "{:?} EG white on {sq} vs black on {mirrored}", piece);
            }
        }
    }

    // White pawn on rank 7 (promotion imminent) should score higher than rank 2 (starting row).
    #[test]
    fn white_pawn_advances_toward_promotion() {
        let e2 = Square::E2;
        let e7 = Square::E7;
        let mg_e2 = Piece::Pawn.mg_table(Color::White, e2);
        let mg_e7 = Piece::Pawn.mg_table(Color::White, e7);
        assert!(mg_e7 > mg_e2, "white pawn MG e7 ({mg_e7}) should be > e2 ({mg_e2})");
    }

    // Black pawn on rank 2 (promotion imminent) should score higher than rank 7 (starting row).
    #[test]
    fn black_pawn_advances_toward_promotion() {
        let e7 = Square::E7;
        let e2 = Square::E2;
        let mg_e7 = Piece::Pawn.mg_table(Color::Black, e7);
        let mg_e2 = Piece::Pawn.mg_table(Color::Black, e2);
        assert!(mg_e2 > mg_e7, "black pawn MG e2 ({mg_e2}) should be > e7 ({mg_e7})");
    }

    // Exact value check against the published PeSTO tables.
    // Reference: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
    // Material values: P=82, N=337, B=365, R=477, Q=1025, K=20000 (MG).
    #[test]
    fn pesto_exact_known_values() {
        // White pawn on e4: PeSTO rank 4, file e = +17. Total = 82 + 17 = 99.
        assert_eq!(Piece::Pawn.mg_table(Color::White, Square::E4), 82 + 17);
        // White pawn on e2 (starting square): PeSTO rank 2, file e = -15. Total = 82 - 15 = 67.
        assert_eq!(Piece::Pawn.mg_table(Color::White, Square::E2), 82 - 15);
        // White knight on f3 (developed): PeSTO rank 3, file f = 17. Total = 337 + 17 = 354.
        assert_eq!(Piece::Knight.mg_table(Color::White, Square::F3), 337 + 17);
        // White knight on b1 (starting): PeSTO rank 1, file b = -21. Total = 337 - 21 = 316.
        assert_eq!(Piece::Knight.mg_table(Color::White, Square::B1), 337 - 21);
        // White king on g1 (castled): PeSTO rank 1, file g = 24. Total = 20000 + 24.
        assert_eq!(Piece::King.mg_table(Color::White, Square::G1), 20_000 + 24);
        // White king on e1 (starting): PeSTO rank 1, file e = 8. Total = 20000 + 8.
        assert_eq!(Piece::King.mg_table(Color::White, Square::E1), 20_000 + 8);
        // White king on d1: PeSTO rank 1, file d = -54. Total = 20000 - 54.
        assert_eq!(Piece::King.mg_table(Color::White, Square::D1), 20_000 - 54);
        // EG king on e4 (active in endgame): PeSTO rank 4, file e = +27.
        assert_eq!(Piece::King.eg_table(Color::White, Square::E4), 20_000 + 27);
        // EG king on a1 (corner, bad in endgame): PeSTO rank 1, file a = -53.
        assert_eq!(Piece::King.eg_table(Color::White, Square::A1), 20_000 - 53);
    }
}
