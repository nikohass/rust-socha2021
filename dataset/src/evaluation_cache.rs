use game_sdk::{
    Action, GameState, EVALUATION_CACHE_HASH, EVALUATION_CACHE_PLY_HASH,
    EVALUATION_CACHE_START_PIECE_TYPE_HASH,
};
use player::neural_network::Rotation;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

pub struct EvaluationCache {
    pub size: usize,
    pub actions: Vec<Action>,
    pub hashes: Vec<u128>,
    pub plies: Vec<u8>,
    pub depth: Vec<u8>,
}

impl EvaluationCache {
    pub fn with_size(size: usize) -> EvaluationCache {
        EvaluationCache {
            size,
            actions: vec![Action::Skip; size],
            hashes: vec![0; size],
            plies: vec![255; size],
            depth: vec![0; size],
        }
    }

    pub fn from_file(filename: &str, size: usize) -> EvaluationCache {
        let mut cache = EvaluationCache::with_size(size);
        let file = File::open(filename.to_string()).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let entries: Vec<&str> = line.split(' ').collect();
            let hash = entries[0].parse::<u128>().unwrap();
            let index = (hash % size as u128) as usize;
            cache.hashes[index] = hash;
            cache.actions[index] = Action::deserialize(format!("{} {}", entries[1], entries[2]));
            cache.plies[index] = entries[3].parse::<u8>().unwrap();
            cache.depth[index] = entries[4].parse::<u8>().unwrap();
        }
        cache
    }

    pub fn lookup(&self, state: &GameState) -> Option<Action> {
        let rotation = Rotation::from_state(&state);
        let mut rotated_state = state.clone();
        rotation.rotate_state(&mut rotated_state);

        let hash = u128_hash(&rotated_state);
        let index = (hash % self.size as u128) as usize;
        let action = self.actions[index];

        if self.hashes[index] != hash || action == Action::Skip {
            None
        } else {
            Some(rotation.rotate_action(action))
        }
    }

    pub fn insert(&mut self, state: &GameState, action: &Action, depth: u8) {
        if *action != Action::Skip {
            let rotation = Rotation::from_state(&state);
            let mut rotated_state = state.clone();
            rotation.rotate_state(&mut rotated_state);
            let hash = u128_hash(&rotated_state);
            let index = (hash % self.size as u128) as usize;

            if self.should_replace(index, depth, state.ply) {
                self.hashes[index] = hash;
                self.actions[index] = rotation.rotate_action(*action);
                self.plies[index] = state.ply;
                self.depth[index] = depth;
            }
        }
    }

    pub fn should_replace(&self, index: usize, depth: u8, ply: u8) -> bool {
        self.plies[index] > ply || (self.plies[index] == ply && self.depth[index] < depth)
    }

    pub fn save(&mut self, filename: &str) {
        if self.size == 0 {
            panic!("Trying to save empty cache")
        }
        let mut file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(filename.to_string())
            .unwrap();
        for (i, hash) in self.hashes.iter().enumerate() {
            if *hash != 0 && self.actions[i] != Action::Skip {
                if let Err(e) = writeln!(
                    file,
                    "{} {} {} {}",
                    hash,
                    self.actions[i].serialize(),
                    self.plies[i],
                    self.depth[i]
                ) {
                    println!("Couldn't write to file: {}", e);
                }
            }
        }
    }

    pub fn merge(&mut self, filename: &str) {
        let mut loaded_cache = EvaluationCache::from_file(filename, self.size);

        for (i, hash) in self.hashes.iter().enumerate() {
            if *hash != 0
                && self.actions[i] != Action::Skip
                && loaded_cache.should_replace(i, self.depth[i], self.plies[i])
            {
                loaded_cache.hashes[i] = *hash;
                loaded_cache.plies[i] = self.plies[i];
                loaded_cache.actions[i] = self.actions[i];
                loaded_cache.depth[i] = self.depth[i];
                println!("Added {} to {}", self.actions[i], filename);
            }
        }
        loaded_cache.save(filename);
    }
}

fn u128_hash(state: &GameState) -> u128 {
    let mut hash = EVALUATION_CACHE_PLY_HASH[state.ply as usize]
        ^ EVALUATION_CACHE_START_PIECE_TYPE_HASH[state.start_piece_type as usize];
    //let (board, mirror, start_corner) = normalize_board(state.board);

    for (board_index, board) in state.board.iter().enumerate() {
        let mut board_copy = *board;
        while board_copy.not_zero() {
            let bit_index = board_copy.trailing_zeros();
            board_copy.flip_bit(bit_index);
            hash ^= EVALUATION_CACHE_HASH[bit_index as usize][board_index];
        }
    }
    hash
}
