use argparse::{ArgumentParser, Store};
use game_sdk::{Action, GameState};
mod evaluation_cache;
use evaluation_cache::EvaluationCache;
//use player::mcts::MCTS;
use player::search::{random_action, Searcher};
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
    let mut cache = EvaluationCache::from_file("cache.txt", 100_000_000);
    let mut last_saved = 0;
    let mut searcher = Searcher::new(2_000, "weights");

    loop {
        let mut state = GameState::new();
        println!("{}", state.start_piece_type);
        while !state.is_game_over() && state.ply < 30 {
            println!("{}", state.to_fen());
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
                if next_action != Action::Skip {
                    save(&evaluated_state, path);
                    last_saved += 1;
                    cache.insert(&state, &next_action, 0);
                }
                next_action
            } else {
                println!("Found action in cache");
                cache_action
            };

            if rng.next_u64() as usize % 100 > 95 && cache_hit {
                println!("Random action");
                state.do_action(random_action(&state));
            } else {
                println!("Searched action");
                state.do_action(next_action);
            }
            println!("{}", state);

            if last_saved > 100 {
                cache.merge("cache.txt");
                println!("Saved cache");
                last_saved = 0;
                cache = EvaluationCache::from_file("cache.txt", 100_000_000);
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
    let mut path = "datasets/dataset.txt".to_string();
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut path)
            .add_option(&["-p", "--path"], Store, "File path");
        parser.parse_args_or_exit();
    }
    generate_evaluated_states(&path);
}
