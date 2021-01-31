use game_sdk::*;
use player::mcts::MCTS;
//use player::search::random_action;
use player::search::Searcher;

fn main() {
    let mut mcts = MCTS::new(9000);
    let mut searcher = Searcher::new(1900, "weights");

    let mut state = GameState::new();
    println!("{}", state);

    while !state.is_game_over() {
        let action: Action;
        if state.ply & 0b1 == 0 {
            action = if state.ply >= 12 {
                mcts.search_action(&state)
            } else {
                searcher.search_action(&state)
            };
        } else {
            action = searcher.search_action(&state);
        }
        if !state.validate_action(&action) {
            println!("{}", action.visualize());
            println!("{}", action);
            state.do_action(action);
            println!("{}", state);
        }
        state.do_action(action);
        println!("{}", state);
        println!("{}", state.to_fen());
    }
    println!("{}", state.game_result());
}
