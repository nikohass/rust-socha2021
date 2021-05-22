use game_sdk::Action;

pub struct TranspositionTable {
    cache: Vec<TranspositionTableEntry>,
    entries: usize,
}

impl TranspositionTable {
    pub fn with_size(entries: usize) -> Self {
        let cache = vec![TranspositionTableEntry::empty(); entries];
        Self { cache, entries }
    }

    pub fn insert(&mut self, hash: u64, entry: TranspositionTableEntry) {
        let index = hash as usize % self.entries;
        let current_entry = self.cache[index];
        if entry.ply < current_entry.ply {
            self.cache[index] = entry;
        }
    }

    pub fn lookup(&self, hash: u64) -> TranspositionTableEntry {
        self.cache[hash as usize % self.entries]
    }
}

#[derive(Copy, Clone)]
pub struct TranspositionTableEntry {
    pub action: Action,
    pub score: i16,
    pub ply: u8,
    pub depth_left: u8,
    pub alpha: bool,
    pub beta: bool,
    pub hash: u64,
}

impl TranspositionTableEntry {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.depth_left == std::u8::MAX
    }

    pub fn empty() -> Self {
        Self {
            action: Action::SKIP,
            score: 0,
            ply: 0,
            depth_left: std::u8::MAX,
            alpha: false,
            beta: false,
            hash: 0,
        }
    }
}

pub struct EvaluationCache {
    pub cache: Vec<EvaluationCacheEntry>,
    pub entries: usize,
}

impl EvaluationCache {
    pub fn with_size(entries: usize) -> EvaluationCache {
        EvaluationCache {
            cache: vec![EvaluationCacheEntry::empty(); entries],
            entries,
        }
    }

    pub fn insert(&mut self, hash: u64, score: i16) {
        self.cache[hash as usize % self.entries] = EvaluationCacheEntry { hash, score };
    }

    pub fn lookup(&self, hash: u64) -> EvaluationCacheEntry {
        self.cache[hash as usize % self.entries]
    }
}

#[derive(Copy, Clone)]
pub struct EvaluationCacheEntry {
    pub hash: u64,
    pub score: i16,
}

impl EvaluationCacheEntry {
    pub fn empty() -> Self {
        Self {
            hash: 0,
            score: std::i16::MIN,
        }
    }
}
