use game_sdk::Action;

pub struct Cache {
    cache: Vec<CacheEntry>,
    entries: usize,
}

impl Cache {
    pub fn with_size(entries: usize) -> Cache {
        let cache = vec![CacheEntry::empty(); entries];
        Cache { cache, entries }
    }

    pub fn insert(&mut self, hash: u64, entry: CacheEntry) {
        self.cache[hash as usize % self.entries] = entry;
    }

    pub fn lookup(&self, hash: u64) -> CacheEntry {
        self.cache[hash as usize % self.entries]
    }
}

#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub action: Action,
    pub score: i16,
    pub depth: u8,
    pub depth_left: u8,
    pub alpha: bool,
    pub beta: bool,
}

impl CacheEntry {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.depth_left == std::u8::MAX
    }

    pub fn empty() -> CacheEntry {
        CacheEntry {
            action: Action::Skip,
            score: 0,
            depth: 0,
            depth_left: std::u8::MAX,
            alpha: false,
            beta: false,
        }
    }
}
