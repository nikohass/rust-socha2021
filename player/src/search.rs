use super::cache::Cache;
use super::principal_variation_search::principal_variation_search;
use game_sdk::action::Action;
use game_sdk::actionlist::{ActionList, ActionListStack};
use game_sdk::gamestate::GameState;
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 25;
pub const MATE_SCORE: i16 = i16::MAX;

pub fn random_action(state: &GameState) -> Action {
    let state = state.clone();
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();
    state.get_possible_actions(&mut action_list);
    let rand = rng.next_u64() as usize % action_list.size;
    action_list[rand]
}

pub struct SearchParams {
    pub nodes_searched: u64,
    pub root_ply: u8,
    pub target_depth: usize,
    pub start_time: Instant,
    pub stop: bool,
    pub action_list_stack: ActionListStack,
    pub best_action: Action,
    pub best_score: i16,
    pub transposition_table: Cache,
}

pub fn search_action(state: &GameState) -> Action {
    let mut state = state.clone();

    let mut params = SearchParams {
        nodes_searched: 0,
        root_ply: state.ply,
        target_depth: 1,
        start_time: Instant::now(),
        stop: false,
        action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
        best_action: Action::Skip,
        best_score: -MATE_SCORE,
        transposition_table: Cache::new(),
    };

    let mut score = -MATE_SCORE;
    let mut best_action = params.best_action;
    for depth in 1..=MAX_SEARCH_DEPTH {
        params.target_depth = depth;

        score = principal_variation_search(&mut params, &mut state, -MATE_SCORE, MATE_SCORE, 0);
        println!("depth {:2}; score: {:3}", depth, score);

        if params.stop {
            break;
        }
        params.best_score = -MATE_SCORE;
        best_action = params.best_action;
    }

    println!("score: {}; nodes: {}", score, params.nodes_searched);
    params.best_action
}
