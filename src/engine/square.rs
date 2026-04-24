use std::fmt;

// A Square is one of the 64 squares on a chess board.
//
// We number them 0–63, starting from a1 (bottom-left) to h8 (top-right):
//
//   rank 8 │ 56 57 58 59 60 61 62 63
//   rank 7 │ 48 49 50 51 52 53 54 55
//   rank 6 │ 40 41 42 43 44 45 46 47
//   rank 5 │ 32 33 34 35 36 37 38 39
//   rank 4 │ 24 25 26 27 28 29 30 31
//   rank 3 │ 16 17 18 19 20 21 22 23
//   rank 2 │  8  9 10 11 12 13 14 15
//   rank 1 │  0  1  2  3  4  5  6  7
//            ─────────────────────────
//             a  b  c  d  e  f  g  h   (file)
//
// index = rank * 8 + file   (both 0-based)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(pub u8);

impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);

    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);

    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);

    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);

    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);

    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    #[inline]
    pub fn from_file_rank(file: u8, rank: u8) -> Square {
        debug_assert!(file < 8 && rank < 8, "file and rank must be 0–7");
        return Square(rank * 8 + file)
    }

    pub fn from_str(s: &str) -> Option<Square> {
        if s.len() != 2 {
            return None;
        }
        let bytes = s.as_bytes();
        let file = bytes[0];
        let rank = bytes[1];
        if file < b'a' || file > b'h' || rank < b'1' || rank > b'8' {
            return None;
        }
        return Some(Square::from_file_rank(file - b'a', rank - b'1'))
    }

    // Return the file (column) of this square: 0 = a, 7 = h.
    #[inline]
    pub fn file(self) -> u8 {
        self.0 % 8
    }

    // Return the rank (row) of this square: 0 = rank 1, 7 = rank 8.
    #[inline]
    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    // Convert this square to a BitBoard with exactly one bit set.
    #[inline]
    pub fn to_bitboard(self) -> u64 {
        1u64 << self.0
    }

    // The raw index, useful when indexing into arrays.
    #[inline]
    pub fn to_index(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        write!(f, "{}{}", file_char, rank_char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a1_is_index_zero() {
        assert_eq!(Square::A1.to_index(), 0);
    }

    #[test]
    fn h8_is_index_63() {
        assert_eq!(Square::H8.to_index(), 63);
    }

    #[test]
    fn e4_file_and_rank() {
        assert_eq!(Square::E4.file(), 4); // e = file 4
        assert_eq!(Square::E4.rank(), 3); // rank 4 = index 3
    }

    #[test]
    fn e4_display() {
        assert_eq!(Square::E4.to_string(), "e4");
    }

    #[test]
    fn from_file_rank_roundtrip() {
        let sq = Square::from_file_rank(4, 3); // e4
        assert_eq!(sq, Square::E4);
    }

    #[test]
    fn to_bitboard_sets_correct_bit() {
        assert_eq!(Square::A1.to_bitboard(), 1);
        assert_eq!(Square::H8.to_bitboard(), 1u64 << 63);
    }
}
