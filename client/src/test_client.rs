use game_sdk::{GameState, Player};
use std::io;

pub fn run_test_client(mut player: Box<dyn Player>) {
    loop {
        let mut fen = String::new();
        io::stdin().read_line(&mut fen).expect("Can't read line");
        fen.pop(); // remove \n
        let state = GameState::from_fen(fen.clone());
        if state.ply < 2 {
            player.on_reset();
        }
        let action = player.on_move_request(&state);
        println!("action: {}", action.serialize());
    }
}
