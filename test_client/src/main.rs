use game_sdk::gamestate::GameState;
use player::search::search_action;
use std::io;

fn main() {
    loop {
        let mut fen = String::new();
        io::stdin().read_line(&mut fen).expect("Can't read line");
        fen.pop(); // remove \n
        let state = GameState::from_fen(fen.clone());
        let action = search_action(&state);
        println!("{}", action.serialize());
    }
}
