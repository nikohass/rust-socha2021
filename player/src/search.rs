use super::cache::Cache;
use super::neural_network::NeuralNetwork;
use super::principal_variation_search::principal_variation_search;
use game_sdk::{Action, ActionList, ActionListStack, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 30;
pub const MAX_SCORE: i16 = i16::MAX;
pub const MATE_SCORE: i16 = -32_000;

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

pub struct Searcher {
    pub search_time: u128,
    pub dont_cancel: bool,
    pub verbose: usize,
    pub neural_network: NeuralNetwork,
    pub depth_reached: u8,
}

impl Searcher {
    pub fn new(
        search_time: u128,
        dont_cancel: bool,
        verbose: usize,
        weights_file: &str,
    ) -> Searcher {
        let mut neural_network = NeuralNetwork::new();
        neural_network.load_weights(weights_file);
        Searcher {
            search_time,
            dont_cancel,
            verbose,
            neural_network,
            depth_reached: 0,
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        if self.verbose > 0 {
            println!("Searching action for {}...", state.to_fen());
            if self.verbose == 2 {
                println!("{}", state);
            }
        }

        let mut params = SearchParameters {
            nodes_searched: 0,
            root_ply: state.ply,
            start_time: Instant::now(),
            stop: false,
            action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            principal_variation: ActionList::default(),
            transposition_table: Cache::with_size(60_000_000),
            pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH + 2),
            time: if self.dont_cancel {
                100_000_000u128
            } else {
                self.search_time
            },
        };

        let mut state = state.clone();
        //let (nn_action, _) = self.neural_network.choose_action(&state);
        //return nn_action;

        /*
        if state.ply < 20 {
            let mut test_state = state.clone();
            for i in 0..(20 - state.ply as usize) {
                let (nn_action, confidence) = self.neural_network.choose_action(&test_state);
                test_state.do_action(nn_action);
                params.principal_variation.push(nn_action);
                if i > 5 {
                    break;
                }
            }
            for i in 0..params.principal_variation.size {
                println!("{}", params.principal_variation[i].to_string());
            }
        }*/

        state.hash = 0;
        let mut score = -MAX_SCORE;
        let mut best_action = Action::Skip;
        for depth in 1..=usize::max(MAX_SEARCH_DEPTH, 101 - state.ply as usize) {
            /*
            let mut toy_state = state.clone();
            for index in 0..params.principal_variation.size {
                toy_state.do_action(params.principal_variation[index]);
            }
            let (nn_action, confidence) = self.neural_network.choose_action(&toy_state);
            if self.verbose > 0 {
                println!("nn_action {}, conf: {}", nn_action.to_string(), confidence);
            }
            params.principal_variation.push(nn_action);*/

            score = principal_variation_search(
                &mut params,
                &mut state,
                -MAX_SCORE,
                MAX_SCORE,
                0,
                depth,
            );
            if self.verbose > 0 {
                print!("Depth {:3} Score {:5} ", depth, score);
            }
            if params.stop {
                break;
            }
            self.depth_reached = depth as u8;
            params.principal_variation = params.pv_table[0].clone();
            best_action = params.principal_variation[0];

            if self.verbose > 0 {
                print!("pv: ");
                for i in 0..params.principal_variation.size {
                    print!("{:20}, ", params.principal_variation[i]);
                }
                println!();
            }

            if self.dont_cancel && params.start_time.elapsed().as_millis() > self.search_time {
                break;
            }
        }
        if self.verbose > 0 {
            println!(
                "\nSearch finished after {}ms. Score: {}, nodes searched: {}",
                params.start_time.elapsed().as_millis(),
                score,
                params.nodes_searched,
            );
        }
        best_action
    }
}
