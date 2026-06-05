use super::mv::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    // Exact value of the node (alpha < score < beta).
    Exact,
    // Score is a lower bound: a beta cutoff occurred (score >= beta).
    Lower,
    // Score is an upper bound: no move beat alpha (score <= alpha).
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub struct TtEntry {
    pub key: u64,
    pub depth: i32,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TtEntry>>,
    mask: usize,
}

impl TranspositionTable {
    pub fn with_capacity_mb(megabytes: usize) -> TranspositionTable {
        let entry_size = std::mem::size_of::<Option<TtEntry>>();
        let wanted = (megabytes * 1024 * 1024) / entry_size;
        let count = wanted.next_power_of_two() / 2;
        let count = count.max(1);
        TranspositionTable {
            entries: vec![None; count],
            mask: count - 1,
        }
    }

    #[inline]
    fn index(&self, hash: u64) -> usize {
        hash as usize & self.mask
    }

    pub fn probe(&self, hash: u64) -> Option<&TtEntry> {
        match &self.entries[self.index(hash)] {
            Some(entry) if entry.key == hash => Some(entry),
            _ => None,
        }
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: i32,
        score: i32,
        bound: Bound,
        best_move: Option<Move>,
    ) {
        let idx = self.index(hash);
        self.entries[idx] = Some(TtEntry {
            key: hash,
            depth,
            score,
            bound,
            best_move,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_then_probe_roundtrips() {
        let mut tt = TranspositionTable::with_capacity_mb(1);
        tt.store(0xDEAD_BEEF, 5, 42, Bound::Exact, None);
        let entry = tt.probe(0xDEAD_BEEF).expect("entry present");
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 42);
        assert_eq!(entry.bound, Bound::Exact);
    }

    #[test]
    fn probe_miss_on_unknown_key() {
        let tt = TranspositionTable::with_capacity_mb(1);
        assert!(tt.probe(0x1234).is_none());
    }

    #[test]
    fn capacity_is_power_of_two() {
        let tt = TranspositionTable::with_capacity_mb(16);
        assert!(tt.entries.len().is_power_of_two());
    }
}