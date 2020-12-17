use game_sdk::GameState;
use player::search::Searcher;
use std::io;

pub fn run_test_client(mut searcher: Searcher) {
    loop {
        let mut fen = String::new();
        io::stdin().read_line(&mut fen).expect("Can't read line");
        fen.pop(); // remove \n
        let state = GameState::from_fen(fen.clone());
        let action = searcher.search_action(&state);
        println!("action: {}", action.serialize());
    }
}
