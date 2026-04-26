use crate::engine::board::BitBoard;
use crate::engine::square::Square;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn sparse(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}

pub(super) struct MagicTable {
    masks:   [BitBoard; 64],
    magics:  [u64; 64],
    shifts:  [u32; 64],
    attacks: [Vec<BitBoard>; 64],
}

impl MagicTable {
    pub fn new(
        blocker_mask: fn(Square) -> BitBoard,
        attacks_slow: fn(Square, BitBoard) -> BitBoard,
    ) -> Self {
        let mut masks   = [0u64; 64];
        let mut magics  = [0u64; 64];
        let mut shifts  = [0u32; 64];
        let mut attacks: [Vec<BitBoard>; 64] = std::array::from_fn(|_| Vec::new());

        let mut rng = Rng(1_070_372);

        for sq in 0..64u8 {
            let square = Square(sq);
            let mask   = blocker_mask(square);
            let n      = mask.count_ones();
            let shift  = 64 - n;
            let size   = 1usize << n;

            masks[sq as usize]  = mask;
            shifts[sq as usize] = shift;

            let mut occ_list = vec![0u64; size];
            let mut att_list = vec![0u64; size];
            let mut subset   = 0u64;
            for i in 0..size {
                occ_list[i] = subset;
                att_list[i] = attacks_slow(square, subset);
                subset = subset.wrapping_sub(mask) & mask;
            }

            'search: loop {
                let magic = rng.sparse();

                if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 {
                    continue;
                }

                let mut table = vec![0u64; size];
                for i in 0..size {
                    let index = (occ_list[i].wrapping_mul(magic) >> shift) as usize;
                    if table[index] == 0 {
                        table[index] = att_list[i];
                    } else if table[index] != att_list[i] {
                        continue 'search;
                    }
                }

                magics[sq as usize]  = magic;
                attacks[sq as usize] = table;
                break;
            }
        }

        MagicTable { masks, magics, shifts, attacks }
    }

    pub fn attacks(&self, sq: Square, occupancy: BitBoard) -> BitBoard {
        let i        = sq.to_index();
        let relevant = occupancy & self.masks[i];
        let index    = relevant.wrapping_mul(self.magics[i]) >> self.shifts[i];
        self.attacks[i][index as usize]
    }
}