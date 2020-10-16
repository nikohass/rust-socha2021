use super::principal_variation_search::principal_variation_search;
use game_sdk::{Action, ActionList, ActionListStack, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 100;
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
    pub start_time: Instant,
    pub stop: bool,
    pub action_list_stack: ActionListStack,
    pub principal_variation: ActionList,
    pub pv_table: ActionListStack,
}

pub fn search_action(state: &GameState) -> Action {
    println!("Searching action for {}...", state.to_fen());
    let mut state = state.clone();

    let mut params = SearchParams {
        nodes_searched: 0,
        root_ply: state.ply,
        start_time: Instant::now(),
        stop: false,
        action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
        principal_variation: ActionList::default(),
        pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH + 2),
    };

    let mut score = -MATE_SCORE;
    let mut best_action = Action::Skip;
    for depth in 1..=MAX_SEARCH_DEPTH {
        score =
            principal_variation_search(&mut params, &mut state, -MATE_SCORE, MATE_SCORE, 0, depth);
        print!("depth: {:2} score: {:4} pv: ", depth, score);

        if params.stop {
            break;
        }
        params.principal_variation = params.pv_table[0].clone();
        for i in 0..params.principal_variation.size {
            print!("{:20}, ", params.principal_variation[i]);
        }
        println!();
        best_action = params.principal_variation[0];
    }

    println!("score: {} nodes: {}", score, params.nodes_searched);
    best_action
}
