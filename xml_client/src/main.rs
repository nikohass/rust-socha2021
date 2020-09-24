//use argparse::{ArgumentParser, Store};
use game_sdk::gamestate::GameState;
use player::search::{random_action, search_action};

fn main() {
    let mut state = GameState::new();
    while !state.is_game_over() {
        let action = if state.ply % 4 == 0 || state.ply % 4 == 2 {
            search_action(&state)
        } else {
            random_action(&state)
        };
        println!("{}", action.to_string());
        state.do_action(action);
        println!("{}", state);
    }
    println!("{}", state.game_result());
}
