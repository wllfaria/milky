use crate::MATE_LOWER_BOUND;
use crate::zobrist::ZobristKey;

static ONE_MB: usize = 0x100000;
static TT_SIZE_MB: usize = 4;
static TT_SIZE_BYTES: usize = ONE_MB * TT_SIZE_MB;
static TT_ENTRY_COUNT: usize = TT_SIZE_BYTES / std::mem::size_of::<TTEntry>();

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum TTFlag {
    #[default]
    Beta,
    Alpha,
    Exact,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TTEntry {
    pub key: ZobristKey,
    pub score: i32,
    pub depth: u8,
    pub flag: TTFlag,
}

#[derive(Debug)]
pub struct TranspositionTable {
    entries: Vec<TTEntry>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            entries: vec![TTEntry::default(); TT_ENTRY_COUNT],
        }
    }

    fn index(&self, key: ZobristKey) -> usize {
        key.inner() as usize % TT_ENTRY_COUNT
    }

    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::default());
    }

    pub fn get(
        &self,
        key: ZobristKey,
        alpha: i32,
        beta: i32,
        depth: u8,
        ply: usize,
    ) -> Option<i32> {
        let entry = self.entries[self.index(key)];

        if entry.key != key {
            return None;
        }

        if entry.depth < depth {
            return None;
        }

        let mut score = entry.score;
        if score < -MATE_LOWER_BOUND {
            score += ply as i32
        }

        if score > MATE_LOWER_BOUND {
            score -= ply as i32;
        }
        match entry.flag {
            TTFlag::Exact => Some(score),
            TTFlag::Alpha if score <= alpha => Some(alpha),
            TTFlag::Beta if score >= beta => Some(beta),
            _ => None,
        }
    }

    pub fn set(&mut self, key: ZobristKey, depth: u8, mut score: i32, flag: TTFlag, ply: usize) {
        let index = self.index(key);

        if score < -MATE_LOWER_BOUND {
            score -= ply as i32
        }

        if score > MATE_LOWER_BOUND {
            score += ply as i32;
        }

        self.entries[index] = TTEntry {
            key,
            depth,
            score,
            flag,
        };
    }
}
