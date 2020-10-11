use game_sdk::Action;

const ENTRIES: usize = 1024 * 64;

#[derive(Clone, Copy)]
pub struct CacheEntry {
    pub score: i16,
    pub action: Action,
    pub depth: u8,
    pub alpha: bool,
    pub beta: bool,
}

pub struct Cache {
    pub entries: Vec<Option<CacheEntry>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            entries: vec![None; ENTRIES],
        }
    }

    pub fn lookup(&self, hash: u64) -> Option<CacheEntry> {
        let index = hash % ENTRIES as u64;
        self.entries[index as usize]
    }

    pub fn insert(&mut self, hash: u64, entry: CacheEntry) {
        let index = hash % ENTRIES as u64;
        self.entries[index as usize] = Some(entry);
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
