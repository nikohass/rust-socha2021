use argparse::{ArgumentParser, Store};
use game_sdk::{Action, GameState};
mod evaluation_cache;
use evaluation_cache::EvaluationCache;
use player::search::random_action;
use player::search::Searcher;
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct EvaluatedState {
    pub state: GameState,
    pub best_action: Action,
}

impl EvaluatedState {
    pub fn from_state(state: GameState) -> EvaluatedState {
        EvaluatedState {
            state,
            best_action: Action::Skip,
        }
    }

    pub fn evaluate(&mut self, searcher: &mut Searcher) -> Action {
        self.best_action = searcher.search_action(&self.state);
        self.best_action
    }

    pub fn to_fen(&self) -> String {
        let mut ret = self.state.to_fen();
        ret.push(' ');
        ret.push_str(&self.best_action.serialize());
        ret
    }
}

fn generate_evaluated_states(path: &str) {
    let mut rng = SmallRng::from_entropy();
    let mut searcher = Searcher::new(2000, true, 1);
    let mut cache = EvaluationCache::from_file("cache.txt", 100_000_000);
    let mut last_saved = 0;
    loop {
        let mut state = GameState::new();
        while !state.is_game_over() && state.ply < 20 {
            let mut evaluated_state = EvaluatedState::from_state(state.clone());
            let mut cache_action = Action::Skip;
            let mut cache_hit = false;

            let cache_entry = cache.lookup(&state);
            if let Some(cache_entry) = cache_entry {
                cache_action = cache_entry;
                cache_hit = true;
            }

            let next_action = if !cache_hit {
                let next_action = evaluated_state.evaluate(&mut searcher);
                cache.insert(&state, &next_action, searcher.depth_reached);
                next_action
            } else {
                println!("Found action in cache");
                cache_action
            };

            if rng.next_u64() as usize % 100 < 90 {
                println!("Searched action");
                state.do_action(next_action);
            } else {
                println!("Random action");
                state.do_action(random_action(&state));
            }

            println!("{}", state);
            if evaluated_state.best_action != Action::Skip && !cache_hit {
                save(&evaluated_state, path);
                if last_saved > 10 {
                    cache.merge("cache.txt");
                    println!("Saved cache");
                    last_saved = 0;
                    cache = EvaluationCache::from_file("cache.txt", 100_000_000);
                }
                last_saved += 1;
            }
        }
    }
}

fn save(evaluated_state: &EvaluatedState, path: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    if let Err(e) = writeln!(file, "{}", evaluated_state.to_fen()) {
        println!("Couldn't write to file: {}", e);
    }
}

fn main() {
    let mut path = "dataset.txt".to_string();
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut path)
            .add_option(&["-p", "--path"], Store, "File path");
        parser.parse_args_or_exit();
    }
    generate_evaluated_states(&path);
}
