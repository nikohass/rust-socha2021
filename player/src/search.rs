use super::cache::{EvaluationCache, TranspositionTable};
use super::principal_variation_search::principal_variation_search;
use game_sdk::{Action, ActionList, ActionListStack, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 30;
pub const MAX_SCORE: i16 = i16::MAX;
pub const MATE_SCORE: i16 = -32_000;
pub const TT_SIZE: usize = 20_000_000;
pub const EVAL_CACHE_SIZE: usize = 1_000_000;

pub fn random_action(state: &GameState) -> Action {
    let state = state.clone();
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();
    state.get_possible_actions(&mut action_list);
    let rand = rng.next_u64() as usize % action_list.size;
    action_list[rand]
}

pub struct Searcher {
    pub nodes_searched: u64,
    pub depth_reached: u8,
    pub root_ply: u8,
    pub stop: bool,
    pub action_list_stack: ActionListStack,
    pub principal_variation: ActionList,
    pub pv_table: ActionListStack,
    pub transposition_table: TranspositionTable,
    pub evaluation_cache: EvaluationCache,
    pub start_time: Instant,
    pub time_limit: u128,
    pub dont_cancel: bool,
}

impl Searcher {
    pub fn new(time_limit: u128) -> Searcher {
        Searcher {
            nodes_searched: 0,
            depth_reached: 0,
            root_ply: 0,
            stop: false,
            action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            principal_variation: ActionList::default(),
            pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH + 2),
            transposition_table: TranspositionTable::with_size(TT_SIZE),
            evaluation_cache: EvaluationCache::with_size(EVAL_CACHE_SIZE),
            start_time: Instant::now(),
            time_limit,
            dont_cancel: false,
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        println!("Searching action for {}...", state.to_fen());
        let mut state = state.clone();
        state.hash = 0;
        self.nodes_searched = 0;
        self.root_ply = state.ply;
        self.start_time = Instant::now();
        self.stop = false;
        self.principal_variation.size = 0;
        self.time_limit = if self.dont_cancel {
            100_000_000u128
        } else {
            self.time_limit
        };

        let mut score = -MAX_SCORE;
        let mut best_action = Action::Skip;
        let mut last_principal_variation_size: usize = 0;
        for depth in 1..=MAX_SEARCH_DEPTH {
            score = principal_variation_search(self, &mut state, -MAX_SCORE, MAX_SCORE, 0, depth);
            print!("Depth {:3} Score {:5} ", depth, score);
            if self.stop {
                println!("(canceled)");
                break;
            }
            self.depth_reached = depth as u8;
            self.principal_variation = self.pv_table[0].clone();
            best_action = self.principal_variation[0];

            if self.principal_variation.size == last_principal_variation_size {
                println!("\nReached the end of the search tree.");
                break;
            }
            last_principal_variation_size = self.principal_variation.size;
            println!("{}", self.format_principal_variation());

            if self.dont_cancel && self.start_time.elapsed().as_millis() > self.time_limit {
                break;
            }
        }
        println!(
            "Search finished after {}ms. Score: {}, nodes searched: {}",
            self.start_time.elapsed().as_millis(),
            score,
            self.nodes_searched,
        );
        best_action
    }

    pub fn reset(&mut self) {
        self.transposition_table = TranspositionTable::with_size(TT_SIZE);
        self.evaluation_cache = EvaluationCache::with_size(EVAL_CACHE_SIZE);
        self.depth_reached = 0;
        self.nodes_searched = 0;
    }

    pub fn format_principal_variation(&self) -> String {
        let mut ret = "pv: ".to_string();
        for i in 0..self.principal_variation.size {
            if i != 0 {
                ret.push_str(", ");
            }
            ret.push_str(&self.principal_variation[i].to_short_name());
        }
        ret
    }
}
