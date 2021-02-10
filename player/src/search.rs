use super::cache::{EvaluationCache, TranspositionTable};
use super::neural_network::NeuralNetwork;
use super::principal_variation_search::principal_variation_search;
use game_sdk::{Action, ActionList, ActionListStack, GameState, Player};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 40;
pub const MAX_SCORE: i16 = std::i16::MAX;
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
    pub root_ply: u8,
    pub stop: bool,
    pub action_list_stack: ActionListStack,
    pub principal_variation: ActionList,
    pub pv_table: ActionListStack,
    pub transposition_table: TranspositionTable,
    pub evaluation_cache: EvaluationCache,
    pub start_time: Instant,
    pub time_limit: u128,
    pub neural_network: Option<NeuralNetwork>,
}

impl Searcher {
    pub fn new(time_limit: u128, weights_file: &str) -> Searcher {
        let mut neural_network = NeuralNetwork::policy_network();
        let neural_network = if neural_network.load_weights(weights_file) {
            Some(neural_network)
        } else {
            None
        };
        Searcher {
            nodes_searched: 0,
            root_ply: 0,
            stop: false,
            action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            principal_variation: ActionList::default(),
            pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            transposition_table: TranspositionTable::with_size(TT_SIZE),
            evaluation_cache: EvaluationCache::with_size(EVAL_CACHE_SIZE),
            start_time: Instant::now(),
            neural_network,
            time_limit,
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        println!("Searching action using PV-Search");
        println!("Depth    Time   Score     Nodes PV-prediction   Conf     Nodes/s PV");
        let mut state = state.clone();
        self.nodes_searched = 0;
        self.root_ply = state.ply;
        self.start_time = Instant::now();
        self.stop = false;
        self.principal_variation.clear();

        let mut score = -MAX_SCORE;
        let mut best_action = Action::Skip;
        let mut last_principal_variation_size: usize = 0;
        for depth in 1..=MAX_SEARCH_DEPTH {
            let depth_start_time = Instant::now();
            let (nn_action, confidence) = if let Some(neural_network) = &self.neural_network {
                neural_network.append_principal_variation(&mut self.principal_variation, &state)
            } else {
                (Action::Skip, std::f32::NEG_INFINITY)
            };
            let current_score =
                principal_variation_search(self, &mut state, -MAX_SCORE, MAX_SCORE, 0, depth);
            let time = self.start_time.elapsed().as_millis();
            print!(
                "{:5} {:5}ms {:7} {:9} {:12} {:7.3} {:11.1} ",
                depth,
                time,
                current_score,
                self.nodes_searched,
                nn_action.to_short_name(),
                confidence,
                (self.nodes_searched as f64) / (time as f64) * 1000.
            );
            if self.stop {
                println!("(canceled)");
                break;
            }
            score = current_score;
            self.principal_variation = self.pv_table[0].clone();
            best_action = self.principal_variation[0];

            if self.principal_variation.size == last_principal_variation_size {
                println!("\nReached the end of the search tree.");
                break;
            }
            last_principal_variation_size = self.principal_variation.size;
            println!("{}", format_principal_variation(&self.principal_variation));

            if depth_start_time.elapsed().as_millis() > (self.time_limit - time) / 2 {
                break;
            }
        }
        println!(
            "Search finished after {}ms. Score: {} Nodes: {} Nodes/s: {:.3} PV: {}",
            self.start_time.elapsed().as_millis(),
            score,
            self.nodes_searched,
            self.nodes_searched as f64 / self.start_time.elapsed().as_millis() as f64 * 1000.,
            format_principal_variation(&self.principal_variation),
        );
        best_action
    }

    pub fn reset(&mut self) {
        self.transposition_table = TranspositionTable::with_size(TT_SIZE);
        self.evaluation_cache = EvaluationCache::with_size(EVAL_CACHE_SIZE);
        self.nodes_searched = 0;
        self.root_ply = 0;
        self.stop = false;
    }
}

impl Player for Searcher {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search_action(state)
    }

    fn on_reset(&mut self) {
        self.reset();
    }
}

pub fn format_principal_variation(principal_variation: &ActionList) -> String {
    let mut ret = String::new();
    for i in 0..principal_variation.size {
        if i != 0 {
            ret.push_str(", ");
        }
        ret.push_str(&principal_variation[i].to_short_name());
    }
    ret
}
