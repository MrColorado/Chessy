// xorshift64 — same generator as movegen::magic, expressed as a const fn.
const fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

// A small stream of keys derived from a fixed seed. We thread the running
// state through every draw so all keys are distinct.
struct KeyStream(u64);

impl KeyStream {
    const fn next(&mut self) -> u64 {
        self.0 = xorshift(self.0);
        self.0
    }
}

pub struct Zobrist {
    // Indexed [piece.index()][color.index()][square].
    pub piece_keys: [[[u64; 64]; 2]; 6],
    // Indexed by the four CastlingRights bits (kingside/queenside × color).
    pub castling_keys: [u64; 4],
    // Indexed by the file (0..7) of the en-passant target square.
    pub en_passant_keys: [u64; 8],
    // XOR-ed in when it is Black to move.
    pub side_to_move_key: u64,
}

const fn generate() -> Zobrist {
    let mut stream = KeyStream(0x9E37_79B9_7F4A_7C15);

    let mut piece_keys = [[[0u64; 64]; 2]; 6];
    let mut p = 0;
    while p < 6 {
        let mut c = 0;
        while c < 2 {
            let mut sq = 0;
            while sq < 64 {
                piece_keys[p][c][sq] = stream.next();
                sq += 1;
            }
            c += 1;
        }
        p += 1;
    }

    let mut castling_keys = [0u64; 4];
    let mut i = 0;
    while i < 4 {
        castling_keys[i] = stream.next();
        i += 1;
    }

    let mut en_passant_keys = [0u64; 8];
    let mut f = 0;
    while f < 8 {
        en_passant_keys[f] = stream.next();
        f += 1;
    }

    let side_to_move_key = stream.next();

    Zobrist {
        piece_keys,
        castling_keys,
        en_passant_keys,
        side_to_move_key,
    }
}

pub static ZOBRIST: Zobrist = generate();

#[cfg(test)]
mod tests {
    use super::*;

    // Collect every key into one vector and check they are all distinct — a
    // cheap guard against a broken PRNG producing repeats.
    #[test]
    fn keys_are_distinct() {
        let mut keys = Vec::new();
        for p in 0..6 {
            for c in 0..2 {
                for sq in 0..64 {
                    keys.push(ZOBRIST.piece_keys[p][c][sq]);
                }
            }
        }
        keys.extend_from_slice(&ZOBRIST.castling_keys);
        keys.extend_from_slice(&ZOBRIST.en_passant_keys);
        keys.push(ZOBRIST.side_to_move_key);

        let total = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), total, "duplicate Zobrist keys generated");
    }
}