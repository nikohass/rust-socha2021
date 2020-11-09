use argparse::{ArgumentParser, Store};
use game_sdk::GameState;
use player::search::search_action;
use std::io;

fn main() {
    let mut time: u64 = 200;
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut time)
            .add_option(&["-t", "--time"], Store, "Time in milliseconds");
        parser.parse_args_or_exit();
    }

    loop {
        let mut fen = String::new();
        io::stdin().read_line(&mut fen).expect("Can't read line");
        fen.pop(); // remove \n
        let state = GameState::from_fen(fen.clone());
        let action = search_action(&state, time);
        println!("action: {}", action.serialize());
    }
}
