use game_sdk::{Action, ActionList, GameState};
use player::mcts::MCTS;
//use player::simple_client::SimpleClient;
use std::fs::OpenOptions;
use std::io::prelude::*;

fn save(
    states: &[GameState],
    actions: &mut ActionList,
    result: i16,
    values: &mut [f32; 100],
    path: &str,
) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    for (i, state) in states.iter().enumerate() {
        let fen = format!(
            "{} {} {} {}",
            state.to_fen(),
            actions[i].serialize(),
            values[i],
            result * state.current_color.team_i16()
        );
        if let Err(e) = writeln!(file, "{}", fen) {
            println!("Couldn't write to file: {}", e);
        }
    }
}

pub fn generate_dataset() {
    let mut mcts = MCTS::new(7100);
    let mut actions: ActionList = ActionList::default();
    let mut values: [f32; 100] = [0.; 100];
    let mut states: Vec<GameState> = Vec::with_capacity(100);
    let mut states_searched: u64 = 0;
    let mut games_played: u64 = 0;
    let mut sum_results: i64 = 0;
    let mut sum_plies: u64 = 0;
    let mut one_wins: u64 = 0;
    let mut draws: u64 = 0;

    loop {
        let mut state = GameState::new();
        while !state.is_game_over() && state.ply < 60 {
            println!("{}", state.to_fen());
            let (action, value) = mcts.search_action(&state);
            if action == Action::Skip {
                state.do_action(Action::Skip);
                continue;
            }
            states_searched += 1;
            values[state.ply as usize] = value;
            states.push(state.clone());
            actions.push(action);
            state.do_action(action);
        }
        sum_plies += state.ply as u64;
        games_played += 1;
        let result = state.game_result();
        match result {
            0 => draws += 1,
            r if r > 0 => one_wins += 1,
            _ => {}
        }
        sum_results += result as i64;
        save(
            &states,
            &mut actions,
            state.game_result(),
            &mut values,
            "datasets/dataset.txt",
        );
        actions.clear();
        states.truncate(0);

        println!(
            "Games: {} Searched: {} Average game length: {} plies Average result: {} Sum results: {} One: {} Draws: {} Two: {}",
            games_played,
            states_searched,
            sum_plies as f64 / games_played as f64,
            sum_results as f64 / games_played as f64,
            sum_results,
            one_wins,
            draws,
            games_played - one_wins - draws
        );
    }
}

pub fn generate_opening_dataset() {
    let mut mcts = MCTS::new(15_000);
    let mut simple_client = MCTS::new(500); //SimpleClient::default();
    let mut actions: ActionList = ActionList::default();
    let mut values: [f32; 100] = [f32::NAN; 100];
    let mut states: Vec<GameState> = Vec::with_capacity(100);
    let mut states_searched: u64 = 0;
    let mut games_played: usize = 0;

    loop {
        let mut state = GameState::new();
        while !state.is_game_over() && state.ply < 24 {
            println!("{}", state.to_fen());
            if state.ply as usize & 0b1 == games_played & 0b1 {
                let (action, _) = simple_client.search_action(&state);
                //state.do_action(simple_client.search_action(&state));
                state.do_action(action);
                println!("{}", state);
                continue;
            }
            let (action, _) = mcts.search_action(&state);
            if action == Action::Skip {
                state.do_action(Action::Skip);
                continue;
            }
            states_searched += 1;
            states.push(state.clone());
            actions.push(action);
            state.do_action(action);
            println!("{}", state);
        }
        games_played += 1;
        let result = std::i16::MAX;
        save(
            &states,
            &mut actions,
            result,
            &mut values,
            "datasets/openings.txt",
        );
        actions.clear();
        states.truncate(0);

        println!("Searched: {} ", states_searched,);
    }
}

fn main() {
    generate_dataset();
    //generate_opening_dataset();
}
