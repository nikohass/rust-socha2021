use game_sdk::*;
use player::mcts::MCTS;
//use player::search::random_action;
use player::search::Searcher;

fn main() {
    let mut mcts = MCTS::new(19000);
    let mut searcher = Searcher::new(19000, "weights");

    let mut state = GameState::new();
    println!("{}", state);

    while !state.is_game_over() {
        let action = if state.ply & 0b1 == 0 {
            mcts.search_action(&state)
        } else {
            searcher.search_action(&state)
        };
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
