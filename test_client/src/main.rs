use argparse::{ArgumentParser, Store};
use game_sdk::GameState;
use player::search::Searcher;
use std::io;

fn main() {
    let mut time: u128 = 1900;
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut time)
            .add_option(&["-t", "--time"], Store, "Time in milliseconds");
        parser.parse_args_or_exit();
    }

    let mut searcher = Searcher::new(time, false, 0);
    loop {
        let mut fen = String::new();
        io::stdin().read_line(&mut fen).expect("Can't read line");
        fen.pop(); // remove \n
        let state = GameState::from_fen(fen.clone());
        let action = searcher.search_action(&state);
        println!("action: {}", action.serialize());
    }
}
