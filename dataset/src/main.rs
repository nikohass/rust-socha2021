use argparse::{ArgumentParser, Store};
use game_sdk::GameState;
use player::mcts::search::Mcts;

fn main() {
    let mut fen = "".to_string();
    let mut iterations = 500_000;
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut fen)
            .add_option(&["-f", "--fen"], Store, "Fen");
        parser
            .refer(&mut iterations)
            .add_option(&["-i", "--iterations"], Store, "iterations");
        parser.parse_args_or_exit();
    }
    let mut mcts = Mcts::default();
    mcts.set_iteration_limit(iterations);
    let state = GameState::from_fen(fen);
    println!("{}", state);
    let action = mcts.search_action(&state);
    let mut string = "".to_string();
    for pair in mcts.get_action_value_pairs().iter() {
        string.push_str(&format!("{} {} ", pair.0.serialize(), pair.1));
    }
    println!(
        "result: {} {}{}",
        action.serialize(),
        string,
        mcts.get_value()
    );
}
