use super::cache::Cache;
use super::principal_variation_search::principal_variation_search;
use game_sdk::{Action, ActionList, ActionListStack, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 30;
pub const MAX_SCORE: i16 = i16::MAX;
pub const MATE_SCORE: i16 = -32000;

pub fn random_action(state: &GameState) -> Action {
    let state = state.clone();
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();
    state.get_possible_actions(&mut action_list);
    let rand = rng.next_u64() as usize % action_list.size;
    action_list[rand]
}

pub struct SearchParameters {
    pub nodes_searched: u64,
    pub root_ply: u8,
    pub start_time: Instant,
    pub stop: bool,
    pub action_list_stack: ActionListStack,
    pub principal_variation: ActionList,
    pub pv_table: ActionListStack,
    pub transposition_table: Cache,
    pub time: u128,
}

pub fn search_action(state: &GameState, time: u64) -> Action {
    println!("Searching action for {}...", state.to_fen());

    let time = time as u128;
    let mut params = SearchParameters {
        nodes_searched: 0,
        root_ply: state.ply,
        start_time: Instant::now(),
        stop: false,
        action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
        principal_variation: ActionList::default(),
        transposition_table: Cache::with_size(60_000_000),
        pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH + 2),
        time,
    };

    let mut state = state.clone();
    state.hash = 0;
    let mut score = -MAX_SCORE;
    let mut best_action = Action::Skip;
    for depth in 1..=usize::max(MAX_SEARCH_DEPTH, 101 - state.ply as usize) {
        score =
            principal_variation_search(&mut params, &mut state, -MAX_SCORE, MAX_SCORE, 0, depth);
        print!("depth: {:3} score: {:5} ", depth, score);

        if params.stop {
            break;
        }
        params.principal_variation = params.pv_table[0].clone();
        best_action = params.principal_variation[0];

        print!("pv: ");
        for i in 0..params.principal_variation.size {
            print!("{:20}, ", params.principal_variation[i]);
        }
        println!();
    }
    println!(
        "\nSearch finished after {}ms. Score: {}, nodes searched: {}",
        params.start_time.elapsed().as_millis(),
        score,
        params.nodes_searched,
    );
    best_action
}
