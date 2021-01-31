pub mod cache;
pub mod evaluation;
pub mod mcts;
pub mod neural_network;
pub mod principal_variation_search;
pub mod search;

use game_sdk::{Action, GameState};

pub trait Player {
    fn on_move_request(&mut self, state: &GameState) -> Action;

    fn on_reset(&mut self) {}
}
