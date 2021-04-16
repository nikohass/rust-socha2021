use argparse::{ArgumentParser, Store};
use game_sdk::{Action, Bitboard, GameState, Player};
use player::mcts::Mcts;
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::thread;

pub struct Example {
    pub state: GameState,
    pub value_map: Vec<u16>,
    pub best_action: Action,
}

fn save_examples(examples: &mut Vec<Example>, path: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut string = String::new();
    for example in examples.iter_mut() {
        string.push_str(&example.state.to_fen());
        for v in example.value_map.iter() {
            string.push(' ');
            string.push_str(&v.to_string());
        }
        string.push_str(&format!(" {}", example.best_action.serialize()));
        string.push('\n');
    }
    string.pop();
    if let Err(e) = writeln!(file, "{}", string) {
        println!("Couldn't write to file: {}", e);
    }
    examples.truncate(0);
}

fn sort(action_value_pairs: &mut Vec<(Action, f32)>) {
    for i in 0..action_value_pairs.len() {
        let mut max_value = std::f32::NEG_INFINITY;
        let mut best_index = 0;
        for (j, pair) in action_value_pairs.iter().enumerate().skip(i) {
            let value = pair.1;
            if value > max_value {
                max_value = value;
                best_index = j;
            }
        }
        action_value_pairs.swap(i, best_index);
    }
}

fn get_y(player: &Mcts) -> Vec<u16> {
    let mut action_value_pairs = player.get_action_value_pairs();
    sort(&mut action_value_pairs);
    let mut value_map: Vec<f32> = vec![0.; 400];
    let mut frequency_map: Vec<f32> = vec![0.; 400];
    for (action, value) in action_value_pairs.iter() {
        let mut action_board =
            Bitboard::with_piece(action.get_destination(), action.get_shape() as usize);
        while action_board.not_zero() {
            let field_index = action_board.trailing_zeros();
            action_board.flip_bit(field_index);
            let x = field_index as usize % 21;
            let y = (field_index as usize - x) / 21;
            let index = x + y * 20;
            frequency_map[index] += 1.0;
            value_map[index] += value;
        }
    }
    for (i, value) in value_map.iter_mut().enumerate() {
        *value /= frequency_map[i];
    }
    let mut y: Vec<u16> = Vec::with_capacity(401);
    for value in value_map.iter_mut() {
        *value *= (std::u16::MAX) as f32;
        y.push(*value as u16);
    }
    y.push((player.get_value() * (std::u16::MAX) as f32) as u16);
    y
}

fn generate_dataset(path: &str) {
    let mut player = Mcts::default();
    player.set_time_limit(None);
    player.set_neural_network(None);
    player.set_iteration_limit(Some(500_000));
    let mut opponent = Mcts::default();
    opponent.set_neural_network(None);
    opponent.set_time_limit(None);
    opponent.set_iteration_limit(Some(6_000));

    let mut rng = SmallRng::from_entropy();
    let mut examples: Vec<Example> = Vec::with_capacity(300);
    let mut team = 0;
    loop {
        let mut state = GameState::random();
        while !state.is_game_over() && state.ply < 12 {
            println!("{}", state);
            if state.ply % 2 != team
                || state.ply < 4
                || rng.next_u64() as f64 / (std::u64::MAX as f64) > 0.95
            {
                state.do_action(opponent.on_move_request(&state));
                opponent.on_reset();
                continue;
            }
            player.on_reset();
            let action = player.on_move_request(&state);
            if action.is_set() {
                let y = get_y(&player);
                examples.push(Example {
                    state: state.clone(),
                    value_map: y,
                    best_action: action,
                });
            }
            state.do_action(action);
            //println!("{}", state);
            save_examples(&mut examples, path);
            //println!("Saved");
        }
        team = (team + 1) % 2;
    }
}

fn main() {
    let mut path = "datasets/1.txt".to_string();
    let mut threads = 3;
    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut path)
            .add_option(&["-p", "--path"], Store, "Path");
        parser
            .refer(&mut threads)
            .add_option(&["-t", "--threads"], Store, "Threads");
        parser.parse_args_or_exit();
    }

    let mut children = vec![];
    for _ in 0..threads {
        let p = path.clone();
        children.push(thread::spawn(move || {
            generate_dataset(&p);
        }));
    }

    for child in children {
        let _ = child.join();
    }
}
